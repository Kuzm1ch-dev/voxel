use winit::window::Window;
use crate::logger::Logger;

pub struct CursorManager {
    is_locked: bool,
}

impl CursorManager {
    pub fn new() -> Self {
        Self { is_locked: false }
    }

    pub fn lock_cursor(&mut self, window: &Window) {
        if !self.is_locked {
            match window.set_cursor_grab(winit::window::CursorGrabMode::Confined) {
                Ok(_) => {
                    window.set_cursor_visible(false);
                    self.is_locked = true;
                    Logger::info("Cursor locked");
                }
                Err(e) => {
                    Logger::warn(&format!("Failed to lock cursor: {}", e));
                }
            }
        }
    }

    pub fn unlock_cursor(&mut self, window: &Window) {
        if self.is_locked {
            match window.set_cursor_grab(winit::window::CursorGrabMode::None) {
                Ok(_) => {
                    window.set_cursor_visible(true);
                    self.is_locked = false;
                    Logger::info("Cursor unlocked");
                }
                Err(e) => {
                    Logger::warn(&format!("Failed to unlock cursor: {}", e));
                }
            }
        }
    }

    pub fn toggle_cursor(&mut self, window: &Window) {
        if self.is_locked {
            self.unlock_cursor(window);
        } else {
            self.lock_cursor(window);
        }
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }
}