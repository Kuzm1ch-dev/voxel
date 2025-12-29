use glam::{Vec2, Vec4};
use crate::ui::elements::{BaseElement, BaseElementBuilder};
use crate::{ui::traits::Element};
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::ui::layout::Layout;

pub struct Text {
    pub base: BaseElement,
    text: String,
    scale: f32
}


impl Text {
    pub fn builder(id: &str) -> TextBuilder {
        TextBuilder::new(id)
    }
}

pub struct TextBuilder {
    pub base: BaseElementBuilder,
    text: String,
    scale: f32
}

impl TextBuilder {
    pub fn new(id: &str) -> Self {
        Self {
            base: BaseElementBuilder::new(id),
            text: "".to_string(),
            scale: 1.0
        }
    }

    pub fn with_base<F>(mut self, f: F) -> Self 
    where
        F: FnOnce(BaseElementBuilder) -> BaseElementBuilder
    {
        self.base = f(self.base);
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
    
    pub fn build(self) -> Text {
        Text {
            base: self.base.build(),
            text: self.text,
            scale: self.scale
        }
    }
}

impl Element for Text {
    fn render(&self, renderer: &mut crate::UIRenderer, pos: Vec2) {
        if !self.base.visible { return; }
        
        let abs_pos = pos + self.base.position;
    
        renderer.render_text( self.text.as_str(), abs_pos, self.scale, self.base.color);
    }
}