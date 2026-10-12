/// Capture region abstraction for screen recording.
///
/// This module defines domain types for capture regions, decoupling the
/// pipeline layer from UI-specific area selection logic.

/// Represents a rectangular region on the screen for capture.
///
/// This is a domain type that abstracts away UI implementation details
/// (like screencast portal or area selector widgets). The pipeline layer
/// depends on this type, not on area_selector-specific structures.
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
}
