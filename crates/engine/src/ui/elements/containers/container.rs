use glam::Vec2;
use crate::ui::elements::{BaseElement, BaseElementBuilder};
use crate::ui::traits::Element;
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::ui::layout::Layout;

pub struct Container {
    pub base: BaseElement,
}

impl Container {
    pub fn builder(id: &str) -> ContainerBuilder {
        ContainerBuilder::new(id)
    }
}

pub struct ContainerBuilder {
    pub base: BaseElementBuilder,
}

impl ContainerBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            base: BaseElementBuilder::new(id),
        }
    }
    
    pub fn build(self) -> Container {
        Container {
            base: self.base.build(),
        }
    }
}

impl Element for Container {
    fn render(&self, renderer: &mut crate::UIRenderer, pos: Vec2) {
        if !self.base.visible { return; }
        
        let abs_pos = pos + self.base.position;
        
        for child in &self.base.children {
            if let Ok(child_ref) = child.try_borrow() {
                child_ref.render(renderer, abs_pos);
            } else {
                println!("Child is already borrowed mutably");
            }
        }
    }
}