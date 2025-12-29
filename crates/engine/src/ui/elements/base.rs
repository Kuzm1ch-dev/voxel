use std::{cell::RefCell, rc::{Rc, Weak}};

use glam::{Vec2, Vec4};

use crate::ui::{layout::Layout, traits::Element};

pub struct BaseElement {
    pub id: String,
    pub parent: Option<Weak<RefCell<dyn Element>>>,
    pub children: Vec<Rc<RefCell<dyn Element>>>,
    pub layout: Layout,
    pub position: Vec2,
    pub size: Vec2,
    pub color: Vec4,
    pub visible: bool,
}

impl BaseElement {}

pub struct BaseElementBuilder {
    id: String,
    parent: Option<Weak<RefCell<dyn Element>>>,
    children: Vec<Rc<RefCell<dyn Element>>>,
    layout: Layout,
    position: Vec2,
    size: Vec2,
    color: Vec4,
    visible: bool
}

impl BaseElementBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            parent: None,
            children: Vec::new(),
            layout: Layout::default(),
            position: Vec2::ZERO,
            size: Vec2::ZERO,
            color: Vec4::ZERO,
            visible: true
        }
    }
    
    pub fn parent(mut self, parent: Option<&Rc<RefCell<dyn Element>>>) -> Self {
        self.parent = parent.map(|p| Rc::downgrade(p));
        self
    }
    
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
    
    pub fn position(mut self, position: Vec2) -> Self {
        self.position = position;
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_children(mut self, children: Vec<Rc<RefCell<dyn Element>>>) -> Self {
        self.children = children;
        self
    }
    
    pub fn build(self) -> BaseElement {
        BaseElement {
            id: self.id,
            parent: self.parent,
            children: self.children,
            layout: self.layout,
            position: self.position,
            size: self.size,
            color: self.color,
            visible: self.visible
        }
    }
}

