mod progress_icon;
mod toggle_button;

use adw::{prelude::*, subclass::prelude::*};
use anyhow::{Error, Result};
use gettextrs::gettext;
use gtk::{
    gio,
    glib::{self, clone},
};

use std::cell::RefCell;

use self::{progress_icon::ProgressIcon, toggle_button::ToggleButton};
use crate::{
    Application,
    cancelled::Cancelled,
    config::PROFILE,
    format,
    help::ContextWithHelp,
    preferences_dialog::PreferencesDialog,
    recording::{NoProfileError, Recording, RecordingState},
    settings::CaptureMode,
};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/seadve/Kooha/ui/window.ui")]
    pub struct Window {
        #[template_child]
        pub(super) title: TemplateChild<adw::WindowTitle>,
        #[template_child]
        pub(super) stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub(super) main_page: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub(super) forget_video_sources_revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub(super) recording_page: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) recording_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) recording_time_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) pause_record_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) delay_page: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) delay_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub(super) flushing_page: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) flushing_progress_icon: TemplateChild<ProgressIcon>,

        #[template_child]
        pub(super) quick_controls: TemplateChild<gtk::Box>,
        pub(super) bubble: RefCell<Option<crate::bubble::Bubble>>,
        pub(super) last_file: RefCell<Option<gio::File>>,
        pub(super) pending_copy: RefCell<Option<gio::File>>,
        pub(super) copy_status: gtk::Label,

        pub(super) inhibit_cookie: RefCell<Option<u32>>,
        pub(super) recording: RefCell<Option<(Recording, Vec<glib::SignalHandlerId>)>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "KoohaWindow";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            ToggleButton::ensure_type();

            klass.bind_template();

            klass.install_action_async("win.toggle-record", None, |obj, _, _| async move {
                obj.toggle_record().await;
            });

            klass.install_action("win.toggle-pause", None, move |obj, _, _| {
                if let Err(err) = obj.toggle_pause() {
                    let err = err.context(gettext("Failed to toggle pause"));
                    tracing::error!("{:?}", err);
                    obj.present_recording_error_dialog(&err);
                }
            });

            klass.install_action("win.cancel-record", None, move |obj, _, _| {
                obj.cancel_record();
            });

            klass.install_action("win.forget-video-sources", None, move |_obj, _, _| {
                Application::get()
                    .settings()
                    .reset_screencast_restore_token();
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn constructed(&self) {
            self.parent_constructed();

            let obj = self.obj();

            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            obj.setup_settings();
            obj.setup_quick_controls();
            let settings = Application::get().settings().clone();
            settings.connect_record_camera_changed(clone!(
                #[weak]
                obj,
                move |_| {
                    obj.refresh_bubble();
                }
            ));
            settings.connect_camera_device_changed(clone!(
                #[weak]
                obj,
                move |_| {
                    obj.refresh_bubble();
                }
            ));
            obj.refresh_bubble();
            obj.connect_is_active_notify(|window| {
                if window.is_active() {
                    window.copy_pending_file();
                }
            });

            obj.update_view();
            obj.update_title_label();
            obj.update_subtitle_label();
            obj.update_forget_video_sources_action();
        }
    }

    impl WidgetImpl for Window {
        fn map(&self) {
            self.parent_map();

            let obj = self.obj();

            obj.update_audio_actions();
        }
    }

    impl WindowImpl for Window {
        fn close_request(&self) -> glib::Propagation {
            let obj = self.obj();

            if obj.is_busy() {
                glib::spawn_future_local(clone!(
                    #[weak]
                    obj,
                    async move {
                        if obj.run_quit_confirmation_dialog().await.is_proceed() {
                            obj.destroy();
                        }
                    }
                ));
                return glib::Propagation::Stop;
            }

            self.parent_close_request()
        }
    }

    impl ApplicationWindowImpl for Window {}
    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gio::ActionMap, gtk::ShortcutManager, gio::ActionGroup, gtk::Native, gtk::Root;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    fn copy_pending_file(&self) {
        let Some(file) = self.imp().pending_copy.take() else {
            return;
        };
        // FileList enables GTK's portal file transfer for Flatpak destinations.
        // URI and GNOME file-manager formats also serve non-GTK recipients.
        let files = gtk::gdk::FileList::from_array(std::slice::from_ref(&file));
        let uri = format!("{}\r\n", file.uri());
        let copied = format!("copy\n{}", file.uri());
        let content = gtk::gdk::ContentProvider::new_union(&[
            gtk::gdk::ContentProvider::for_value(&files.to_value()),
            gtk::gdk::ContentProvider::for_bytes(
                "text/uri-list",
                &glib::Bytes::from_owned(uri.into_bytes()),
            ),
            gtk::gdk::ContentProvider::for_bytes(
                "x-special/gnome-copied-files",
                &glib::Bytes::from_owned(copied.into_bytes()),
            ),
        ]);
        match self.clipboard().set_content(Some(&content)) {
            Ok(()) => self
                .imp()
                .copy_status
                .set_label("Video copied — paste it as an attachment"),
            Err(err) => {
                tracing::warn!(?err, "Could not copy recording");
                self.imp()
                    .copy_status
                    .set_label("Saved. Click Copy video to try again.");
            }
        }
    }

    fn setup_quick_controls(&self) {
        let controls = &self.imp().quick_controls;
        let settings = Application::get().settings().clone();
        let cameras = crate::camera::devices();
        if cameras.is_empty() {
            settings.set_record_camera(false);
        }
        self.action_set_enabled("win.record-camera", !cameras.is_empty());
        let names: Vec<&str> = cameras.iter().map(|(name, _)| name.as_str()).collect();
        let camera = gtk::DropDown::from_strings(if names.is_empty() {
            &["No camera detected"]
        } else {
            &names
        });
        camera.set_sensitive(!names.is_empty());
        camera.set_tooltip_text(Some("Camera"));
        if let Some(index) = cameras
            .iter()
            .position(|(_, path)| path == &settings.camera_device())
        {
            camera.set_selected(index as u32);
        } else if settings.camera_device().is_empty()
            && let Some((_, path)) = cameras.first()
        {
            settings.set_camera_device(path);
        } else {
            camera.set_selected(gtk::INVALID_LIST_POSITION);
        }
        camera.connect_selected_notify(clone!(
            #[strong]
            settings,
            move |dropdown| {
                if let Some((_, path)) = cameras.get(dropdown.selected() as usize) {
                    settings.set_camera_device(path);
                }
            }
        ));
        controls.append(&camera);
        let corners =
            gtk::DropDown::from_strings(&["Top left", "Top right", "Bottom left", "Bottom right"]);
        corners.set_tooltip_text(Some("Webcam corner"));
        settings.bind_camera_corner(&corners, "selected").build();
        controls.append(&corners);
        let preview = gtk::Button::with_label("Show webcam bubble");
        preview.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                if let Err(err) = window.preview_camera() {
                    window.present_recording_error_dialog(&err);
                }
            }
        ));
        controls.append(&preview);
        let sizes = gtk::DropDown::from_strings(&["Small bubble", "Medium bubble", "Large bubble"]);
        sizes.set_selected(1);
        sizes.connect_selected_notify(clone!(
            #[weak(rename_to = obj)]
            self,
            move |sizes| {
                if let Some(bubble) = obj.imp().bubble.borrow().as_ref() {
                    let width = [160, 240, 320][sizes.selected().min(2) as usize];
                    bubble.window.set_default_size(width, width);
                }
            }
        ));
        controls.append(&sizes);
        let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let copy = gtk::Button::with_label("Copy video");
        copy.set_hexpand(true);
        copy.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                if let Some(file) = window.imp().last_file.borrow().clone() {
                    window.imp().pending_copy.replace(Some(file));
                    window.copy_pending_file();
                }
            }
        ));
        let open = gtk::Button::with_label("Open video");
        open.set_hexpand(true);
        open.connect_clicked(clone!(
            #[weak(rename_to = window)]
            self,
            move |_| {
                if let Some(file) = window.imp().last_file.borrow().clone() {
                    Application::get()
                        .activate_action("launch-uri", Some(&file.uri().to_variant()));
                }
            }
        ));
        buttons.append(&copy);
        buttons.append(&open);
        controls.append(&buttons);
        self.imp().copy_status.set_wrap(true);
        self.imp()
            .copy_status
            .set_label("Recordings are saved and copied automatically");
        controls.append(&self.imp().copy_status);
    }

    fn refresh_bubble(&self) {
        if self.is_busy() {
            return;
        }
        let previous = self.imp().bubble.take();
        drop(previous);
        let settings = Application::get().settings().clone();
        if !settings.record_camera() {
            return;
        }
        match crate::bubble::Bubble::new(&settings.camera_device(), settings.camera_corner()) {
            Ok(bubble) => {
                bubble.window.connect_close_request(clone!(
                    #[weak(rename_to = obj)]
                    self,
                    #[upgrade_or]
                    glib::Propagation::Proceed,
                    move |_| {
                        if !obj.is_busy() {
                            Application::get().settings().set_record_camera(false);
                        }
                        glib::Propagation::Stop
                    }
                ));
                self.imp().bubble.replace(Some(bubble));
            }
            Err(error) => self.present_recording_error_dialog(&error),
        }
    }

    pub fn camera_feed(&self) -> Result<crate::bubble::CameraFeed> {
        self.imp()
            .bubble
            .borrow()
            .as_ref()
            .map(|b| b.feed.clone())
            .ok_or_else(|| {
                anyhow::anyhow!("Enable the webcam and select a working camera before recording")
            })
    }

    pub fn place_bubble(&self, x: f64, y: f64, width: f64) {
        if let Some(bubble) = self.imp().bubble.borrow().as_ref() {
            bubble.feed.place(x, y, width);
        }
    }

    fn preview_camera(&self) -> Result<()> {
        let settings = Application::get().settings().clone();
        settings.set_record_camera(true);
        let bubble = self.imp().bubble.borrow();
        let bubble = bubble
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("The camera could not be opened"))?;
        bubble.window.present();
        Ok(())
    }

    /// Returns `true` if the window is busy with a recording.
    pub fn is_busy(&self) -> bool {
        self.imp()
            .recording
            .borrow()
            .as_ref()
            .is_some_and(|(recording, _)| {
                matches!(
                    recording.state(),
                    RecordingState::Recording
                        | RecordingState::Paused
                        | RecordingState::Flushing { .. }
                )
            })
    }

    /// Returns `Proceed` if the user wants to proceed with the quit operation.
    pub async fn run_quit_confirmation_dialog(&self) -> glib::Propagation {
        const CANCEL_RESPONSE_ID: &str = "cancel";
        const QUIT_RESPONSE_ID: &str = "quit";

        debug_assert!(
            self.is_busy(),
            "quit confirmation dialog must only be presented when busy"
        );

        let body_text = match self
            .imp()
            .recording
            .borrow()
            .as_ref()
            .map(|(recording, _)| recording.state())
        {
            Some(RecordingState::Recording) | Some(RecordingState::Paused) => gettext(
                "A recording is currently in progress. Quitting immediately may cause the recording to be unplayable. Please stop the recording before quitting.",
            ),
            Some(RecordingState::Flushing { .. }) => gettext(
                "Quitting will cancel the processing and may cause the recording to be unplayable.",
            ),
            state => unreachable!("unexpected recording state: {:?}", state),
        };

        let dialog = adw::AlertDialog::builder()
            .heading(gettext("Quit the Application?"))
            .body(body_text)
            .close_response(CANCEL_RESPONSE_ID)
            .default_response(CANCEL_RESPONSE_ID)
            .build();
        dialog.add_response(CANCEL_RESPONSE_ID, &gettext("Cancel"));

        dialog.add_response(QUIT_RESPONSE_ID, &gettext("Quit"));
        dialog.set_response_appearance(QUIT_RESPONSE_ID, adw::ResponseAppearance::Destructive);

        match dialog.choose_future(Some(self)).await.as_str() {
            CANCEL_RESPONSE_ID => glib::Propagation::Stop,
            QUIT_RESPONSE_ID => glib::Propagation::Proceed,
            _ => unreachable!(),
        }
    }

    fn present_recording_error_dialog(&self, err: &Error) {
        const OK_RESPONSE_ID: &str = "ok";

        let err_text = format!("{:?}", err);

        let err_view = gtk::TextView::builder()
            .buffer(&gtk::TextBuffer::builder().text(&err_text).build())
            .editable(false)
            .monospace(true)
            .top_margin(6)
            .bottom_margin(6)
            .left_margin(6)
            .right_margin(6)
            .build();

        let scrolled_window = gtk::ScrolledWindow::builder()
            .min_content_width(360)
            .min_content_height(120)
            .child(&err_view)
            .build();

        let scrolled_window_row = gtk::ListBoxRow::builder()
            .overflow(gtk::Overflow::Hidden)
            .activatable(false)
            .selectable(false)
            .child(&scrolled_window)
            .build();
        scrolled_window_row.add_css_class("error-view");

        let copy_button = gtk::Button::builder()
            .valign(gtk::Align::Center)
            .tooltip_text(gettext("Copy to clipboard"))
            .icon_name("edit-copy-symbolic")
            .build();
        copy_button.connect_clicked(move |button| {
            button.clipboard().set_text(&err_text);
            button.set_tooltip_text(Some(&gettext("Copied to clipboard")));
            button.set_icon_name("checkmark-symbolic");
            button.add_css_class("copy-done");
        });

        let expander = adw::ExpanderRow::builder()
            .title(gettext("Show detailed error"))
            .activatable(false)
            .build();
        expander.add_row(&scrolled_window_row);
        expander.add_suffix(&copy_button);

        let list_box = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .build();
        list_box.add_css_class("boxed-list");
        list_box.append(&expander);

        let dialog = adw::AlertDialog::builder()
            .heading(err.to_string())
            .body_use_markup(true)
            .default_response(OK_RESPONSE_ID)
            .extra_child(&list_box)
            .build();

        if let Some(context) = err.downcast_ref::<ContextWithHelp>() {
            dialog.set_body(&format!(
                "<b>{}</b>: {}",
                gettext("Help"),
                context.help_message()
            ));
        }

        dialog.add_response(OK_RESPONSE_ID, &gettext("Ok, Got It"));
        dialog.present(Some(self));
    }

    fn present_no_profile_error_dialog(&self) {
        const OPEN_RESPONSE_ID: &str = "open";
        const LATER_RESPONSE_ID: &str = "later";

        let dialog = adw::AlertDialog::builder()
            .heading(gettext("Open Preferences?"))
            .body(gettext("The previously selected format may have been unavailable. Open preferences and select a format to continue recording."))
            .default_response(OPEN_RESPONSE_ID)
            .build();

        dialog.add_response(LATER_RESPONSE_ID, &gettext("Later"));

        dialog.add_response(OPEN_RESPONSE_ID, &gettext("Open"));
        dialog.set_response_appearance(OPEN_RESPONSE_ID, adw::ResponseAppearance::Suggested);

        dialog.connect_response(
            Some(OPEN_RESPONSE_ID),
            clone!(
                #[weak(rename_to = obj)]
                self,
                move |dialog, _| {
                    dialog.close();

                    let app = Application::get();
                    let preferences_dialog = PreferencesDialog::new(app.settings());
                    preferences_dialog.present(Some(&obj));

                    let was_focused = preferences_dialog.profile_row_grab_focus();
                    debug_assert!(was_focused);
                }
            ),
        );

        dialog.present(Some(self));
    }

    async fn toggle_record(&self) {
        let imp = self.imp();

        if let Some((ref recording, _)) = *imp.recording.borrow() {
            recording.stop();
            return;
        }

        let recording = Recording::new();
        let handler_ids = vec![
            recording.connect_state_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |_| {
                    obj.update_view();
                    obj.update_inhibit();
                }
            )),
            recording.connect_duration_notify(clone!(
                #[weak(rename_to = obj)]
                self,
                move |recording| {
                    let formatted_time = format::digital_clock(recording.duration());
                    obj.imp().recording_time_label.set_label(&formatted_time);
                }
            )),
            recording.connect_finished(clone!(
                #[weak(rename_to = obj)]
                self,
                move |recording, res| {
                    obj.handle_recording_finished(recording, res);
                }
            )),
        ];
        imp.recording
            .replace(Some((recording.clone(), handler_ids)));

        self.update_inhibit();

        recording
            .start(Some(self), Application::get().settings())
            .await;
    }

    fn toggle_pause(&self) -> Result<()> {
        let imp = self.imp();

        if let Some((ref recording, _)) = *imp.recording.borrow() {
            if matches!(recording.state(), RecordingState::Paused) {
                recording.resume()?;
            } else {
                recording.pause()?;
            };
        }

        Ok(())
    }

    fn cancel_record(&self) {
        let imp = self.imp();

        if let Some((ref recording, _)) = *imp.recording.borrow() {
            recording.cancel();
        }
    }

    fn handle_recording_finished(
        &self,
        recording: &Recording,
        res: &Result<(gio::File, gst::ClockTime)>,
    ) {
        debug_assert_eq!(recording.state(), RecordingState::Finished);

        match res {
            Ok((recording_file, duration)) => {
                self.imp().last_file.replace(Some(recording_file.clone()));
                self.imp()
                    .pending_copy
                    .replace(Some(recording_file.clone()));
                self.imp()
                    .copy_status
                    .set_label("Saved. Preparing clipboard…");
                self.present();
                if self.is_active() {
                    self.copy_pending_file();
                }
                let duration = *duration;
                glib::spawn_future_local(clone!(
                    #[strong]
                    recording_file,
                    async move {
                        let app = Application::get();
                        app.send_record_success_notification(&recording_file, duration)
                            .await;
                    }
                ));

                let recent_manager = gtk::RecentManager::default();
                recent_manager.add_item(&recording_file.uri());
            }
            Err(err) => {
                if err.is::<Cancelled>() {
                    tracing::debug!("{:?}", err);
                } else if err.is::<NoProfileError>() {
                    self.present_no_profile_error_dialog();
                } else {
                    tracing::error!("{:?}", err);

                    self.present_recording_error_dialog(err);

                    if let Some(surface) = self.surface() {
                        surface.beep();
                    }
                }
            }
        }

        if let Some((recording, handler_ids)) = self.imp().recording.take() {
            for handler_id in handler_ids {
                recording.disconnect(handler_id);
            }

            self.update_inhibit();
        } else {
            tracing::warn!("Recording finished but no stored recording");
        }
    }

    fn update_inhibit(&self) {
        let imp = self.imp();

        let app = Application::get();
        let is_busy = self.is_busy();

        if is_busy && imp.inhibit_cookie.borrow().is_none() {
            let inhibit_cookie = app.inhibit(
                Some(self),
                gtk::ApplicationInhibitFlags::LOGOUT | gtk::ApplicationInhibitFlags::IDLE,
                Some(&gettext("A recording is in progress")),
            );
            imp.inhibit_cookie.replace(Some(inhibit_cookie));

            tracing::debug!("Inhibited logout and idle");
        } else if !is_busy && let Some(inhibit_cookie) = imp.inhibit_cookie.take() {
            app.uninhibit(inhibit_cookie);

            tracing::debug!("Uninhibited logout and idle");
        }
    }

    fn update_view(&self) {
        let imp = self.imp();

        // TODO disregard ms granularity recording state change

        let state = imp
            .recording
            .borrow()
            .as_ref()
            .map_or(RecordingState::Init, |(recording, _)| recording.state());

        match state {
            RecordingState::Init | RecordingState::Finished => {
                imp.stack.set_visible_child(&*imp.main_page);

                imp.recording_time_label
                    .set_label(&format::digital_clock(gst::ClockTime::ZERO));
            }
            RecordingState::Delayed { secs_left } => {
                imp.delay_label.set_label(&secs_left.to_string());

                imp.stack.set_visible_child(&*imp.delay_page);
            }
            RecordingState::Recording => {
                imp.pause_record_button
                    .set_icon_name("media-playback-pause-symbolic");
                imp.recording_label.set_label(&gettext("Recording"));
                imp.recording_time_label.remove_css_class("paused");

                imp.stack.set_visible_child(&*imp.recording_page);
            }
            RecordingState::Paused => {
                imp.pause_record_button
                    .set_icon_name("media-playback-start-symbolic");
                imp.recording_label.set_label(&gettext("Paused"));
                imp.recording_time_label.add_css_class("paused");

                imp.stack.set_visible_child(&*imp.recording_page);
            }
            RecordingState::Flushing { progress } => {
                imp.flushing_progress_icon
                    .set_progress(progress as f64 / 100.0);

                imp.stack.set_visible_child(&*imp.flushing_page);
            }
        }

        self.action_set_enabled(
            "win.toggle-record",
            !matches!(
                state,
                RecordingState::Delayed { .. } | RecordingState::Flushing { .. }
            ),
        );
        self.action_set_enabled(
            "win.toggle-pause",
            matches!(state, RecordingState::Recording | RecordingState::Paused),
        );
        self.action_set_enabled(
            "win.cancel-record",
            matches!(
                state,
                RecordingState::Delayed { .. } | RecordingState::Flushing { .. }
            ),
        );
    }

    fn update_title_label(&self) {
        let imp = self.imp();

        match Application::get().settings().capture_mode() {
            CaptureMode::MonitorWindow => imp.title.set_title(&gettext("Quickcast")),
            CaptureMode::Selection => imp.title.set_title(&gettext("Quickcast · Region")),
        }
    }

    fn update_subtitle_label(&self) {
        let imp = self.imp();

        let app = Application::get();
        let settings = app.settings();

        let profile_text = settings
            .profile()
            .map_or_else(|| gettext("None"), |profile| profile.name().to_string());
        let framerate_text = format::framerate(settings.framerate());

        imp.title
            .set_subtitle(&format!("{} • {} FPS", profile_text, framerate_text));
    }

    fn update_audio_actions(&self) {
        let is_enabled = Application::get()
            .settings()
            .profile()
            .is_none_or(|profile| profile.supports_audio());

        self.action_set_enabled("win.record-desktop-audio", is_enabled);
        self.action_set_enabled("win.record-microphone", is_enabled);
    }

    fn update_forget_video_sources_action(&self) {
        let has_restore_token = !Application::get()
            .settings()
            .screencast_restore_token()
            .is_empty();

        self.imp()
            .forget_video_sources_revealer
            .set_reveal_child(has_restore_token);

        self.action_set_enabled("win.forget-video-sources", has_restore_token);
    }

    fn setup_settings(&self) {
        let app = Application::get();
        let settings = app.settings();

        settings.connect_capture_mode_changed(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.update_title_label();
            }
        ));

        settings.connect_profile_changed(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.update_audio_actions();
                obj.update_subtitle_label();
            }
        ));

        settings.connect_framerate_changed(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.update_subtitle_label();
            }
        ));

        settings.connect_screencast_restore_token_changed(clone!(
            #[weak(rename_to = obj)]
            self,
            move |_| {
                obj.update_forget_video_sources_action();
            }
        ));

        self.add_action(&settings.create_record_desktop_audio_action());
        self.add_action(&settings.create_record_microphone_action());
        self.add_action(&settings.create_record_camera_action());
        self.add_action(&settings.create_show_pointer_action());
        self.add_action(&settings.create_capture_mode_action());
    }
}

#[cfg(test)]
mod quickcast_tests {
    use super::*;

    #[gtk::test]
    fn window_copies_file_formats_and_renders() {
        gst::init().unwrap();
        let resource = gio::Resource::load(crate::config::RESOURCES_FILE).unwrap();
        gio::resources_register(&resource);
        let app = Application::new();
        app.register(None::<&gio::Cancellable>).unwrap();
        app.activate();
        let window = app.window();
        let paintable = gtk::WidgetPaintable::new(Some(&window));
        window.present();
        let context = glib::MainContext::default();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        while std::time::Instant::now() < deadline {
            while context.pending() {
                context.iteration(false);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let file = gio::File::for_path("/tmp/Quickcast test clip.mp4");
        window.imp().pending_copy.replace(Some(file.clone()));
        window.copy_pending_file();
        let formats = window.clipboard().formats();
        assert!(formats.contains_type(gtk::gdk::FileList::static_type()));
        assert!(formats.contain_mime_type("text/uri-list"));
        assert!(formats.contain_mime_type("x-special/gnome-copied-files"));
        assert!(window.imp().copy_status.label().contains("Video copied"));
        for format in ["text/uri-list", "x-special/gnome-copied-files", "files"] {
            let mut child = std::process::Command::new("python3")
                .arg(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/scripts/read-clipboard.py"
                ))
                .args([format, file.uri().as_str()])
                .spawn()
                .unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(8);
            let status = loop {
                while context.pending() {
                    context.iteration(false);
                }
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    panic!("Clipboard consumer did not exit");
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            };
            assert!(
                status.success(),
                "Clipboard format {format} failed in another process"
            );
        }
        // Exercise the floating window and the shared camera feed without a
        // physical camera. A second receiver must not open another camera.
        use gst::prelude::*;
        gstgtk4::plugin_register_static().unwrap();
        let source = gst::parse::bin_from_description(
            "videotestsrc is-live=true pattern=red ! video/x-raw,width=320,height=240,framerate=30/1 ! identity", true).unwrap();
        let circular = crate::camera::circular_filter().unwrap();
        let camera = gst::Bin::new();
        camera.add_many([&source, &circular]).unwrap();
        source.link(&circular).unwrap();
        camera
            .add_pad(&gst::GhostPad::with_target(&circular.static_pad("src").unwrap()).unwrap())
            .unwrap();
        let bubble = crate::bubble::Bubble::from_source(&camera, 3).unwrap();
        let bubble_paintable = gtk::WidgetPaintable::new(Some(&bubble.window));
        assert_eq!(app.window(), window);
        let capture = gst::Pipeline::new();
        let screen = gst::parse::bin_from_description(
            "videotestsrc is-live=true pattern=blue ! video/x-raw,width=640,height=360,framerate=30/1 ! identity", true).unwrap();
        let output = gst::ElementFactory::make("fakesink")
            .property("signal-handoffs", true)
            .build()
            .unwrap();
        let frames = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        output.connect("handoff", false, {
            let frames = frames.clone();
            move |_| {
                frames.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                None
            }
        });
        capture.add_many([screen.upcast_ref(), &output]).unwrap();
        crate::camera::attach(
            &capture,
            screen.upcast_ref(),
            &output,
            Some((&bubble.feed, 3)),
        )
        .unwrap();
        bubble.feed.place(0.25, 0.75, 0.2);
        capture.set_state(gst::State::Playing).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while std::time::Instant::now() < deadline {
            while context.pending() {
                context.iteration(false);
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let pad = capture
            .by_name("quickcast-compositor")
            .unwrap()
            .static_pad("sink_1")
            .unwrap();
        assert_eq!(pad.property::<i32>("xpos"), 128);
        assert_eq!(pad.property::<i32>("ypos"), 174);
        assert!(frames.load(std::sync::atomic::Ordering::Relaxed) >= 5);
        capture.set_state(gst::State::Null).unwrap();
        let snapshot = gtk::Snapshot::new();
        let width = bubble.window.width();
        let height = bubble.window.height();
        assert_eq!(width, height, "Bubble must remain square");
        bubble_paintable.snapshot(&snapshot, f64::from(width), f64::from(height));
        let node = snapshot.to_node().expect("Bubble rendered no content");
        let texture = bubble.window.renderer().unwrap().render_texture(
            &node,
            Some(&gtk::graphene::Rect::new(
                0.0,
                0.0,
                width as f32,
                height as f32,
            )),
        );
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        texture.download(&mut pixels, (width * 4) as usize);
        assert_eq!(pixels[3], 0, "Bubble corner must be transparent");
        assert_eq!(
            pixels[((height / 2 * width + width / 2) * 4 + 3) as usize],
            255,
            "Bubble center must show the camera"
        );
        if let Ok(directory) = std::env::var("QUICKCAST_TEST_DIR") {
            texture
                .save_to_png(std::path::Path::new(&directory).join("bubble.png"))
                .unwrap();
        }
        drop(bubble);
        if let Ok(directory) = std::env::var("QUICKCAST_TEST_DIR") {
            let snapshot = gtk::Snapshot::new();
            paintable.snapshot(
                &snapshot,
                f64::from(window.width()),
                f64::from(window.height()),
            );
            let node = snapshot.to_node().expect("Window rendered no content");
            let renderer = window.renderer().expect("Window has no renderer");
            let texture = renderer.render_texture(&node, None);
            texture
                .save_to_png(std::path::Path::new(&directory).join("window.png"))
                .unwrap();
        }
        window.destroy();
    }
}
