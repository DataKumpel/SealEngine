use std::collections::HashMap;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;
use winit::event::ElementState;
use winit::event::WindowEvent;
use winit::event::MouseButton;


///// INPUT STATE STRUCTURE ////////////////////////////////////////////////////////////////////////
#[derive(Default)]
pub struct InputState {
    keys: HashMap<KeyCode, bool>,
    keys_pressed: HashMap<KeyCode, bool>,
    keys_released: HashMap<KeyCode, bool>,

    mouse_buttons: HashMap<MouseButton, bool>,
    mouse_position: (f64, f64),
    mouse_delta: (f64, f64),
    mouse_wheel_delta: f32,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                // ---> Extract keycode from phys. key:
                if let PhysicalKey::Code(keycode) = key_event.physical_key {
                    let is_pressed = key_event.state == ElementState::Pressed;
                    let was_pressed = self.keys.get(&keycode).copied().unwrap_or(false);

                    self.keys.insert(keycode, is_pressed);

                    // ---> Track pressed event for this frame:
                    if is_pressed && !was_pressed {
                        self.keys_pressed.insert(keycode, true);
                    }

                    // ---> Track released event for this frame:
                    if !is_pressed && was_pressed {
                        self.keys_released.insert(keycode, true);
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let is_pressed = *state == ElementState::Pressed;
                self.mouse_buttons.insert(*button, is_pressed);
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_pos = (position.x, position.y);
                self.mouse_delta = (
                    new_pos.0 - self.mouse_position.0,
                    new_pos.1 - self.mouse_position.1,
                );
                self.mouse_position = new_pos;
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                        self.mouse_wheel_delta = *y;
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        self.mouse_wheel_delta = pos.y as f32 / 100.0;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn end_frame(&mut self) {
        // ---> Clear frame-specific data:
        self.keys_pressed.clear();
        self.keys_released.clear();
        self.mouse_delta = (0.0, 0.0);
        self.mouse_wheel_delta = 0.0;
    }

    //===== KEYBOARD QUERIES =======================================================================
    pub fn is_key_held(&self, key: KeyCode) -> bool {
        self.keys.get(&key).copied().unwrap_or(false)
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.get(&key).copied().unwrap_or(false)
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.keys_released.get(&key).copied().unwrap_or(false)
    }
    //===== KEYBOARD QUERIES =======================================================================

    //===== MOUSE STATE QUERIES ====================================================================
    pub fn is_mouse_button_held(&self, button: MouseButton) -> bool {
        self.mouse_buttons.get(&button).copied().unwrap_or(false)
    }

    pub fn mouse_position(&self) -> (f64, f64) {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    pub fn mouse_wheel_delta(&self) -> f32 {
        self.mouse_wheel_delta
    }
    //===== MOUSE STATE QUERIES ====================================================================
}
///// INPUT STATE STRUCTURE ////////////////////////////////////////////////////////////////////////

