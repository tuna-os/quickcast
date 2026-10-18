use std::time::Duration;

use anyhow::Result;
use gtk::{glib::{self, clone}, subclass::prelude::*};

use super::{Recording, RecordingState};
use crate::timer::Timer;

pub(super) async fn wait(recording: &Recording, duration: Duration) -> Result<()> {
    let timer = Timer::new(
        duration,
        clone!(
            #[weak(rename_to = obj)]
            recording,
            move |secs_left| {
                obj.set_state(RecordingState::Delayed { secs_left });
            }
        ),
    );
    recording.imp().timer.replace(Some(Timer::clone(&timer)));
    timer.await?;

    Ok(())
}
