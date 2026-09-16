use adw::prelude::*;
use anyhow::Error;
use gettextrs::gettext;
use gtk::glib::{self, clone};

use super::Window;
use crate::{
    Application,
    help::ContextWithHelp,
    preferences_dialog::PreferencesDialog,
    recording::RecordingState,
};

impl Window {
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

    pub(super) fn present_recording_error_dialog(&self, err: &Error) {
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

    pub(super) fn present_no_profile_error_dialog(&self) {
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
}
