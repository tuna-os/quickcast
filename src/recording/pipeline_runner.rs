use std::{os::unix::io::RawFd, path::PathBuf};

use anyhow::Result;

use crate::{
    application::Application,
    area_selector::AreaSelector,
    pipeline::PipelineBuilder,
    profile::Profile,
    screencast_portal::{SourceType, Stream},
    settings::{CaptureMode, Settings},
};

pub(super) async fn build(
    file_path: PathBuf,
    settings: &Settings,
    profile: &Profile,
    raw_fd: RawFd,
    streams: &[Stream],
) -> Result<gst::Pipeline> {
    let mut pipeline_builder = PipelineBuilder::new(
        file_path,
        settings.framerate(),
        profile.clone(),
        raw_fd,
        streams.to_vec(),
    );

    if settings.capture_mode() == CaptureMode::Selection {
        let data = AreaSelector::select(raw_fd, streams, &Application::get().window()).await?;
        pipeline_builder.select_area_data(data);
    }

    if profile.supports_audio() {
        pipeline_builder.record_desktop_audio(settings.record_desktop_audio());
        pipeline_builder.record_microphone(settings.record_microphone());
    }

    // Monitor/region streams already include the floating webcam window.
    // Compositing it again would duplicate the bubble. Window-only streams
    // exclude other windows, so feed the same camera frames to the encoder.
    if settings.record_camera()
        && streams
            .iter()
            .any(|stream| stream.source_type() == Some(SourceType::WINDOW))
    {
        pipeline_builder.camera(
            Application::get().window().camera_feed()?,
            settings.camera_corner(),
        );
    }

    pipeline_builder.build()
}
