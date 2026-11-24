//! Capture infrastructure shared by the recording pipeline and the area
//! selector preview: the `PipeWire` video source and the crop applied to a
//! captured stream. Nothing here depends on a widget, so the selector and the
//! recording pipeline both depend on this module instead of on each other.

use std::os::unix::io::RawFd;

use anyhow::{Context, Result, bail};
use gst::prelude::*;
use gtk::graphene::Rect;

use crate::screencast_portal::Stream;

/// Pixels removed from each edge of a captured stream.
///
/// Every value is even, because x264enc requires an even resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureCrop {
    /// Size of the uncropped stream, in pixels.
    pub stream_size: (i32, i32),
    pub top: i32,
    pub left: i32,
    pub right: i32,
    pub bottom: i32,
}

impl CaptureCrop {
    /// Computes the crop for `selection`, a rectangle in the same widget
    /// coordinates as `paintable_rect`, where the stream of `stream_size`
    /// pixels is drawn.
    pub fn from_selection(
        selection: &Rect,
        paintable_rect: &Rect,
        stream_size: (i32, i32),
    ) -> Self {
        let (stream_width, stream_height) = stream_size;
        let scale_factor_h = stream_width as f32 / paintable_rect.width();
        let scale_factor_v = stream_height as f32 / paintable_rect.height();

        if scale_factor_h != scale_factor_v {
            tracing::warn!(
                scale_factor_h,
                scale_factor_v,
                "Scale factors of horizontal and vertical are unequal"
            );
        }

        // Both paintable and selection position are relative to the widget coordinates.
        // To get the absolute position and so correct crop values, subtract the paintable
        // rect's position from the selection rect.
        let selection_rect_scaled = Rect::new(
            selection.x() - paintable_rect.x(),
            selection.y() - paintable_rect.y(),
            selection.width(),
            selection.height(),
        )
        .scale(scale_factor_h, scale_factor_v);

        let raw_top_crop = selection_rect_scaled.y();
        let raw_left_crop = selection_rect_scaled.x();
        let raw_right_crop =
            stream_width as f32 - (selection_rect_scaled.width() + selection_rect_scaled.x());
        let raw_bottom_crop =
            stream_height as f32 - (selection_rect_scaled.height() + selection_rect_scaled.y());

        tracing::debug!(raw_top_crop, raw_left_crop, raw_right_crop, raw_bottom_crop);

        let crop = Self {
            stream_size,
            top: round_to_even_f32(raw_top_crop).clamp(0, stream_height),
            left: round_to_even_f32(raw_left_crop).clamp(0, stream_width),
            right: round_to_even_f32(raw_right_crop).clamp(0, stream_width),
            bottom: round_to_even_f32(raw_bottom_crop).clamp(0, stream_height),
        };

        tracing::debug!(?crop);

        crop
    }
}

fn make_pipewiresrc(fd: RawFd, path: &str) -> Result<gst::Element> {
    let src = gst::ElementFactory::make("pipewiresrc")
        .property("fd", fd)
        .property("path", path)
        .property("do-timestamp", true)
        .property("provide-clock", false)
        .property("keepalive-time", 1000)
        .property("resend-last", true)
        .build()?;

    Ok(src)
}

fn make_videoflip() -> Result<gst::Element> {
    let videoflip = gst::ElementFactory::make("videoflip")
        .property_from_str("video-direction", "auto")
        .build()?;

    Ok(videoflip)
}

/// Creates a bin with a src pad for multiple pipewire streams.
///
/// Single stream:
///
/// pipewiresrc -> videoflip -> videorate
///
/// Multiple streams:
///
/// pipewiresrc1 -> videoflip -> |
///                              |
/// pipewiresrc2 -> videoflip -> | -> compositor -> videorate
///                              |
/// pipewiresrcn -> videoflip -> |
pub fn make_videosrc_bin(
    fd: RawFd,
    streams: &[Stream],
    framerate: gst::Fraction,
) -> Result<gst::Bin> {
    // TODO Create a bin that hotswaps compositor depending whether gl is supported or not.

    let bin = gst::Bin::builder().name("kooha-pipewiresrc-bin").build();

    let videorate = gst::ElementFactory::make("videorate")
        .property("skip-to-first", true)
        .build()?;
    let videorate_capsfilter = gst::ElementFactory::make("capsfilter")
        .property(
            "caps",
            gst::Caps::builder("video/x-raw")
                .field("framerate", framerate)
                .build(),
        )
        .build()?;
    bin.add_many([&videorate, &videorate_capsfilter])?;
    videorate.link(&videorate_capsfilter)?;

    match streams {
        [] => bail!("No streams provided"),
        [stream] => {
            let pipewiresrc = make_pipewiresrc(fd, &stream.node_id().to_string())?;
            let videoflip = make_videoflip()?;
            bin.add_many([&pipewiresrc, &videoflip])?;
            gst::Element::link_many([&pipewiresrc, &videoflip, &videorate])?;
        }
        streams => {
            let compositor = gst::ElementFactory::make("compositor").build()?;
            bin.add(&compositor)?;
            compositor.link(&videorate)?;

            let mut last_pos = 0;
            for stream in streams {
                let pipewiresrc = make_pipewiresrc(fd, &stream.node_id().to_string())?;
                let videoflip = make_videoflip()?;
                bin.add_many([&pipewiresrc, &videoflip])?;
                pipewiresrc.link(&videoflip)?;

                let compositor_sink_pad = compositor
                    .request_pad_simple("sink_%u")
                    .context("Failed to request sink_%u pad from compositor")?;
                compositor_sink_pad.set_property("xpos", last_pos);
                videoflip
                    .static_pad("src")
                    .unwrap()
                    .link(&compositor_sink_pad)?;

                let (stream_width, _) = stream.size().unwrap();
                last_pos += stream_width;
            }
        }
    }

    let src_pad = videorate_capsfilter.static_pad("src").unwrap();
    bin.add_pad(&gst::GhostPad::with_target(&src_pad)?)?;

    Ok(bin)
}

fn round_to_even_f32(number: f32) -> i32 {
    (number / 2.0).round() as i32 * 2
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! assert_even {
        ($number:expr) => {
            assert_eq!($number % 2, 0)
        };
    }

    #[test]
    fn odd_round_to_even_f32() {
        assert_even!(round_to_even_f32(3.0));
        assert_even!(round_to_even_f32(99.0));
    }

    #[test]
    fn even_round_to_even_f32() {
        assert_even!(round_to_even_f32(300.0));
        assert_even!(round_to_even_f32(6.0));
    }

    #[test]
    fn float_round_to_even_f32() {
        assert_even!(round_to_even_f32(5.3));
        assert_even!(round_to_even_f32(2.9));
    }

    #[test]
    fn crop_scales_selection_from_widget_to_stream_pixels() {
        // The 1920x1080 stream is drawn at half size, offset by (10, 20).
        let paintable_rect = Rect::new(10.0, 20.0, 960.0, 540.0);
        let selection = Rect::new(110.0, 70.0, 400.0, 300.0);

        let crop = CaptureCrop::from_selection(&selection, &paintable_rect, (1920, 1080));

        assert_eq!(
            crop,
            CaptureCrop {
                stream_size: (1920, 1080),
                top: 100,
                left: 200,
                right: 920,
                bottom: 380,
            }
        );
    }

    #[test]
    fn crop_rounds_to_even_and_clamps_to_stream() {
        // A selection hanging off the top-left corner is clamped to zero, and
        // odd edges are rounded to even pixels.
        let paintable_rect = Rect::new(0.0, 0.0, 101.0, 51.0);
        let selection = Rect::new(-5.0, -5.0, 51.0, 30.0);

        let crop = CaptureCrop::from_selection(&selection, &paintable_rect, (101, 51));

        assert_eq!(
            crop,
            CaptureCrop {
                stream_size: (101, 51),
                top: 0,
                left: 0,
                right: 56,
                bottom: 26,
            }
        );
    }

    /// The area selector and the recording pipeline share capture code through
    /// this module. If either imports the other again, the selector's preview
    /// and the recording graph are coupled in both directions.
    #[test]
    fn area_selector_and_pipeline_do_not_import_each_other() {
        for (file, source) in [
            ("area_selector/mod.rs", include_str!("area_selector/mod.rs")),
            (
                "area_selector/view_port.rs",
                include_str!("area_selector/view_port.rs"),
            ),
        ] {
            assert!(
                !source.contains("pipeline::"),
                "src/{file} uses the recording pipeline module"
            );
        }
        assert!(
            !include_str!("pipeline.rs").contains("area_selector"),
            "src/pipeline.rs uses the area selector module"
        );
    }

    /// The recording side (this module, the pipeline builder and the camera
    /// compositing) builds GStreamer graphs only. If it names a window module,
    /// a change to a window can break recording, as `CameraFeed` in
    /// `bubble.rs` and `SelectAreaData` in `area_selector` once did. Only the
    /// code above each file's tests is checked, so this test does not match
    /// itself.
    #[test]
    fn capture_side_does_not_import_window_modules() {
        for (file, source) in [
            ("capture.rs", include_str!("capture.rs")),
            ("pipeline.rs", include_str!("pipeline.rs")),
            ("camera.rs", include_str!("camera.rs")),
        ] {
            let code = source.split("#[cfg(test)]").next().unwrap();
            for module in ["area_selector", "bubble", "window", "application"] {
                assert!(
                    !code.contains(&format!("{module}::"))
                        && !code.contains(&format!("crate::{module}")),
                    "src/{file} uses the {module} module"
                );
            }
            assert!(
                !code.contains("Application"),
                "src/{file} uses the application"
            );
        }
    }
}
