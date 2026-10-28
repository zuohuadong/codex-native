//! Viewport geometry adapted from ZCode 3.16.2 timeline-ledge.ts (Apache-2.0).
//! These coordinates are logical pixels, independent of native display scale.

use gpui_kit::{point, px, ScrollHandle};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

/// Opacity fade from `.wf-sb`: 160ms, cubic-bezier(0.22, 0.61, 0.36, 1).
pub const SCROLLBAR_FADE: Duration = Duration::from_millis(160);
/// Track height from `.wf-sb`: 120ms on the same curve.
pub const SCROLLBAR_THICKEN: Duration = Duration::from_millis(120);
/// `useTimelineViewport` keeps `scrolling` true for 800ms after the last scroll.
pub const SCROLLBAR_SCROLL_HOLD: Duration = Duration::from_millis(800);
const SCROLLBAR_REST_OPACITY: f32 = 0.;
const SCROLLBAR_REST_THICKNESS: f32 = 2.;
const SCROLLBAR_THICK: f32 = 4.;
const THUMB_MIN: f32 = 24.;
/// Native approximation of browser `behavior: "smooth"`; reference timing is
/// browser-owned and still needs calibration against matched captures.
const SCROLL_TRAVEL: Duration = Duration::from_millis(300);

/// State shared by the timeline, its camera, and its scrollbar across renders.
#[derive(Clone, Default)]
pub struct Viewport {
    pub scroll: ScrollHandle,
    pub camera: Rc<Cell<Option<usize>>>,
    requested_camera: Rc<Cell<Option<usize>>>,
    dragging: Rc<Cell<bool>>,
    hovered: Rc<Cell<bool>>,
    bar_hovered: Rc<Cell<bool>>,
    last_offset: Rc<Cell<Option<f32>>>,
    scrolling_until: Rc<Cell<Option<Instant>>>,
    hold_wake: Rc<Cell<Option<Instant>>>,
    opacity: Rc<Cell<Option<Channel>>>,
    thickness: Rc<Cell<Option<Channel>>>,
    motion_running: Rc<Cell<bool>>,
    scroll_motion: Rc<Cell<Option<Channel>>>,
    scroll_motion_index: Rc<Cell<Option<usize>>>,
    bar_origin_x: Rc<Cell<Option<f32>>>,
    bar_origin_y: Rc<Cell<Option<f32>>>,
    bar_height: Rc<Cell<f32>>,
    metrics: Rc<Cell<BarMetrics>>,
    thumb_drag: Rc<Cell<Option<ThumbDrag>>>,
}

#[derive(Clone, Copy)]
struct Channel {
    from: f32,
    to: f32,
    started: Instant,
}

#[derive(Clone, Copy, Default)]
struct BarMetrics {
    scroll_left: f32,
    client_width: f32,
    scroll_width: f32,
    thumb_left: f32,
    thumb_width: f32,
    has_thumb: bool,
}

#[derive(Clone, Copy)]
struct ThumbDrag {
    start_x: f32,
    start_left: f32,
    ratio: f32,
    maximum: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarThumb {
    pub left: f32,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarFrame {
    pub opacity: f32,
    pub thickness: f32,
    pub thumb: Option<ScrollbarThumb>,
}

impl Viewport {
    pub fn set_hovered(&self, hovered: bool) {
        self.hovered.set(hovered);
    }

    pub fn set_bar_hovered(&self, hovered: bool) -> bool {
        if self.bar_hovered.get() == hovered {
            return false;
        }
        self.bar_hovered.set(hovered);
        true
    }

    pub fn mark_user_interaction(&self) {
        // A manual gesture settles the pending camera request without moving.
        self.camera.set(self.requested_camera.get());
        self.scroll_motion.set(None);
        self.scroll_motion_index.set(None);
    }

    #[cfg(test)]
    pub fn start_drag(&self) {
        self.dragging.set(true);
    }

    pub fn end_drag(&self) {
        self.dragging.set(false);
        self.thumb_drag.set(None);
    }

    pub fn reset_drag_if_invalid(&self, width: f32, maximum: f32) {
        if width <= 0. || maximum <= 0. {
            self.end_drag();
            self.scroll_motion.set(None);
            self.scroll_motion_index.set(None);
        }
    }

    pub fn prepare_camera(&self, running_index: Option<usize>) {
        self.requested_camera.set(running_index);
    }

    pub fn camera_target(&self, width: f32, maximum: f32) -> Option<(usize, f32)> {
        if width <= 0. {
            return None;
        }
        let requested = self.requested_camera.get();
        if self.camera.replace(requested) == requested {
            return None;
        }
        requested.map(|index| (index, camera_left(index, width, maximum)))
    }

    /// Keep the station that a smooth programmatic move is bringing into view
    /// out of the edge ledge until the move settles.
    pub fn camera_keep_index(&self) -> Option<usize> {
        self.scroll_motion_index.get().or_else(|| {
            (self.requested_camera.get() != self.camera.get())
                .then_some(self.requested_camera.get())
                .flatten()
        })
    }

    /// A new pointer sequence cannot inherit an abandoned drag from a previous
    /// mount. The bubble handler starts the drag for this press afterward.
    pub fn begin_pointer_sequence(&self) {
        self.end_drag();
    }

    /// The first observation only records the position. A later change holds
    /// the scrollbar open, matching the viewport hook's scroll listener.
    pub fn note_offset(&self, offset: f32, now: Instant) -> Option<Instant> {
        let previous = self.last_offset.get();
        if previous.is_some_and(|previous| (previous - offset).abs() <= 0.5) {
            return None;
        }
        self.last_offset.set(Some(offset));
        if previous.is_none() {
            return None;
        }
        let until = now + SCROLLBAR_SCROLL_HOLD;
        self.scrolling_until.set(Some(until));
        Some(until)
    }

    pub fn scrolling_deadline(&self) -> Option<Instant> {
        self.scrolling_until.get()
    }

    pub fn hold_armed(&self) -> Option<Instant> {
        self.hold_wake.get()
    }

    pub fn arm_hold(&self, deadline: Instant) {
        self.hold_wake.set(Some(deadline));
    }

    pub fn disarm_hold(&self, deadline: Instant) {
        if self.hold_wake.get() == Some(deadline) {
            self.hold_wake.set(None);
        }
    }

    pub fn set_bar_bounds(&self, origin_x: f32, origin_y: f32, height: f32) {
        self.bar_origin_x.set(Some(origin_x));
        self.bar_origin_y.set(Some(origin_y));
        self.bar_height.set(height.max(0.));
    }

    pub fn bar_local_x(&self, window_x: f32) -> Option<f32> {
        self.bar_origin_x.get().map(|origin| window_x - origin)
    }

    pub fn bar_contains_y(&self, window_y: f32) -> bool {
        self.bar_origin_y
            .get()
            .is_some_and(|origin| window_y >= origin && window_y <= origin + self.bar_height.get())
    }

    pub fn motion_is_running(&self) -> bool {
        self.motion_running.get()
    }

    pub fn hits_thumb(&self, local_x: f32) -> bool {
        let metrics = self.metrics.get();
        metrics.has_thumb
            && local_x >= metrics.thumb_left
            && local_x < metrics.thumb_left + metrics.thumb_width
    }

    pub fn begin_thumb_drag(&self, window_x: f32) {
        let metrics = self.metrics.get();
        if !metrics.has_thumb || metrics.client_width <= 0. {
            return;
        }
        self.mark_user_interaction();
        self.dragging.set(true);
        self.thumb_drag.set(Some(ThumbDrag {
            start_x: window_x,
            start_left: metrics.scroll_left,
            ratio: metrics.scroll_width / metrics.client_width,
            maximum: (metrics.scroll_width - metrics.client_width).max(0.),
        }));
    }

    pub fn drag_thumb(&self, window_x: f32) -> bool {
        let Some(drag) = self.thumb_drag.get() else {
            return false;
        };
        let left = scrollbar_drag_left(
            drag.start_left,
            window_x - drag.start_x,
            drag.ratio,
            drag.maximum,
        );
        self.user_scroll_to(left)
    }

    pub fn page_from_local(&self, local_x: f32, reduced_motion: bool, now: Instant) -> bool {
        let metrics = self.metrics.get();
        if !metrics.has_thumb || self.hits_thumb(local_x) {
            return false;
        }
        let current = -f32::from(self.scroll.offset().x);
        let width = f32::from(self.scroll.bounds().size.width);
        let maximum = f32::from(self.scroll.max_offset().x);
        let forward = local_x >= metrics.thumb_left + metrics.thumb_width;
        let requested = if forward { current + 1. } else { current - 1. };
        let target = scrollbar_target(false, current, requested, width, maximum);
        self.mark_user_interaction();
        self.scroll_to(target, reduced_motion, now)
    }

    /// Programmatic camera, ledge and track moves share one interruptible
    /// transition; a replacement always starts at the displayed position.
    pub fn scroll_to(&self, target: f32, reduced_motion: bool, now: Instant) -> bool {
        if !target.is_finite() {
            return false;
        }
        self.scroll_motion_index.set(None);
        let offset = self.scroll.offset();
        let from = -f32::from(offset.x);
        let maximum = f32::from(self.scroll.max_offset().x).max(0.);
        let target = target.clamp(0., maximum);
        self.scroll_motion.set(None);
        if (from - target).abs() < 0.5 || reduced_motion {
            self.scroll.set_offset(point(px(-target), offset.y));
            return from != target;
        }
        self.scroll_motion.set(Some(Channel {
            from,
            to: target,
            started: now,
        }));
        true
    }

    pub fn scroll_to_stage(
        &self,
        index: usize,
        target: f32,
        reduced_motion: bool,
        now: Instant,
    ) -> bool {
        let moved = self.scroll_to(target, reduced_motion, now);
        if self.scroll_is_moving() {
            self.scroll_motion_index.set(Some(index));
        }
        moved
    }

    pub fn scroll_is_moving(&self) -> bool {
        self.scroll_motion.get().is_some()
    }

    /// Called after layout so resize and content changes clamp to the live range.
    pub fn advance_scroll(&self, maximum: f32, reduced_motion: bool, now: Instant) -> bool {
        let Some(channel) = self.scroll_motion.get() else {
            return false;
        };
        let (left, running) = scroll_motion_frame(channel, maximum, reduced_motion, now);
        if !running {
            self.scroll_motion.set(None);
            self.scroll_motion_index.set(None);
        }
        let offset = self.scroll.offset();
        if offset.x == px(-left) {
            // Clearing the camera keep still changes the rendered ledges even
            // when the scroll position already reached its final pixel.
            return !running;
        }
        self.scroll.set_offset(point(px(-left), offset.y));
        true
    }

    pub fn finish_thumb_drag(&self) -> bool {
        let active = self.dragging.get() || self.thumb_drag.get().is_some();
        self.end_drag();
        active
    }

    pub fn presentation(
        &self,
        scroll_left: f32,
        client_width: f32,
        scroll_width: f32,
        reduced_motion: bool,
        now: Instant,
    ) -> ScrollbarFrame {
        let thumb = scrollbar_thumb(scroll_left, client_width, scroll_width);
        let mut metrics = BarMetrics {
            scroll_left,
            client_width,
            scroll_width,
            ..BarMetrics::default()
        };
        if let Some(thumb) = thumb {
            metrics.thumb_left = thumb.left;
            metrics.thumb_width = thumb.width;
            metrics.has_thumb = true;
        }
        self.metrics.set(metrics);
        if thumb.is_none() {
            self.motion_running.set(false);
            return ScrollbarFrame {
                opacity: 0.,
                thickness: SCROLLBAR_REST_THICKNESS,
                thumb: None,
            };
        }
        let revealed = scrollbar_revealed(
            self.hovered.get(),
            self.bar_hovered.get(),
            self.dragging.get(),
            self.scrolling_until.get(),
            now,
        );
        let thick = scrollbar_thick(self.bar_hovered.get(), self.dragging.get());
        let (opacity, opacity_running) = sample_channel(
            &self.opacity,
            SCROLLBAR_REST_OPACITY,
            if revealed { 1. } else { 0. },
            now,
            SCROLLBAR_FADE,
            reduced_motion,
        );
        let (thickness, thickness_running) = sample_channel(
            &self.thickness,
            SCROLLBAR_REST_THICKNESS,
            if thick {
                SCROLLBAR_THICK
            } else {
                SCROLLBAR_REST_THICKNESS
            },
            now,
            SCROLLBAR_THICKEN,
            reduced_motion,
        );
        self.motion_running
            .set(opacity_running || thickness_running);
        ScrollbarFrame {
            opacity,
            thickness,
            thumb,
        }
    }

    fn user_scroll_to(&self, left: f32) -> bool {
        self.mark_user_interaction();
        let offset = self.scroll.offset();
        let current = -f32::from(offset.x);
        if (current - left).abs() < 0.5 {
            return false;
        }
        self.scroll.set_offset(point(px(-left), offset.y));
        true
    }
}

fn scroll_motion_frame(
    channel: Channel,
    maximum: f32,
    reduced_motion: bool,
    now: Instant,
) -> (f32, bool) {
    if !maximum.is_finite() || maximum <= 0. {
        return (0., false);
    }
    let running = !reduced_motion && now.saturating_duration_since(channel.started) < SCROLL_TRAVEL;
    let left = if running {
        channel_value(channel, now, SCROLL_TRAVEL)
    } else {
        channel.to
    };
    (left.clamp(0., maximum), running)
}

/// The bar is invisible until the timeline, the bar, a drag, or a recent scroll shows it.
pub fn scrollbar_revealed(
    timeline_hovered: bool,
    bar_hovered: bool,
    dragging: bool,
    scrolling_until: Option<Instant>,
    now: Instant,
) -> bool {
    timeline_hovered || bar_hovered || dragging || scrolling_until.is_some_and(|until| now < until)
}

/// Hovering or dragging the bar itself thickens the 2px track to 4px.
pub fn scrollbar_thick(bar_hovered: bool, dragging: bool) -> bool {
    bar_hovered || dragging
}

/// Thumb length is viewport squared over content, at least 24px. No overflow, no thumb.
pub fn scrollbar_thumb(
    scroll_left: f32,
    client_width: f32,
    scroll_width: f32,
) -> Option<ScrollbarThumb> {
    if !scroll_left.is_finite()
        || !client_width.is_finite()
        || !scroll_width.is_finite()
        || client_width <= 0.
        || scroll_width <= client_width
    {
        return None;
    }
    let width = ((client_width * client_width) / scroll_width).max(THUMB_MIN);
    let range = scroll_width - client_width;
    let left = ((client_width - width) * scroll_left.clamp(0., range) / range).round();
    Some(ScrollbarThumb {
        left,
        width: width.round(),
    })
}

pub fn scrollbar_drag_left(start_left: f32, delta_x: f32, ratio: f32, maximum: f32) -> f32 {
    (start_left + delta_x * ratio).clamp(0., maximum.max(0.))
}

fn sample_channel(
    slot: &Cell<Option<Channel>>,
    resting: f32,
    target: f32,
    now: Instant,
    duration: Duration,
    reduced_motion: bool,
) -> (f32, bool) {
    if reduced_motion || duration.is_zero() {
        slot.set(Some(Channel {
            from: target,
            to: target,
            started: now,
        }));
        return (target, false);
    }
    let channel = match slot.get() {
        Some(channel) if channel.to == target => channel,
        Some(channel) => {
            let current = channel_value(channel, now, duration);
            let next = Channel {
                from: current,
                to: target,
                started: now,
            };
            slot.set(Some(next));
            next
        }
        None => {
            let next = if resting == target {
                Channel {
                    from: target,
                    to: target,
                    started: now,
                }
            } else {
                Channel {
                    from: resting,
                    to: target,
                    started: now,
                }
            };
            slot.set(Some(next));
            next
        }
    };
    let value = channel_value(channel, now, duration);
    (value, value != channel.to)
}

fn channel_value(channel: Channel, now: Instant, duration: Duration) -> f32 {
    if duration.is_zero() {
        return channel.to;
    }
    let elapsed = now.saturating_duration_since(channel.started);
    if elapsed >= duration {
        return channel.to;
    }
    let linear = elapsed.as_secs_f32() / duration.as_secs_f32();
    let progress = workflow_ease(linear).clamp(0., 1.);
    channel.from + (channel.to - channel.from) * progress
}

fn workflow_ease(progress: f32) -> f32 {
    gpui_kit::base::animation::cubic_bezier(0.22, 0.61, 0.36, 1.)(progress)
}

/// A click on the uncovered track pages one viewport. Thumb drags use
/// `scrollbar_drag_left` instead of this requested-offset mapping.
pub fn scrollbar_target(
    dragging: bool,
    current: f32,
    requested: f32,
    width: f32,
    maximum: f32,
) -> f32 {
    let target = if dragging {
        requested
    } else if requested > current {
        current + width
    } else if requested < current {
        current - width
    } else {
        current
    };
    target.clamp(0., maximum.max(0.))
}

/// The reference timeline is `overflow-x-auto overflow-y-hidden`: only a
/// wheel gesture with a horizontal component belongs to the timeline.
pub fn timeline_consumes_wheel(horizontal_range: bool, delta_x: f32) -> bool {
    horizontal_range && delta_x.is_finite() && delta_x != 0.
}

pub const PITCH: f32 = 192.;
pub const STATION_WIDTH: f32 = 168.;
pub const FADE: f32 = 40.;

/// Width of the horizontal station row, including the 24px gap between stations.
/// Keeping this explicit prevents GPUI's flex measurement from shrinking a row
/// that is meant to establish the scrollable content width.
pub fn timeline_content_width(stage_count: usize) -> f32 {
    if stage_count == 0 {
        0.
    } else {
        STATION_WIDTH + (stage_count.saturating_sub(1) as f32 * PITCH)
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Fold {
    pub left: Vec<usize>,
    pub right: Vec<usize>,
}

pub fn ledge_width(count: usize) -> f32 {
    if count == 0 {
        return 0.;
    }
    16. + count.min(3) as f32 * 16. - 6. + if count > 3 { 26. } else { 0. }
}

/// The left stub stops 6px before the next lamp; the right one starts at
/// the preceding station's slot end so it never crosses its heading.
pub fn ledge_stub_width(indexes: &[usize], left: bool, offset: f32, width: f32) -> f32 {
    if indexes.is_empty() || !offset.is_finite() || !width.is_finite() || width <= 0. {
        return 0.;
    }
    let inner = ledge_width(indexes.len()) - 8.;
    let stub = if left {
        let open = *indexes.last().expect("nonempty indexes") as f32 + 1.;
        open * PITCH + 17. - offset - 6. - inner
    } else {
        let open = indexes[0] as f32 - 1.;
        width - inner - (open * PITCH + STATION_WIDTH - offset)
    };
    if stub < 6. {
        0.
    } else {
        stub.round()
    }
}

pub fn fold_stations(count: usize, offset: f32, width: f32, keep: Option<usize>) -> Fold {
    let mut fold = Fold::default();
    if width <= 0. || !width.is_finite() || !offset.is_finite() {
        return fold;
    }
    for _ in 0..8 {
        let left_edge = if fold.left.is_empty() {
            0.
        } else {
            ledge_width(fold.left.len()) + FADE
        };
        let right_edge = width
            - if fold.right.is_empty() {
                0.
            } else {
                ledge_width(fold.right.len()) + FADE
            };
        let mut next = Fold::default();
        for index in 0..count {
            let x = index as f32 * PITCH + 17. - offset;
            if x < left_edge && Some(index) != keep {
                next.left.push(index);
            } else if x > right_edge && Some(index) != keep {
                next.right.push(index);
            }
        }
        if next == fold {
            break;
        }
        fold = next;
    }
    fold
}

pub fn camera_left(index: usize, width: f32, maximum: f32) -> f32 {
    (index as f32 * PITCH - (width - STATION_WIDTH) / 2.).clamp(0., maximum.max(0.))
}

/// Stage/ledge focus uses one station pitch per arrow, as in WorkflowTimeline.
pub fn keyboard_stage_offset(key: &str, current: f32, maximum: f32) -> Option<f32> {
    if maximum <= 0. {
        return None;
    }
    let delta = match key {
        "left" => -PITCH,
        "right" => PITCH,
        _ => return None,
    };
    Some((current + delta).clamp(0., maximum))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_wide_and_narrow_viewports() {
        assert_eq!(timeline_content_width(0), 0.);
        assert_eq!(timeline_content_width(1), 168.);
        assert_eq!(timeline_content_width(5), 936.);
        assert_eq!(fold_stations(5, 0., 930., None), Fold::default());
        assert_eq!(
            fold_stations(5, 0., 418., None),
            Fold {
                left: vec![],
                right: vec![2, 3, 4]
            }
        );
        assert_eq!(
            fold_stations(5, 518., 418., None),
            Fold {
                left: vec![0, 1, 2, 3],
                right: vec![]
            }
        );
        assert_eq!(fold_stations(5, 0., 0., None), Fold::default());
    }

    #[test]
    fn edge_widths_are_bounded_and_camera_can_reach_hidden_stations() {
        assert_eq!(ledge_width(0), 0.);
        assert_eq!(ledge_width(3), 58.);
        assert_eq!(ledge_width(30), 84.);
        assert_eq!(camera_left(0, 418., 518.), 0.);
        assert_eq!(camera_left(4, 418., 518.), 518.);
        assert_eq!(camera_left(2, 418., 518.), 259.);
        assert_eq!(
            fold_stations(5, 259., 418., None),
            Fold {
                left: vec![0, 1],
                right: vec![4]
            }
        );
        assert_eq!(
            fold_stations(5, 518., 418., Some(4)),
            Fold {
                left: vec![0, 1, 2, 3],
                right: vec![]
            }
        );
    }

    #[test]
    fn ledge_stubs_connect_to_lamps_on_the_left_and_slot_ends_on_the_right() {
        assert_eq!(ledge_stub_width(&[], true, 259., 418.), 0.);
        assert_eq!(ledge_stub_width(&[0, 1], true, 259., 418.), 102.);
        assert_eq!(ledge_stub_width(&[4], false, 400., 418.), 56.);
        assert_eq!(ledge_stub_width(&[2, 3, 4], false, 0., 418.), 8.);
        assert_eq!(ledge_stub_width(&[0, 1, 2, 3], true, 518., 418.), 185.);
        assert_eq!(ledge_stub_width(&[4], false, 349., 418.), 0.);
        assert_eq!(ledge_stub_width(&[4], false, 350., 418.), 6.);
        assert_eq!(ledge_stub_width(&[4], false, 0., 0.), 0.);
        assert_eq!(ledge_stub_width(&[4], false, f32::NAN, 418.), 0.);
    }

    #[test]
    fn scrollbar_click_pages_but_drag_uses_the_requested_offset() {
        assert_eq!(scrollbar_target(false, 0., 1., 418., 518.), 418.);
        assert_eq!(scrollbar_target(false, 418., 1., 418., 518.), 0.);
        assert_eq!(scrollbar_target(false, 418., 520., 418., 518.), 518.);
        assert_eq!(scrollbar_target(true, 418., 260., 418., 518.), 260.);
        assert_eq!(scrollbar_target(true, 418., 999., 418., 518.), 518.);
    }

    #[test]
    fn touch_thumb_requires_the_scrollbar_y_band() {
        let viewport = Viewport::default();
        assert!(!viewport.bar_contains_y(240.));
        viewport.set_bar_bounds(100., 240., 4.);
        assert!(viewport.bar_contains_y(240.));
        assert!(viewport.bar_contains_y(244.));
        assert!(!viewport.bar_contains_y(239.));
        assert!(!viewport.bar_contains_y(245.));
        assert!(!viewport.bar_contains_y(f32::NAN));
        // Post-layout measurement must replace stale geometry after resize.
        viewport.set_bar_bounds(100., 400., 2.);
        assert!(!viewport.bar_contains_y(240.));
        assert!(viewport.bar_contains_y(401.));
    }

    #[test]
    fn timeline_wheel_only_claims_horizontal_gestures() {
        assert!(!timeline_consumes_wheel(false, 12.));
        assert!(!timeline_consumes_wheel(true, 0.));
        assert!(!timeline_consumes_wheel(true, -0.));
        assert!(timeline_consumes_wheel(true, 12.));
        assert!(timeline_consumes_wheel(true, -12.));
        assert!(timeline_consumes_wheel(true, 0.0001));
        assert!(!timeline_consumes_wheel(true, f32::NAN));
        assert!(!timeline_consumes_wheel(true, f32::INFINITY));
    }

    #[test]
    fn keyboard_stage_navigation_clamps_and_leaves_other_keys_to_their_owner() {
        assert_eq!(keyboard_stage_offset("right", 0., 518.), Some(192.));
        assert_eq!(keyboard_stage_offset("right", 450., 518.), Some(518.));
        assert_eq!(keyboard_stage_offset("left", 30., 518.), Some(0.));
        assert_eq!(keyboard_stage_offset("left", 518., 518.), Some(326.));
        assert_eq!(keyboard_stage_offset("enter", 0., 518.), None);
        assert_eq!(keyboard_stage_offset("right", 0., 0.), None);
    }

    #[test]
    fn camera_waits_for_layout_and_manual_input_cancels_pending_centering() {
        let viewport = Viewport::default();
        viewport.prepare_camera(Some(2));
        assert_eq!(viewport.camera_keep_index(), Some(2));
        assert_eq!(viewport.camera_target(0., 0.), None);
        assert_eq!(viewport.camera_target(418., 518.), Some((2, 259.)));
        assert_eq!(viewport.camera_keep_index(), None);
        assert_eq!(viewport.camera_target(418., 518.), None);
        viewport.prepare_camera(Some(4));
        viewport.mark_user_interaction();
        assert_eq!(viewport.camera_keep_index(), None);
        viewport.prepare_camera(Some(4));
        assert_eq!(viewport.camera_target(418., 518.), None);
        viewport.prepare_camera(Some(3));
        assert_eq!(viewport.camera_target(418., 518.), Some((3, 451.)));
        viewport.prepare_camera(None);
        assert_eq!(viewport.camera_target(418., 518.), None);
    }

    #[test]
    fn camera_keep_index_lives_only_for_the_programmatic_flight() {
        let viewport = Viewport::default();
        let now = std::time::Instant::now();

        viewport.scroll.set_offset(point(px(-518.), px(0.)));

        assert!(viewport.scroll_to_stage(4, 0., false, now));
        assert_eq!(viewport.camera_keep_index(), Some(4));
        assert!(viewport.scroll_is_moving());

        assert!(viewport.advance_scroll(518., false, now + SCROLL_TRAVEL / 2));
        assert_eq!(viewport.camera_keep_index(), Some(4));
        assert!(viewport.scroll_is_moving());

        viewport.advance_scroll(518., false, now + SCROLL_TRAVEL);
        assert_eq!(viewport.camera_keep_index(), None);
        assert!(!viewport.scroll_is_moving());
        assert_eq!(viewport.scroll.offset().x, px(0.));
    }

    #[test]
    fn camera_keep_index_clears_when_manual_input_takes_over() {
        let viewport = Viewport::default();
        let now = std::time::Instant::now();

        viewport.scroll.set_offset(point(px(-451.), px(0.)));
        assert!(viewport.scroll_to_stage(3, 0., false, now));
        assert_eq!(viewport.camera_keep_index(), Some(3));
        viewport.mark_user_interaction();
        assert_eq!(viewport.camera_keep_index(), None);
        assert!(!viewport.scroll_is_moving());
    }

    #[test]
    fn reduced_motion_lands_immediately_without_retaining_camera_keep() {
        let viewport = Viewport::default();
        let now = std::time::Instant::now();

        viewport.scroll.set_offset(point(px(-259.), px(0.)));
        assert!(viewport.scroll_to_stage(2, 0., true, now));
        assert_eq!(viewport.camera_keep_index(), None);
        assert!(!viewport.scroll_is_moving());
        assert_eq!(viewport.scroll.offset().x, px(0.));
    }

    #[test]
    fn camera_keep_clears_when_reduced_motion_is_enabled_during_travel() {
        let viewport = Viewport::default();
        let now = std::time::Instant::now();
        viewport.scroll.set_offset(point(px(-259.), px(0.)));
        viewport.scroll_to_stage(0, 0., false, now);
        assert_eq!(viewport.camera_keep_index(), Some(0));

        assert!(viewport.advance_scroll(518., true, now + SCROLL_TRAVEL / 2));
        assert_eq!(viewport.scroll.offset().x, px(0.));
        assert_eq!(viewport.camera_keep_index(), None);
        assert!(!viewport.scroll_is_moving());
    }

    #[test]
    fn completed_camera_notifies_even_when_its_offset_is_already_at_target() {
        let viewport = Viewport::default();
        let now = std::time::Instant::now();
        viewport.scroll_motion.set(Some(Channel {
            from: 0.,
            to: 0.,
            started: now,
        }));
        viewport.scroll_motion_index.set(Some(2));

        assert!(viewport.advance_scroll(500., false, now + SCROLL_TRAVEL));
        assert_eq!(viewport.camera_keep_index(), None);
        assert!(!viewport.scroll_is_moving());
        assert!(!viewport.advance_scroll(500., false, now + SCROLL_TRAVEL));
    }

    #[test]
    fn losing_overflow_clears_camera_keep_and_clamps_the_offset() {
        let viewport = Viewport::default();
        let now = std::time::Instant::now();
        viewport.scroll.set_offset(point(px(-259.), px(0.)));
        viewport.scroll_to_stage(0, 0., false, now);

        assert!(viewport.advance_scroll(0., false, now + SCROLL_TRAVEL / 2));
        assert_eq!(viewport.scroll.offset().x, px(0.));
        assert_eq!(viewport.camera_keep_index(), None);
        assert!(!viewport.scroll_is_moving());
    }

    #[test]
    fn programmatic_scroll_frames_finish_clamp_and_respect_reduced_motion() {
        let now = std::time::Instant::now();
        let channel = Channel {
            from: 0.,
            to: 300.,
            started: now,
        };
        assert_eq!(scroll_motion_frame(channel, 500., false, now), (0., true));
        let mid = scroll_motion_frame(channel, 500., false, now + SCROLL_TRAVEL / 2);
        assert!(mid.0 > 0. && mid.0 < 300. && mid.1);
        assert_eq!(
            scroll_motion_frame(channel, 500., false, now + SCROLL_TRAVEL),
            (300., false)
        );
        assert_eq!(scroll_motion_frame(channel, 500., true, now), (300., false));
        assert_eq!(
            scroll_motion_frame(channel, 50., false, now + SCROLL_TRAVEL),
            (50., false)
        );
        assert_eq!(scroll_motion_frame(channel, 0., false, now), (0., false));
    }

    #[test]
    fn manual_scroll_and_drag_cancel_shared_camera_motion_without_a_jump() {
        let viewport = Viewport::default();
        let next_render = viewport.clone();
        let now = std::time::Instant::now();
        let channel = Channel {
            from: 0.,
            to: 300.,
            started: now,
        };
        viewport.prepare_camera(Some(2));
        viewport.scroll_motion.set(Some(channel));
        assert!(next_render.advance_scroll(500., false, now + SCROLL_TRAVEL / 2));
        let position = viewport.scroll.offset();
        assert!(position.x < px(0.) && position.x > px(-300.));
        next_render.mark_user_interaction();
        assert!(!viewport.scroll_is_moving());
        assert!(!viewport.advance_scroll(500., false, now + SCROLL_TRAVEL));
        assert_eq!(viewport.scroll.offset(), position);
        assert_eq!(viewport.camera_target(418., 518.), None);

        viewport.scroll_motion.set(Some(channel));
        viewport.presentation(-f32::from(position.x), 418., 936., false, now);
        next_render.begin_thumb_drag(100.);
        assert!(!viewport.scroll_is_moving());
        assert!(viewport.dragging.get());
        assert_eq!(viewport.scroll.offset(), position);

        viewport.scroll_motion.set(Some(channel));
        assert!(next_render.advance_scroll(500., true, now));
        assert_eq!(viewport.scroll.offset().x, px(-300.));
        assert!(!viewport.scroll_is_moving());

        viewport.scroll_motion.set(Some(channel));
        next_render.reset_drag_if_invalid(418., 0.);
        assert!(!viewport.scroll_is_moving());
        assert!(!viewport.dragging.get());
    }

    #[test]
    fn scrollbar_thumb_matches_the_reference_ratio_and_hides_without_overflow() {
        assert_eq!(scrollbar_thumb(0., 418., 418.), None);
        assert_eq!(scrollbar_thumb(0., 0., 900.), None);
        assert_eq!(
            scrollbar_thumb(0., 418., 936.),
            Some(ScrollbarThumb {
                left: 0.,
                width: 187.
            })
        );
        assert_eq!(
            scrollbar_thumb(28., 418., 936.),
            Some(ScrollbarThumb {
                left: 13.,
                width: 187.
            })
        );
        assert_eq!(
            scrollbar_thumb(518., 418., 936.),
            Some(ScrollbarThumb {
                left: 231.,
                width: 187.
            })
        );
    }

    #[test]
    fn scrollbar_drag_follows_the_content_ratio() {
        let ratio = 936. / 418.;
        assert_eq!(scrollbar_drag_left(10., 20., ratio, 518.).round(), 55.);
        assert_eq!(scrollbar_drag_left(500., 40., ratio, 518.), 518.);
        assert_eq!(scrollbar_drag_left(4., -20., ratio, 518.), 0.);
    }

    #[test]
    fn scrollbar_visibility_follows_hover_drag_and_the_scroll_hold() {
        let now = std::time::Instant::now();
        assert!(!scrollbar_revealed(false, false, false, None, now));
        assert!(scrollbar_revealed(true, false, false, None, now));
        assert!(scrollbar_revealed(false, true, false, None, now));
        assert!(scrollbar_revealed(false, false, true, None, now));
        assert!(scrollbar_revealed(
            false,
            false,
            false,
            Some(now + SCROLLBAR_SCROLL_HOLD),
            now
        ));
        assert!(!scrollbar_revealed(false, false, false, Some(now), now));
        assert!(!scrollbar_thick(false, false));
        assert!(scrollbar_thick(true, false));
        assert!(scrollbar_thick(false, true));
    }

    #[test]
    fn scrollbar_fade_and_thickness_use_the_reference_timing() {
        let viewport = Viewport::default();
        let now = std::time::Instant::now();
        let resting = viewport.presentation(0., 418., 936., false, now);
        assert_eq!(resting.opacity, 0.);
        assert_eq!(resting.thickness, 2.);
        assert!(!viewport.motion_is_running());

        viewport.set_hovered(true);
        let started = viewport.presentation(0., 418., 936., false, now);
        assert_eq!(started.opacity, 0.);
        assert!(viewport.motion_is_running());
        let mid = viewport.presentation(
            0.,
            418.,
            936.,
            false,
            now + std::time::Duration::from_millis(80),
        );
        assert!(mid.opacity > 0.5 && mid.opacity < 1., "{}", mid.opacity);
        let done = viewport.presentation(0., 418., 936., false, now + SCROLLBAR_FADE);
        assert_eq!(done.opacity, 1.);
        assert!(!viewport.motion_is_running());

        assert!(viewport.set_bar_hovered(true));
        let thickening = viewport.presentation(0., 418., 936., false, now + SCROLLBAR_FADE);
        assert_eq!(thickening.thickness, 2.);
        let thick = viewport.presentation(
            0.,
            418.,
            936.,
            false,
            now + SCROLLBAR_FADE + SCROLLBAR_THICKEN,
        );
        assert_eq!(thick.thickness, 4.);

        let snapped = viewport.presentation(
            0.,
            418.,
            936.,
            true,
            now + SCROLLBAR_FADE + SCROLLBAR_THICKEN,
        );
        assert_eq!(snapped.opacity, 1.);
        assert_eq!(snapped.thickness, 4.);
        assert!(!viewport.motion_is_running());
    }

    #[test]
    fn scroll_hold_ignores_the_first_sample_and_later_repeats() {
        let viewport = Viewport::default();
        let now = std::time::Instant::now();
        assert_eq!(viewport.note_offset(0., now), None);
        assert_eq!(viewport.note_offset(0.4, now), None);
        let deadline = viewport.note_offset(12., now).expect("movement holds");
        assert_eq!(deadline, now + SCROLLBAR_SCROLL_HOLD);
        assert_eq!(viewport.note_offset(12.2, now), None);
        assert!(scrollbar_revealed(
            false,
            false,
            false,
            viewport.scrolling_deadline(),
            now
        ));
    }

    #[test]
    fn drag_cleanup_is_shared_across_render_clones() {
        let viewport = Viewport::default();
        let next_render = viewport.clone();
        viewport.start_drag();
        assert!(next_render.dragging.get());
        next_render.reset_drag_if_invalid(418., 0.);
        assert!(!viewport.dragging.get());
        viewport.start_drag();
        next_render.begin_pointer_sequence();
        assert!(!viewport.dragging.get());
        next_render.start_drag();
        viewport.end_drag();
        assert!(!next_render.dragging.get());
    }
}
