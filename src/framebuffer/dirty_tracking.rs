//! Dirty region tracking for automatic refresh optimization
//!
//! This module provides zero-cost dirty region tracking that can be enabled
//! with the `framebuffer-dirty-tracking` feature flag. When disabled, all
//! tracking operations compile to no-ops with zero runtime overhead.
//!
//! # Examples
//!
//! ```no_run
//! use libremarkable::framebuffer::dirty_tracking::DirtyRegionTracker;
//! use libremarkable::framebuffer::common::mxcfb_rect;
//!
//! let mut tracker = DirtyRegionTracker::new();
//!
//! // Track regions as they're modified
//! tracker.mark_dirty(mxcfb_rect { top: 0, left: 0, width: 100, height: 100 });
//! tracker.mark_dirty(mxcfb_rect { top: 50, left: 50, width: 100, height: 100 });
//!
//! // Get merged bounding box
//! if let Some(merged) = tracker.get_merged() {
//!     println!("Dirty region: {:?}", merged);
//! }
//!
//! // Or get individual regions for non-rectangular refresh
//! let regions = tracker.get_regions();
//! println!("Found {} dirty regions", regions.len());
//! ```

use crate::framebuffer::common::mxcfb_rect;

/// Tracks dirty (modified) regions in the framebuffer for refresh optimization
///
/// When the `framebuffer-dirty-tracking` feature is enabled, this struct
/// accumulates regions that have been drawn to. When disabled, it becomes
/// a zero-sized type with no-op methods for zero runtime overhead.
#[cfg(feature = "framebuffer-dirty-tracking")]
#[derive(Debug, Clone)]
pub struct DirtyRegionTracker {
    /// Individual dirty regions before merging
    regions: Vec<mxcfb_rect>,
    /// Merged bounding box of all dirty regions
    merged: Option<mxcfb_rect>,
    /// Quick check if any modifications occurred
    dirty: bool,
}

/// Zero-sized placeholder when dirty tracking is disabled
#[cfg(not(feature = "framebuffer-dirty-tracking"))]
#[derive(Debug, Clone, Copy)]
pub struct DirtyRegionTracker;

#[cfg(feature = "framebuffer-dirty-tracking")]
impl DirtyRegionTracker {
    /// Create a new dirty region tracker
    #[inline]
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            merged: None,
            dirty: false,
        }
    }

    /// Mark a region as dirty (modified)
    ///
    /// This accumulates the region for later refresh optimization.
    #[inline]
    pub fn mark_dirty(&mut self, rect: mxcfb_rect) {
        self.dirty = true;

        // Update merged bounding box
        self.merged = Some(match self.merged {
            Some(existing) => existing.merge_rect(&rect),
            None => rect,
        });

        // Store individual region for non-rectangular refresh
        self.regions.push(rect);
    }

    /// Get all accumulated dirty regions
    ///
    /// Returns individual regions for non-rectangular refresh strategies.
    /// For simple refresh, use `get_merged()` instead.
    #[inline]
    pub fn get_regions(&self) -> &[mxcfb_rect] {
        &self.regions
    }

    /// Get the merged bounding box of all dirty regions
    ///
    /// Returns `None` if no regions have been marked dirty.
    /// This is the simplest way to refresh all modified areas with a single operation.
    #[inline]
    pub fn get_merged(&self) -> Option<mxcfb_rect> {
        self.merged
    }

    /// Clear all tracked dirty regions
    ///
    /// Call this after refreshing to start tracking a new set of modifications.
    #[inline]
    pub fn clear(&mut self) {
        self.regions.clear();
        self.merged = None;
        self.dirty = false;
    }

    /// Check if any modifications have occurred
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Optimize tracked regions by merging adjacent/overlapping rectangles
    ///
    /// This reduces the number of individual regions while trying to minimize
    /// wasted refresh area. The `threshold` parameter controls how aggressively
    /// to merge - regions within `threshold` pixels of each other will be merged.
    ///
    /// # Arguments
    ///
    /// * `threshold` - Maximum pixel distance between regions to consider for merging
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use libremarkable::framebuffer::dirty_tracking::DirtyRegionTracker;
    /// # let mut tracker = DirtyRegionTracker::new();
    /// // ... mark some dirty regions ...
    /// tracker.optimize(10); // Merge regions within 10 pixels of each other
    /// ```
    pub fn optimize(&mut self, threshold: u32) {
        if self.regions.len() <= 1 {
            return;
        }

        let mut optimized = Vec::new();
        let mut remaining: Vec<_> = self.regions.drain(..).collect();

        while let Some(mut current) = remaining.pop() {
            let mut merged_any = true;

            // Keep trying to merge until no more merges possible
            while merged_any {
                merged_any = false;
                remaining.retain(|other| {
                    if should_merge(&current, other, threshold) {
                        current = current.merge_rect(other);
                        merged_any = true;
                        false // Remove from remaining
                    } else {
                        true // Keep in remaining
                    }
                });
            }

            optimized.push(current);
        }

        self.regions = optimized;
    }
}

#[cfg(not(feature = "framebuffer-dirty-tracking"))]
impl DirtyRegionTracker {
    /// Create a new dirty region tracker (no-op when feature disabled)
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    /// Mark a region as dirty (no-op when feature disabled)
    #[inline]
    pub fn mark_dirty(&mut self, _rect: mxcfb_rect) {
        // No-op when feature disabled
    }

    /// Get all accumulated dirty regions (no-op when feature disabled)
    #[inline]
    pub fn get_regions(&self) -> &[mxcfb_rect] {
        &[]
    }

    /// Get the merged bounding box (no-op when feature disabled)
    #[inline]
    pub fn get_merged(&self) -> Option<mxcfb_rect> {
        None
    }

    /// Clear all tracked dirty regions (no-op when feature disabled)
    #[inline]
    pub fn clear(&mut self) {
        // No-op when feature disabled
    }

    /// Check if any modifications have occurred (no-op when feature disabled)
    #[inline]
    pub fn is_dirty(&self) -> bool {
        false
    }

    /// Optimize tracked regions (no-op when feature disabled)
    #[inline]
    pub fn optimize(&mut self, _threshold: u32) {
        // No-op when feature disabled
    }
}

impl Default for DirtyRegionTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to determine if two rects should be merged based on proximity
#[cfg(feature = "framebuffer-dirty-tracking")]
fn should_merge(a: &mxcfb_rect, b: &mxcfb_rect, threshold: u32) -> bool {
    // Check if rects overlap or are within threshold distance
    let a_right = a.left + a.width;
    let a_bottom = a.top + a.height;
    let b_right = b.left + b.width;
    let b_bottom = b.top + b.height;

    // Calculate distances between edges
    let horizontal_gap = if a_right < b.left {
        b.left - a_right
    } else if b_right < a.left {
        a.left - b_right
    } else {
        0 // Overlapping or touching
    };

    let vertical_gap = if a_bottom < b.top {
        b.top - a_bottom
    } else if b_bottom < a.top {
        a.top - b_bottom
    } else {
        0 // Overlapping or touching
    };

    // Merge if both gaps are within threshold
    horizontal_gap <= threshold && vertical_gap <= threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker_is_clean() {
        let tracker = DirtyRegionTracker::new();
        assert!(!tracker.is_dirty());
        assert_eq!(tracker.get_regions().len(), 0);
        assert!(tracker.get_merged().is_none());
    }

    #[cfg(feature = "framebuffer-dirty-tracking")]
    #[test]
    fn test_mark_dirty_sets_flag() {
        let mut tracker = DirtyRegionTracker::new();
        tracker.mark_dirty(mxcfb_rect {
            top: 0,
            left: 0,
            width: 100,
            height: 100,
        });
        assert!(tracker.is_dirty());
        assert_eq!(tracker.get_regions().len(), 1);
    }

    #[cfg(feature = "framebuffer-dirty-tracking")]
    #[test]
    fn test_merge_regions() {
        let mut tracker = DirtyRegionTracker::new();

        tracker.mark_dirty(mxcfb_rect {
            top: 0,
            left: 0,
            width: 100,
            height: 100,
        });

        tracker.mark_dirty(mxcfb_rect {
            top: 50,
            left: 50,
            width: 100,
            height: 100,
        });

        let merged = tracker.get_merged().unwrap();
        assert_eq!(merged.top, 0);
        assert_eq!(merged.left, 0);
        assert_eq!(merged.width, 150);
        assert_eq!(merged.height, 150);
    }

    #[cfg(feature = "framebuffer-dirty-tracking")]
    #[test]
    fn test_clear_resets_tracker() {
        let mut tracker = DirtyRegionTracker::new();
        tracker.mark_dirty(mxcfb_rect {
            top: 0,
            left: 0,
            width: 100,
            height: 100,
        });

        tracker.clear();

        assert!(!tracker.is_dirty());
        assert_eq!(tracker.get_regions().len(), 0);
        assert!(tracker.get_merged().is_none());
    }

    #[cfg(feature = "framebuffer-dirty-tracking")]
    #[test]
    fn test_optimize_merges_adjacent() {
        let mut tracker = DirtyRegionTracker::new();

        // Two adjacent rects
        tracker.mark_dirty(mxcfb_rect { top: 0, left: 0, width: 100, height: 100 });
        tracker.mark_dirty(mxcfb_rect { top: 0, left: 105, width: 100, height: 100 });

        assert_eq!(tracker.get_regions().len(), 2);

        tracker.optimize(10);

        // Should be merged into one
        assert_eq!(tracker.get_regions().len(), 1);
    }

    #[cfg(feature = "framebuffer-dirty-tracking")]
    #[test]
    fn test_optimize_keeps_distant() {
        let mut tracker = DirtyRegionTracker::new();

        // Two distant rects
        tracker.mark_dirty(mxcfb_rect { top: 0, left: 0, width: 100, height: 100 });
        tracker.mark_dirty(mxcfb_rect { top: 200, left: 200, width: 100, height: 100 });

        assert_eq!(tracker.get_regions().len(), 2);

        tracker.optimize(10);

        // Should remain separate
        assert_eq!(tracker.get_regions().len(), 2);
    }
}
