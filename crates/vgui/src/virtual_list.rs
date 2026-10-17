//! Virtual list support — exposes gpui's `uniform_list` primitive for
//! rendering large fixed-height lists with only visible items materialised.
//!
//! The `view!` macro emits calls to `gpui::uniform_list` directly; this
//! module provides the [`use_scroll_handle`] hook for programmatic scroll
//! control and re-exports the gpui types users need.

pub use gpui::{ScrollStrategy, UniformListScrollHandle};

/// Get-or-create a persistent [`UniformListScrollHandle`] cached in the
/// current reactive scope slot.
///
/// On the first render a new handle is created and stored; subsequent
/// re-renders return the same handle so scroll position persists. The
/// handle can be passed to `<uniform_list scroll_handle={...}>` and used
/// between renders to call `.scroll_to_item(ix, ScrollStrategy::Top)`,
/// `.scroll_to_bottom()`, `.is_scrolled_to_end()`, etc.
///
/// Must be called inside a `VguiRoot` render scope (same precondition as
/// `create_signal`).
pub fn use_scroll_handle() -> UniformListScrollHandle {
    crate::reactive::get_or_create_slot(|_cx| UniformListScrollHandle::new())
}

/// Measured, variable-height windowing state for a long list.
///
/// Unlike gpui's `uniform_list` (one fixed row height), [`VirtualListState`]
/// stores a per-item height and a prefix-sum offset table, virtualising rows
/// whose heights differ. The state is rendering-agnostic: the UI layer owns
/// the scroll offset and asks [`visible_range`](VirtualListState::visible_range)
/// which items to materialise, plus [`top_spacer`](VirtualListState::top_spacer)
/// / [`bottom_spacer`](VirtualListState::bottom_spacer) to preserve the scroll
/// track height by offsetting the windowed rows.
#[derive(Clone, Debug)]
pub struct VirtualListState {
    heights: Vec<f32>,
    estimated_height: f32,
    overscan: f32,
}

impl VirtualListState {
    /// State for `count` items all assumed to be `estimated_height` px tall.
    pub fn new(count: usize, estimated_height: f32) -> Self {
        Self {
            heights: vec![estimated_height; count],
            estimated_height: estimated_height.max(1.0),
            overscan: 0.0,
        }
    }

    /// Number of items.
    pub fn len(&self) -> usize {
        self.heights.len()
    }

    /// Whether there are no items.
    pub fn is_empty(&self) -> bool {
        self.heights.is_empty()
    }

    /// Change the number of items. New items take the estimated height;
    /// removed items drop their measurement.
    pub fn resize(&mut self, count: usize) {
        self.heights.resize(count, self.estimated_height);
    }

    /// Render extra pixels of content above and below the visible window.
    pub fn with_overscan(mut self, overscan: f32) -> Self {
        self.overscan = overscan.max(0.0);
        self
    }

    /// Record the measured height of item `index`.
    pub fn set_height(&mut self, index: usize, height: f32) {
        if let Some(slot) = self.heights.get_mut(index) {
            *slot = height.max(0.0);
        }
    }

    /// Measured height of one item.
    pub fn item_height(&self, index: usize) -> f32 {
        self.heights.get(index).copied().unwrap_or(self.estimated_height)
    }

    /// Total content height (sum of every item).
    pub fn total_height(&self) -> f32 {
        self.heights.iter().sum()
    }

    /// Offset of item `index` from the top of the content (prefix sum).
    pub fn offset(&self, index: usize) -> f32 {
        self.heights[..index.min(self.heights.len())].iter().sum()
    }

    /// Px of content to place above the first visible row.
    pub fn top_spacer(&self, first: usize) -> f32 {
        self.offset(first)
    }

    /// Px of content to place below the last visible row.
    pub fn bottom_spacer(&self, last_inclusive: usize) -> f32 {
        let end = (last_inclusive + 1).min(self.heights.len());
        self.total_height() - self.offset(end)
    }

    /// Half-open `[start, end)` index range visible for a viewport spanning
    /// `scroll_offset .. scroll_offset + viewport_height`, expanded by the
    /// configured overscan.
    pub fn visible_range(&self, scroll_offset: f32, viewport_height: f32) -> (usize, usize) {
        if self.heights.is_empty() || viewport_height <= 0.0 {
            return (0, 0);
        }
        let top = (scroll_offset - self.overscan).max(0.0);
        let bottom =
            (scroll_offset + viewport_height + self.overscan).min(self.total_height());
        let start = self.index_at(top);
        let mut end = self.index_at(bottom);
        if end < self.heights.len() && self.offset(end) < bottom {
            end += 1;
        }
        (start, end.max(start))
    }

    /// Index of the item whose vertical extent contains `y`. Out-of-range `y`
    /// clamps to the first / last item.
    pub fn index_at(&self, y: f32) -> usize {
        if self.heights.is_empty() {
            return 0;
        }
        if y <= 0.0 {
            return 0;
        }
        if y >= self.total_height() {
            return self.heights.len().saturating_sub(1);
        }
        // Prefix sums are monotonic: binary search for the first offset > y,
        // then step back to the item straddling it.
        let mut lo = 0usize;
        let mut hi = self.heights.len();
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if self.offset(mid) <= y {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo.saturating_sub(1)
    }

    /// Scroll offset (px) that brings item `index` to the top of the viewport.
    pub fn scroll_offset_for(&self, index: usize) -> f32 {
        self.offset(index.min(self.heights.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uniform_heights_window_as_expected() {
        let s = VirtualListState::new(100, 20.0);
        assert_eq!(s.total_height(), 2000.0);
        assert_eq!(s.offset(5), 100.0);
        assert_eq!(s.visible_range(0.0, 100.0), (0, 5));
        assert_eq!(s.visible_range(200.0, 100.0), (10, 15));
    }

    #[test]
    fn variable_heights_use_prefix_offsets() {
        let mut s = VirtualListState::new(4, 10.0);
        s.set_height(0, 40.0);
        s.set_height(1, 10.0);
        s.set_height(2, 100.0);
        s.set_height(3, 50.0);
        assert_eq!(s.total_height(), 200.0);
        assert_eq!(s.offset(2), 50.0);
        assert_eq!(s.index_at(45.0), 1);
        assert_eq!(s.index_at(60.0), 2);
        assert_eq!(s.visible_range(40.0, 50.0), (1, 3));
        assert_eq!(s.top_spacer(1), 40.0);
        assert_eq!(s.bottom_spacer(2), 50.0);
    }

    #[test]
    fn overscan_expands_the_window() {
        let s = VirtualListState::new(100, 20.0).with_overscan(20.0);
        // window 200..300, overscan 20 -> 180..320 -> items 9..16
        assert_eq!(s.visible_range(200.0, 100.0), (9, 16));
    }

    #[test]
    fn scroll_to_index_uses_offset() {
        let mut s = VirtualListState::new(3, 20.0);
        s.set_height(0, 100.0);
        assert_eq!(s.scroll_offset_for(1), 100.0);
    }

    #[test]
    fn resize_preserves_known_heights() {
        let mut s = VirtualListState::new(2, 20.0);
        s.set_height(0, 80.0);
        s.resize(3);
        assert_eq!(s.item_height(0), 80.0);
        assert_eq!(s.item_height(2), 20.0);
        assert_eq!(s.len(), 3);
    }
}
