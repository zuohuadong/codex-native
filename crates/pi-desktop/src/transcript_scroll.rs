//! User intent is independent of layout-driven changes to scroll geometry.

/// GPUI wheel deltas are positive toward older content. Detach on even a small
/// upward gesture: repinning every frame inside the 48px tolerance otherwise
/// prevents a trackpad user from ever escaping the bottom.
pub fn after_wheel(pinned: bool, delta_y: f32, distance: f32, scrollable: bool) -> bool {
    if !scrollable || delta_y == 0.0 {
        return pinned;
    }
    if delta_y > 0.0 {
        false
    } else {
        distance <= 48.0
    }
}

/// Nested output should consume wheel input only while it still has room in that
/// direction. At either edge the event must continue to the transcript so reading
/// a short/finished tool result cannot trap the page, while scrolling inside the
/// result never changes the transcript FOLLOWING/DETACHED intent.
///
/// `offset` and `maximum` are positive pixel magnitudes (distance scrolled and
/// maximum scrollable distance). GPUI stores the live offset in `[−maximum, 0]`,
/// so capturing while moving up requires a non-zero offset, while capturing while
/// moving down requires offset to be below maximum.
pub fn nested_wheel_captures(delta_y: f32, offset: f32, maximum: f32) -> bool {
    if maximum <= 0.0 || delta_y == 0.0 {
        return false;
    }
    let gpui_offset = -offset.abs();
    if delta_y > 0.0 {
        gpui_offset < 0.0
    } else {
        gpui_offset > -maximum
    }
}
/// Keyboard movement in logical scroll-top coordinates (not GPUI's negative
/// offsets). Call only when the transcript itself owns focus, never an editor.
pub fn keyboard_target(
    key: &str,
    shift: bool,
    top: f32,
    maximum: f32,
    viewport: f32,
) -> Option<(f32, bool)> {
    let maximum = maximum.max(0.);
    let page = (viewport * 0.9).max(1.);
    let (target, toward_bottom) = match (key, shift) {
        ("home", false) => (0., false),
        ("end", false) => (maximum, true),
        ("pageup", false) | ("space", true) => (top - page, false),
        ("pagedown", false) | ("space", false) => (top + page, true),
        ("up", false) => (top - 40., false),
        ("down", false) => (top + 40., true),
        _ => return None,
    };
    let target = target.clamp(0., maximum);
    Some((target, toward_bottom && maximum - target <= 48.))
}

#[cfg(test)]
mod tests {
    use super::{after_wheel, keyboard_target, nested_wheel_captures};

    #[test]
    fn keyboard_history_navigation_detaches_then_end_restores_following() {
        let (top, pinned) = keyboard_target("pageup", false, 1000., 1000., 400.).unwrap();
        assert_eq!(top, 640.);
        assert!(!pinned);
        assert_eq!(
            keyboard_target("space", true, top, 1000., 400.),
            Some((280., false))
        );
        assert_eq!(
            keyboard_target("home", false, top, 1000., 400.),
            Some((0., false))
        );
        assert_eq!(
            keyboard_target("end", false, top, 1000., 400.),
            Some((1000., true))
        );
        assert_eq!(
            keyboard_target("pagedown", false, 950., 1000., 400.),
            Some((1000., true))
        );
    }

    #[test]
    fn keyboard_boundaries_and_selection_keys_are_preserved() {
        assert_eq!(
            keyboard_target("up", false, 0., 900., 400.),
            Some((0., false))
        );
        assert_eq!(
            keyboard_target("down", false, 890., 900., 400.),
            Some((900., true))
        );
        assert_eq!(keyboard_target("up", true, 50., 900., 400.), None);
        assert_eq!(keyboard_target("a", false, 50., 900., 400.), None);
        assert_eq!(
            keyboard_target("end", false, 0., 0., 400.),
            Some((0., true))
        );
    }

    #[test]
    fn fine_trackpad_scroll_detaches_and_only_downward_return_repins() {
        let mut pinned = true;
        for distance in [2., 4., 8., 24., 60.] {
            pinned = after_wheel(pinned, 2., distance, true);
            assert!(!pinned);
        }
        // Layout notifications and horizontal gestures cannot reclaim reading.
        assert!(!after_wheel(pinned, 0., 0., true));
        assert!(!after_wheel(pinned, -4., 49., true));
        assert!(after_wheel(pinned, -4., 48., true));
    }

    #[test]
    fn nested_output_propagates_wheel_events_only_at_its_edges() {
        assert!(nested_wheel_captures(10., 40., 100.));
        assert!(!nested_wheel_captures(10., 0., 100.));
        assert!(nested_wheel_captures(-10., 40., 100.));
        assert!(!nested_wheel_captures(-10., 100., 100.));
        assert!(!nested_wheel_captures(10., 0., 0.));
        assert!(!nested_wheel_captures(-10., 0., 0.));
        assert!(!nested_wheel_captures(0., 40., 100.));
        // Raw GPUI offsets are also interpreted correctly.
        assert!(nested_wheel_captures(10., -40., 100.));
        assert!(nested_wheel_captures(-10., -40., 100.));
        assert!(!nested_wheel_captures(10., 0., 100.));
        assert!(!nested_wheel_captures(-10., -100., 100.));
    }

    #[test]
    fn non_scrollable_content_does_not_change_intent() {
        assert!(after_wheel(true, 20., 0., false));
        assert!(!after_wheel(false, -20., 0., false));
    }
}
