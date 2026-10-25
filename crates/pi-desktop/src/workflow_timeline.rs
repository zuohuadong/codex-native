//! Viewport geometry adapted from ZCode 3.16.2 timeline-ledge.ts (Apache-2.0).
//! These coordinates are logical pixels, independent of native display scale.

use gpui_kit as gpui;
use gpui_kit::base::ScrollbarHandle;
use gpui_kit::{point, px, Bounds, Pixels, Point, ScrollHandle, Size};
use std::{cell::Cell, rc::Rc};

/// State shared by the timeline, its camera, and the native scrollbar across renders.
#[derive(Clone, Default)]
pub struct Viewport {
    pub scroll: ScrollHandle,
    pub camera: Rc<Cell<Option<usize>>>,
    requested_camera: Rc<Cell<Option<usize>>>,
    dragging: Rc<Cell<bool>>,
    hovered: Rc<Cell<bool>>,
}

impl Viewport {
    pub fn scrollbar(
        &self,
        id: impl Into<gpui_kit::ElementId>,
        c: &crate::theme::ThemeColors,
    ) -> gpui_kit::base::Scrollbar {
        use gpui_kit::{px, rgba};
        gpui_kit::base::Scrollbar::horizontal(self)
            .id(id)
            .mode(if self.hovered.get() {
                gpui_kit::base::ScrollbarMode::Always
            } else {
                gpui_kit::base::ScrollbarMode::Hover
            })
            .viewport_from_layout()
            .styles(|styles| {
                styles
                    .track(|track| {
                        track
                            .width(px(2.))
                            .bg(rgba(c.settings_border).into())
                            .border_color(gpui::transparent_black())
                    })
                    .track_hover(|track| {
                        track
                            .width(px(4.))
                            .bg(rgba(c.settings_border).into())
                            .border_color(gpui::transparent_black())
                    })
                    .track_active(|track| {
                        track
                            .width(px(4.))
                            .bg(rgba(c.settings_border).into())
                            .border_color(gpui::transparent_black())
                    })
                    .thumb_active(|thumb| {
                        thumb
                            .width(px(4.))
                            .inset(px(0.))
                            .radius(px(2.))
                            .min_length(px(24.))
                            .bg(rgba(c.foreground_subtlest))
                    })
                    .thumb(|thumb| {
                        thumb
                            .width(px(2.))
                            .inset(px(0.))
                            .radius(px(2.))
                            .min_length(px(24.))
                            .bg(rgba(c.foreground_subtlest))
                    })
                    .thumb_hover(|thumb| {
                        thumb
                            .width(px(4.))
                            .inset(px(0.))
                            .radius(px(2.))
                            .min_length(px(24.))
                            .bg(rgba(c.foreground_subtlest))
                    })
            })
    }
}

impl ScrollbarHandle for Viewport {
    fn viewport_bounds(&self) -> Bounds<Pixels> {
        self.scroll.bounds()
    }
    fn offset(&self) -> Point<Pixels> {
        self.scroll.offset()
    }
    fn content_size(&self) -> Size<Pixels> {
        (self.scroll.max_offset() + self.scroll.bounds().size.into()).into()
    }
    fn start_drag(&self) {
        self.dragging.set(true);
    }
    fn end_drag(&self) {
        self.dragging.set(false);
    }
    fn set_offset(&self, requested: Point<Pixels>) {
        self.mark_user_interaction();
        let offset = self.scroll.offset();
        let current = -f32::from(offset.x);
        let target = scrollbar_target(
            self.dragging.get(),
            current,
            -f32::from(requested.x),
            f32::from(self.scroll.bounds().size.width),
            f32::from(self.scroll.max_offset().x),
        );
        self.scroll.set_offset(point(px(-target), offset.y));
    }
}

impl Viewport {
    pub fn set_hovered(&self, hovered: bool) {
        self.hovered.set(hovered);
    }

    pub fn mark_user_interaction(&self) {
        // A manual gesture settles the pending camera request without moving.
        self.camera.set(self.requested_camera.get());
    }

    pub fn reset_drag_if_invalid(&self, width: f32, maximum: f32) {
        if width <= 0. || maximum <= 0. {
            self.dragging.set(false);
        }
    }

    pub fn prepare_camera(&self, running_index: Option<usize>) {
        self.requested_camera.set(running_index);
    }

    pub fn camera_target(&self, width: f32, maximum: f32) -> Option<f32> {
        if width <= 0. {
            return None;
        }
        let requested = self.requested_camera.get();
        if self.camera.replace(requested) == requested {
            return None;
        }
        requested.map(|index| camera_left(index, width, maximum))
    }

    /// A new pointer sequence cannot inherit an abandoned drag from a previous
    /// mount. The base scrollbar may call start_drag again in the bubble phase.
    pub fn begin_pointer_sequence(&self) {
        self.dragging.set(false);
    }
}

/// Drag follows the pointer; a click on the uncovered track pages one viewport.
/// The base scrollbar calls start/end_drag around both mouse and touch drags.
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

pub const PITCH: f32 = 192.;
pub const STATION_WIDTH: f32 = 168.;
pub const FADE: f32 = 40.;

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

pub fn fold_stations(count: usize, offset: f32, width: f32) -> Fold {
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
            if x < left_edge {
                next.left.push(index);
            } else if x > right_edge {
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
        assert_eq!(fold_stations(5, 0., 930.), Fold::default());
        assert_eq!(
            fold_stations(5, 0., 418.),
            Fold {
                left: vec![],
                right: vec![2, 3, 4]
            }
        );
        assert_eq!(
            fold_stations(5, 518., 418.),
            Fold {
                left: vec![0, 1, 2, 3],
                right: vec![]
            }
        );
        assert_eq!(fold_stations(5, 0., 0.), Fold::default());
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
            fold_stations(5, 259., 418.),
            Fold {
                left: vec![0, 1],
                right: vec![4]
            }
        );
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
        assert_eq!(viewport.camera_target(0., 0.), None);
        assert_eq!(viewport.camera_target(418., 518.), Some(259.));
        assert_eq!(viewport.camera_target(418., 518.), None);
        viewport.prepare_camera(Some(4));
        viewport.mark_user_interaction();
        viewport.prepare_camera(Some(4));
        assert_eq!(viewport.camera_target(418., 518.), None);
        viewport.prepare_camera(Some(3));
        assert_eq!(viewport.camera_target(418., 518.), Some(451.));
        viewport.prepare_camera(None);
        assert_eq!(viewport.camera_target(418., 518.), None);
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
