//! The floating webcam window. Its camera capture feeds the preview and a
//! [`CameraFeed`] that window-only recordings read.
use anyhow::{Context, Result};
use gst::prelude::*;
use gtk::{glib, prelude::*};

use crate::camera::CameraFeed;

/// The Shell extension finds the bubble by this exact title, so it is an IPC
/// value, not UI text: do not pass it through gettext.
pub const WINDOW_TITLE: &str = "Quickcast Webcam";

#[derive(Debug)]
pub struct Bubble {
    pub window: gtk::Window,
    pub feed: CameraFeed,
    pipeline: gst::Pipeline,
    _watch: gst::bus::BusWatchGuard,
}

impl Bubble {
    pub fn new(device: &str, corner: u32) -> Result<Self> {
        Self::from_source(&crate::camera::source(device)?, corner)
    }

    pub(crate) fn from_source(camera: &gst::Bin, corner: u32) -> Result<Self> {
        let pipeline = gst::Pipeline::new();
        let tee = gst::ElementFactory::make("tee").build()?;
        let preview_queue = gst::ElementFactory::make("queue").build()?;
        let preview = gst::ElementFactory::make("gtk4paintablesink").build()?;
        let record_queue = gst::ElementFactory::make("queue")
            .property("max-size-buffers", 2u32)
            .property_from_str("leaky", "downstream")
            .build()?;
        let samples = gst::ElementFactory::make("appsink")
            .property("sync", false)
            .property("max-buffers", 2u32)
            .property("drop", true)
            .build()?
            .downcast::<gst_app::AppSink>()
            .unwrap();
        let feed = CameraFeed::new(corner);
        samples.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample({
                    let feed = feed.clone();
                    move |sink| {
                        let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                        feed.push(&sample);
                        Ok(gst::FlowSuccess::Ok)
                    }
                })
                .build(),
        );
        pipeline.add_many([
            camera.upcast_ref(),
            &tee,
            &preview_queue,
            &preview,
            &record_queue,
            samples.upcast_ref(),
        ])?;
        camera.link(&tee)?;
        gst::Element::link_many([&tee, &preview_queue, &preview])?;
        gst::Element::link_many([&tee, &record_queue, samples.upcast_ref()])?;
        let picture =
            gtk::Picture::for_paintable(&preview.property::<gtk::gdk::Paintable>("paintable"));
        picture.set_content_fit(gtk::ContentFit::Cover);
        picture.set_can_shrink(true);
        let handle = gtk::WindowHandle::new();
        handle.set_child(Some(&picture));
        let window = gtk::Window::builder()
            .application(&crate::Application::get())
            .title(WINDOW_TITLE)
            .decorated(false)
            .default_width(240)
            .default_height(240)
            .child(&handle)
            .build();
        window.add_css_class("quickcast-bubble");
        let status_window = window.downgrade();
        let watch = pipeline
            .bus()
            .context("Camera has no bus")?
            .add_watch_local(move |_, message| {
                if let gst::MessageView::Error(error) = message.view() {
                    tracing::error!(error = %error.error(), "Camera preview failed");
                    if let Some(window) = status_window.upgrade() {
                        let message = gtk::Label::new(Some(&format!(
                            "Camera unavailable: {}",
                            error.error()
                        )));
                        message.set_wrap(true);
                        window.set_child(Some(&message));
                    }
                }
                glib::ControlFlow::Continue
            })?;
        if let Err(error) = pipeline.set_state(gst::State::Playing) {
            let _ = pipeline.set_state(gst::State::Null);
            return Err(error.into());
        }
        window.present();
        Ok(Self {
            window,
            feed,
            pipeline,
            _watch: watch,
        })
    }
}

impl Drop for Bubble {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
        self.window.destroy();
    }
}
