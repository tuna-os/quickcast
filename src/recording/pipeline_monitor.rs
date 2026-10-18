use std::time::Duration;

use anyhow::Error;
use gettextrs::gettext;
use gst::prelude::*;
use gtk::{gio::prelude::FileExt, glib::{self, clone}, subclass::prelude::*};

use super::{Recording, RecordingState};
use crate::{help::ContextWithHelp, i18n::gettext_f};

const DURATION_UPDATE_INTERVAL: Duration = Duration::from_millis(200);

pub(super) fn start(recording: &Recording, pipeline: &gst::Pipeline) {
    let bus_watch_guard = pipeline
        .bus()
        .unwrap()
        .add_watch_local(clone!(
            #[weak(rename_to = obj)]
            recording,
            #[upgrade_or_panic]
            move |_, message| handle_bus_message(&obj, message)
        ))
        .unwrap();
    recording
        .imp()
        .bus_watch_guard
        .replace(Some(bus_watch_guard));
    recording
        .imp()
        .duration_source_id
        .replace(Some(glib::timeout_add_local(
            DURATION_UPDATE_INTERVAL,
            clone!(
                #[weak(rename_to = obj)]
                recording,
                #[upgrade_or_panic]
                move || {
                    update_duration(&obj);
                    update_flushing_progress(&obj);
                    glib::ControlFlow::Continue
                }
            ),
        )));
}

pub(super) fn update_duration(recording: &Recording) {
    let imp = recording.imp();

    let clock_time = imp
        .pipeline
        .get()
        .and_then(|pipeline| pipeline.query_position::<gst::ClockTime>())
        .unwrap_or(gst::ClockTime::ZERO);

    if clock_time == recording.duration() {
        return;
    }

    imp.duration.set(clock_time);
    recording.notify_duration();
}

pub(super) fn update_flushing_progress(recording: &Recording) {
    let imp = recording.imp();

    let RecordingState::Flushing {
        progress: prev_progress,
    } = recording.state()
    else {
        return;
    };

    let progress = imp
        .estimated_final_duration
        .get()
        .map_or(0, |estimated_final_duration| {
            let progress_percent = recording.duration().nseconds() as f64
                / estimated_final_duration.nseconds() as f64
                * 100.0;
            (progress_percent.round() as u8).clamp(0, 100)
        });

    if prev_progress == progress {
        return;
    }

    recording.set_state(RecordingState::Flushing { progress });
}

fn handle_bus_message(recording: &Recording, message: &gst::Message) -> glib::ControlFlow {
    use gst::MessageView;

    let imp = recording.imp();

    match message.view() {
        MessageView::Error(e) => {
            tracing::debug!(state = ?recording.state(), "Received error at bus");

            if let Err(err) = recording.pipeline().set_state(gst::State::Null) {
                tracing::warn!("Failed to stop pipeline on error: {:?}", err);
            }

            recording.close_session();

            if let Some(source_id) = imp.duration_source_id.take() {
                source_id.remove();
            }

            // TODO print error quarks for all glib::Error

            let error = Error::from(e.error())
                .context(e.debug().unwrap_or_else(|| "<no debug>".into()))
                .context(gettext("An error occurred while recording"));

            let error = if e.error().matches(gst::ResourceError::OpenWrite) {
                error.context(ContextWithHelp::new(
                    gettext_f(
                        // Translators: Do NOT translate the contents between '{' and '}', this is a variable name.
                        "Failed to open “{path}” for writing",
                        &[("path", &recording.file().uri())],
                    ),
                    gettext("Make sure that the saving location exists and is accessible."),
                ))
            } else {
                error
            };

            recording.set_finished(Err(error));

            glib::ControlFlow::Break
        }
        MessageView::Eos(..) => {
            tracing::debug!("Eos signal received from record bus");

            debug_assert!(
                matches!(recording.state(), RecordingState::Flushing { .. }),
                "received eos signal on {:?}",
                recording.state()
            );

            recording.set_state(RecordingState::Flushing { progress: 100 });

            update_duration(recording);
            let duration = recording.duration();

            if let Err(err) = recording.pipeline().set_state(gst::State::Null) {
                tracing::error!("Failed to stop pipeline on eos: {:?}", err);
            }

            recording.close_session();

            if let Some(source_id) = imp.duration_source_id.take() {
                source_id.remove();
            }

            recording.set_finished(Ok((recording.file().clone(), duration)));

            glib::ControlFlow::Break
        }
        MessageView::StateChanged(sc) => {
            let new_state = sc.current();

            if message.src()
                != imp
                    .pipeline
                    .get()
                    .map(|pipeline| pipeline.upcast_ref::<gst::Object>())
            {
                tracing::trace!(
                    "`{}` changed state from `{:?}` -> `{:?}`",
                    message
                        .src()
                        .map_or_else(|| "<unknown source>".into(), |element| element.name()),
                    sc.old(),
                    new_state,
                );
                return glib::ControlFlow::Continue;
            }

            tracing::debug!(
                "Pipeline changed state from `{:?}` -> `{:?}`",
                sc.old(),
                new_state,
            );

            let state = match new_state {
                gst::State::Paused => RecordingState::Paused,
                gst::State::Playing => RecordingState::Recording,
                _ => return glib::ControlFlow::Continue,
            };
            recording.set_state(state);

            glib::ControlFlow::Continue
        }
        MessageView::AsyncDone(ad) => {
            tracing::trace!("Received async-done message on bus: {:?}", ad);

            // This is enabled by setting `GST_DEBUG_DUMP_DOT_DIR` to a directory (e.g. `GST_DEBUG_DUMP_DOT_DIR=.`).
            recording.pipeline().debug_to_dot_file_with_ts(
                gst::DebugGraphDetails::VERBOSE,
                "kooha-recording-pipeline",
            );

            glib::ControlFlow::Continue
        }
        MessageView::Warning(w) => {
            tracing::warn!("Received warning message on bus: {:?}", w);
            glib::ControlFlow::Continue
        }
        MessageView::Info(i) => {
            tracing::debug!("Received info message on bus: {:?}", i);
            glib::ControlFlow::Continue
        }
        MessageView::Qos(q) => {
            tracing::warn!("Received qos message on bus: {:?}", q);
            glib::ControlFlow::Continue
        }
        other => {
            tracing::trace!("Received other message on bus: {:?}", other);
            glib::ControlFlow::Continue
        }
    }
}
