use super::*;

#[gtk::test]
fn initial_state() {
    let recording = Recording::new();
    assert_eq!(recording.state(), RecordingState::Init);
}

#[gtk::test]
fn pause_and_resume_require_matching_state() {
    let recording = Recording::new();

    // The state guards run before the pipeline is touched, so no
    // pipeline is needed to test the rejection paths.
    for state in [
        RecordingState::Init,
        RecordingState::Delayed { secs_left: 3 },
        RecordingState::Paused,
        RecordingState::Flushing { progress: 10 },
        RecordingState::Finished,
    ] {
        recording.set_state(state);
        assert!(recording.pause().is_err(), "pause allowed in {state:?}");
    }

    for state in [
        RecordingState::Init,
        RecordingState::Delayed { secs_left: 3 },
        RecordingState::Recording,
        RecordingState::Flushing { progress: 10 },
        RecordingState::Finished,
    ] {
        recording.set_state(state);
        assert!(recording.resume().is_err(), "resume allowed in {state:?}");
    }
}

#[gtk::test]
fn stop_is_noop_in_terminal_or_idle_states() {
    for state in [
        RecordingState::Init,
        RecordingState::Flushing { progress: 50 },
        RecordingState::Finished,
    ] {
        let recording = Recording::new();
        recording.set_state(state);
        recording.stop();
        assert_eq!(recording.state(), state);
    }
}

#[gtk::test]
fn set_state_notifies_only_on_change() {
    let recording = Recording::new();
    let count = Rc::new(Cell::new(0));

    recording.connect_state_notify({
        let count = Rc::clone(&count);
        move |_| count.set(count.get() + 1)
    });

    recording.set_state(RecordingState::Init);
    assert_eq!(count.get(), 0);

    recording.set_state(RecordingState::Delayed { secs_left: 2 });
    recording.set_state(RecordingState::Delayed { secs_left: 1 });
    recording.set_state(RecordingState::Recording);
    recording.set_state(RecordingState::Recording);
    assert_eq!(count.get(), 3);
    assert_eq!(recording.state(), RecordingState::Recording);
}

#[gtk::test]
fn set_finished_emits_error_and_sets_finished_state() {
    let recording = Recording::new();
    let emitted = Rc::new(Cell::new(false));

    recording.connect_finished({
        let emitted = Rc::clone(&emitted);
        move |_, res| {
            assert!(res.is_err());
            emitted.set(true);
        }
    });

    recording.set_finished(Err(Error::msg("boom")));

    assert!(emitted.get());
    assert_eq!(recording.state(), RecordingState::Finished);
}

#[gtk::test]
fn new_recording_path_uses_location_and_extension() {
    let path = new_recording_path(Path::new("/tmp/videos"), "webm");
    assert_eq!(path.parent(), Some(Path::new("/tmp/videos")));
    assert_eq!(path.extension(), Some(OsStr::new("webm")));
    assert!(
        path.file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("Kooha-")
    );
}
