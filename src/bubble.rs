//! One camera capture feeds both the floating preview and window-only recordings.
use anyhow::{Context, Result};
use gst::prelude::*;
use gtk::{glib, prelude::*};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct CameraFeed {
    receiver: Arc<Mutex<glib::WeakRef<gst_app::AppSrc>>>,
    pub placement: Arc<Mutex<(f64, f64, f64)>>,
}

impl CameraFeed {
    pub fn source(&self) -> Result<gst::Bin> {
        let source = gst::ElementFactory::make("appsrc")
            .property("is-live", true)
            .property("do-timestamp", true)
            .property("format", gst::Format::Time)
            .property("max-buffers", 2u64)
            .property_from_str("leaky-type", "downstream")
            .build()?
            .downcast::<gst_app::AppSrc>()
            .unwrap();
        *self.receiver.lock().unwrap() = source.downgrade();
        let bin = gst::Bin::new();
        bin.add(&source)?;
        bin.add_pad(&gst::GhostPad::with_target(
            &source.static_pad("src").unwrap(),
        )?)?;
        Ok(bin)
    }

    pub fn place(&self, x: f64, y: f64, width: f64) {
        if x.is_finite() && y.is_finite() && width.is_finite() {
            *self.placement.lock().unwrap() =
                (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0), width.clamp(0.05, 0.8));
        }
    }
}

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
        let feed = CameraFeed {
            receiver: Arc::new(Mutex::new(glib::WeakRef::new())),
            placement: Arc::new(Mutex::new((
                f64::from(corner % 2),
                f64::from(corner / 2),
                0.2,
            ))),
        };
        samples.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample({
                    let feed = feed.clone();
                    move |sink| {
                        let sample = sink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                        if let Some(receiver) = feed.receiver.lock().unwrap().upgrade()
                            && let Some(buffer) = sample.buffer()
                        {
                            receiver.set_caps(sample.caps().map(ToOwned::to_owned).as_ref());
                            let mut frame = buffer.copy();
                            // The recording has a different running-time origin from the
                            // camera preview. Timestamp on appsrc's recording clock.
                            let frame_mut = frame.make_mut();
                            frame_mut.set_pts(None);
                            frame_mut.set_dts(None);
                            let _ = receiver.push_buffer(frame);
                        }
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
            .title("Quickcast Webcam")
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
