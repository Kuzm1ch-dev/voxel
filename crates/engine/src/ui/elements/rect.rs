use glam::{Vec2, Vec4};
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::ui::elements::{BaseElement, BaseElementBuilder};
use crate::{render::ui::UIVertex, ui::traits::Element};
use crate::ui::layout::Layout;
pub struct Rect {
    pub base: BaseElement,
}

impl Rect {
    pub fn builder(id: &str) -> RectBuilder {
        RectBuilder::new(id)
    }
}

pub struct RectBuilder {
    pub base: BaseElementBuilder,
}

impl RectBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            base: BaseElementBuilder::new(id),
        }
    }
    
    pub fn with_base<F>(mut self, f: F) -> Self 
    where
        F: FnOnce(BaseElementBuilder) -> BaseElementBuilder
    {
        self.base = f(self.base);
        self
    }

    pub fn build(self) -> Rect {
        Rect {
            base: self.base.build(),
        }
    }
}

impl Element for Rect {
    fn render(&self, renderer: &mut crate::UIRenderer, pos: Vec2) {
        if !self.base.visible { return; }
        
        let abs_pos = pos + self.base.position;
        
        renderer.render_rect(abs_pos, self.base.size, self.base.color);
    }
}