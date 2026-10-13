/// Screen capture abstraction for recording.
///
/// This module provides domain-level types for screen capture, hiding D-Bus and
/// XDG Desktop Portal implementation details from callers.

use crate::screencast_portal;

/// Represents a rectangular region on the screen for capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureRegion {
    /// X coordinate of the top-left corner in pixels.
    pub x: i32,
    /// Y coordinate of the top-left corner in pixels.
    pub y: i32,
    /// Width of the capture region in pixels.
    pub width: u32,
    /// Height of the capture region in pixels.
    pub height: u32,
}

impl CaptureRegion {
    /// Creates a new capture region.
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the area in square pixels.
    pub fn area(&self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }

    /// Checks if this region contains the given point.
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + (self.width as i32)
            && y >= self.y
            && y < self.y + (self.height as i32)
    }
}

/// Describes what to capture from the screen.
#[derive(Debug, Clone, Copy)]
pub enum CaptureSource {
    /// Capture a specific monitor by index.
    Monitor(u32),
    /// Capture a specific window.
    Window(u32),
    /// Capture a user-selected region.
    Region(CaptureRegion),
}

impl CaptureSource {
    /// Converts to the portal-level source type for negotiation.
    pub(crate) fn to_portal_source_type(&self) -> screencast_portal::SourceType {
        match self {
            Self::Monitor(_) => screencast_portal::SourceType::MONITOR,
            Self::Window(_) => screencast_portal::SourceType::WINDOW,
            Self::Region(_) => screencast_portal::SourceType::VIRTUAL,
        }
    }
}

/// Represents an active screen capture session.
///
/// This abstracts away D-Bus and XDG Desktop Portal details, providing a
/// clean interface for recording to work with screen capture.
#[derive(Debug)]
pub struct CaptureSession {
    /// Session ID from the portal
    pub(crate) session_id: String,
    /// The source being captured
    pub(crate) source: CaptureSource,
}

impl CaptureSession {
    /// Creates a new capture session (typically called by the portal wrapper).
    pub(crate) fn new(session_id: String, source: CaptureSource) -> Self {
        Self {
            session_id,
            source,
        }
    }

    /// Returns the capture source for this session.
    pub fn source(&self) -> CaptureSource {
        self.source
    }

    /// Returns the session ID (for D-Bus operations, kept internal).
    pub(crate) fn session_id(&self) -> &str {
        &self.session_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_region_area() {
        let region = CaptureRegion::new(10, 20, 640, 480);
        assert_eq!(region.area(), 640 * 480);
    }

    #[test]
    fn test_capture_region_contains() {
        let region = CaptureRegion::new(100, 100, 200, 200);
        assert!(region.contains(150, 150));
        assert!(!region.contains(50, 50));
        assert!(!region.contains(350, 150));
    }

    #[test]
    fn test_capture_source_portal_types() {
        assert_eq!(
            CaptureSource::Monitor(0).to_portal_source_type(),
            screencast_portal::SourceType::MONITOR
        );
        assert_eq!(
            CaptureSource::Window(0).to_portal_source_type(),
            screencast_portal::SourceType::WINDOW
        );
        assert_eq!(
            CaptureSource::Region(CaptureRegion::new(0, 0, 100, 100))
                .to_portal_source_type(),
            screencast_portal::SourceType::VIRTUAL
        );
    }
}
