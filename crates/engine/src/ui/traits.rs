use std::{cell::RefCell, rc::{Rc, Weak}};

use glam::Vec2;

use crate::ui::layout::Layout;

pub trait Element {
    fn render(&self, engine: &mut crate::UIRenderer, abs_pos: Vec2);
    fn hit_test(&self, point: Vec2, abs_pos: Vec2, size: Vec2) -> bool {
        point.x >= abs_pos.x && point.x <= abs_pos.x + size.x &&
        point.y >= abs_pos.y && point.y <= abs_pos.y + size.y
    }
}