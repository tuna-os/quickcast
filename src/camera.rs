//! Camera capture is a separate input, burned into the video after screen cropping.
use anyhow::{Context, Result};
use gst::prelude::*;

pub fn devices() -> Vec<(String, String)> {
    let monitor = gst::DeviceMonitor::new();
    monitor.add_filter(Some("Video/Source"), None);
    if monitor.start().is_err() {
        return Vec::new();
    }
    let mut devices = Vec::new();
    for device in monitor.devices() {
        if let Some(props) = device.properties()
            && let Ok(path) = props.get::<String>("device.path")
            && path.starts_with("/dev/video")
            && !devices.iter().any(|(_, p)| p == &path)
        {
            devices.push((device.display_name().to_string(), path));
        }
    }
    monitor.stop();
    devices.sort();
    devices
}

pub fn source(device: &str) -> Result<gst::Bin> {
    anyhow::ensure!(!device.is_empty(), "Choose a camera before recording");
    let bin = gst::Bin::new();
    let source = gst::ElementFactory::make("v4l2src")
        .property("device", device)
        .property("do-timestamp", true)
        .build()?;
    // decodebin also accepts MJPEG-only cameras; do not assume raw USB frames.
    let decode = gst::ElementFactory::make("decodebin").build()?;
    let convert = gst::ElementFactory::make("videoconvert").build()?;
    let scale = gst::ElementFactory::make("videoscale").build()?;
    let caps = gst::ElementFactory::make("capsfilter")
        .property(
            "caps",
            gst::Caps::builder("video/x-raw")
                .field("width", 320i32)
                .field("height", 240i32)
                .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
                .build(),
        )
        .build()?;
    bin.add_many([&source, &decode, &convert, &scale, &caps])?;
    source.link(&decode)?;
    gst::Element::link_many([&convert, &scale, &caps])?;
    let sink = convert
        .static_pad("sink")
        .context("Camera converter has no sink")?;
    decode.connect_pad_added(move |_, pad| {
        if !sink.is_linked()
            && let Err(error) = pad.link(&sink)
        {
            tracing::error!(?error, "Cannot link decoded camera video");
        }
    });
    bin.add_pad(&gst::GhostPad::with_target(
        &caps.static_pad("src").unwrap(),
    )?)?;
    Ok(bin)
}

// Coordinates are relative to the captured region, never the desktop.
fn geometry(width: i32, height: i32, corner: u32) -> (i32, i32, i32, i32) {
    let margin = (width.min(height) / 40).max(0);
    let w = (width / 5).max(2).min((height * 4 / 9).max(2));
    let h = (w * 3 / 4).max(2);
    let x = if corner.is_multiple_of(2) {
        margin
    } else {
        width - w - margin
    };
    let y = if corner < 2 {
        margin
    } else {
        height - h - margin
    };
    (x.max(0), y.max(0), w, h)
}

pub fn attach(
    pipeline: &gst::Pipeline,
    screen: &gst::Element,
    output: &gst::Element,
    camera: Option<(&str, u32)>,
) -> Result<()> {
    let input = camera
        .map(|(device, corner)| source(device).map(|bin| (bin, corner)))
        .transpose()?;
    attach_inputs(pipeline, screen, output, input)
}

fn attach_inputs(
    pipeline: &gst::Pipeline,
    screen: &gst::Element,
    output: &gst::Element,
    camera: Option<(gst::Bin, u32)>,
) -> Result<()> {
    let compositor = gst::ElementFactory::make("compositor")
        .property("ignore-inactive-pads", true)
        .build()?;
    let canvas = gst::ElementFactory::make("capsfilter").build()?;
    let scale = gst::ElementFactory::make("videoscale").build()?;
    let size = gst::ElementFactory::make("capsfilter").build()?;
    pipeline.add_many([&compositor, &canvas, &scale, &size])?;
    screen.link_pads(Some("src"), &compositor, Some("sink_0"))?;
    gst::Element::link_many([&compositor, &canvas, &scale, &size, output])?;
    let overlay = if let Some((source, corner)) = camera {
        let queue = gst::ElementFactory::make("queue")
            .property_from_str("leaky", "downstream")
            .property("max-size-buffers", 2u32)
            .build()?;
        pipeline.add_many([source.upcast_ref(), &queue])?;
        source.link(&queue)?;
        queue.link_pads(Some("src"), &compositor, Some("sink_1"))?;
        let pad = compositor.static_pad("sink_1").unwrap();
        pad.set_property("zorder", 1u32);
        Some((pad, corner))
    } else {
        None
    };
    // Follow source resizes and fractional-scale capture dimensions. Fixing the
    // canvas to the screen prevents the camera determining the output extent.
    screen
        .static_pad("src")
        .context("Screen has no source pad")?
        .add_probe(gst::PadProbeType::EVENT_DOWNSTREAM, move |_, info| {
            if let Some(gst::PadProbeData::Event(ref event)) = info.data
                && let gst::EventView::Caps(caps) = event.view()
                && let Some(s) = caps.caps().structure(0)
                && let (Ok(w), Ok(h)) = (s.get::<i32>("width"), s.get::<i32>("height"))
                && w > 0
                && h > 0
            {
                canvas.set_property(
                    "caps",
                    gst::Caps::builder("video/x-raw")
                        .field("width", w)
                        .field("height", h)
                        .build(),
                );
                let ratio = (1920.0 / f64::from(w)).min(1080.0 / f64::from(h)).min(1.0);
                let ow = ((f64::from(w) * ratio) as i32 / 2 * 2).max(2);
                let oh = ((f64::from(h) * ratio) as i32 / 2 * 2).max(2);
                size.set_property(
                    "caps",
                    gst::Caps::builder("video/x-raw")
                        .field("width", ow)
                        .field("height", oh)
                        .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
                        .build(),
                );
                if let Some((ref pad, corner)) = overlay {
                    let (x, y, width, height) = geometry(w, h, corner);
                    pad.set_property("xpos", x);
                    pad.set_property("ypos", y);
                    pad.set_property("width", width);
                    pad.set_property("height", height);
                }
            }
            gst::PadProbeReturn::Ok
        });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlay_stays_in_captured_region() {
        for (width, height) in [(1920, 1080), (3840, 2160), (301, 901), (100, 40)] {
            for corner in 0..4 {
                let (x, y, w, h) = geometry(width, height, corner);
                assert!(x >= 0 && y >= 0 && x + w <= width && y + h <= height);
            }
        }
    }
    #[test]
    fn records_composited_mp4_with_audio() {
        gst::init().unwrap();
        let resources = gtk::gio::Resource::load(crate::config::RESOURCES_FILE).unwrap();
        gtk::gio::resources_register(&resources);
        let directory = std::env::var("QUICKCAST_TEST_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("quickcast-tests"));
        std::fs::create_dir_all(&directory).unwrap();
        let file = directory.join("composite.mp4");
        let pipeline = gst::Pipeline::new();
        let screen = gst::parse::bin_from_description(
            "videotestsrc num-buffers=60 pattern=blue ! video/x-raw,width=640,height=360,framerate=30/1 ! identity", true).unwrap();
        let camera = gst::parse::bin_from_description(
            "videotestsrc num-buffers=60 pattern=red ! video/x-raw,width=320,height=240,framerate=30/1 ! identity", true).unwrap();
        let audio = gst::parse::bin_from_description(
            "audiotestsrc num-buffers=100 samplesperbuffer=960 ! audio/x-raw,rate=48000 ! queue",
            true,
        )
        .unwrap();
        let video_queue = gst::ElementFactory::make("queue").build().unwrap();
        let sink = gst::ElementFactory::make("filesink")
            .property("location", file.to_str().unwrap())
            .build()
            .unwrap();
        pipeline
            .add_many([screen.upcast_ref(), audio.upcast_ref(), &video_queue, &sink])
            .unwrap();
        attach_inputs(
            &pipeline,
            screen.upcast_ref(),
            &video_queue,
            Some((camera, 3)),
        )
        .unwrap();
        crate::profile::Profile::from_id("mp4")
            .unwrap()
            .attach(&pipeline, &video_queue, Some(audio.upcast_ref()), &sink)
            .unwrap();
        pipeline.set_state(gst::State::Playing).unwrap();
        let message = pipeline.bus().unwrap().timed_pop_filtered(
            gst::ClockTime::from_seconds(20),
            &[gst::MessageType::Eos, gst::MessageType::Error],
        );
        pipeline.set_state(gst::State::Null).unwrap();
        match message.as_ref().map(|m| m.view()) {
            Some(gst::MessageView::Eos(_)) => {}
            Some(gst::MessageView::Error(error)) => {
                panic!("{}: {:?}", error.error(), error.debug())
            }
            _ => panic!("Synthetic recording did not finalize in 20 seconds"),
        }
        assert!(std::fs::metadata(file).unwrap().len() > 1000);
    }
}
