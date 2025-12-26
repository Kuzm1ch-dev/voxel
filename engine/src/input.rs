#[derive(Debug, Clone)]
pub enum InputEvent {
    KeyPressed(winit::keyboard::KeyCode),
    KeyReleased(winit::keyboard::KeyCode),
    MouseButton(winit::event::MouseButton, winit::event::ElementState),
    MouseMotion(f32, f32),
}

pub fn process_winit_event(event: &winit::event::Event<()>) -> Option<InputEvent> {
    match event {
        winit::event::Event::WindowEvent { event, .. } => {
            match event {
                winit::event::WindowEvent::KeyboardInput { event, .. } => {
                    if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
                        match event.state {
                            winit::event::ElementState::Pressed => Some(InputEvent::KeyPressed(keycode)),
                            winit::event::ElementState::Released => Some(InputEvent::KeyReleased(keycode)),
                        }
                    } else {
                        None
                    }
                }
                winit::event::WindowEvent::MouseInput { button, state, .. } => {
                    Some(InputEvent::MouseButton(*button, *state))
                }
                _ => None
            }
        }
        winit::event::Event::DeviceEvent { event, .. } => {
            match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    Some(InputEvent::MouseMotion(delta.0 as f32, delta.1 as f32))
                }
                _ => None
            }
        }
        _ => None
    }
}