use winit::window::Window;

pub struct CursorManager {
    is_locked: bool,
}

impl CursorManager {
    pub fn new() -> Self {
        Self { is_locked: true } // Начинаем с заблокированным курсором
    }

    pub fn lock_cursor(&mut self, window: &Window) {
        if !self.is_locked {
            let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Confined);
            window.set_cursor_visible(false);
            self.is_locked = true;
        }
    }

    pub fn unlock_cursor(&mut self, window: &Window) {
        if self.is_locked {
            let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            window.set_cursor_visible(true);
            self.is_locked = false;
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