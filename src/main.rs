mod camera;
mod gpu;
mod material;
mod model;
mod input;
mod instance;
mod lighting;
mod scene;
mod state;
mod texture;
mod vertex;

// ---> Intern dependencies:
use state::State;

// ---> Extern dependencies:
//use wgpu::util::DeviceExt;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop:: EventLoop;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::window::Window;
use winit::window::WindowId;
use winit::keyboard::KeyCode;
use std::sync::Arc;


///// APP STRUCTURE ////////////////////////////////////////////////////////////////////////////////
#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // ---> Create a window:
        let window = Arc::new(event_loop.create_window(
                Window::default_attributes().with_title("SealEngine v0.1 (alpha)")
        ).unwrap());
        
        // ---> Async initialization for wgpu:
        let state = pollster::block_on(State::new(&window.clone()));
        self.window = Some(window.clone());
        self.state = Some(state);
    }

    fn window_event(&mut self, 
                    event_loop: &ActiveEventLoop, 
                    _window_id: WindowId, 
                    event: WindowEvent) {
        if let Some(state) = &mut self.state {
            let input_consumed = state.handle_input(&event);

            if !input_consumed {
                match event {
                    WindowEvent::RedrawRequested => {
                        if let Some(state) = &mut self.state {
                            // ---> Update before rendering:
                            state.update();
                            
                            match state.render() {
                                Ok(_) => {
                                    if let Some(window) = &self.window {
                                        window.request_redraw();
                                    }
                                }
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    // ---> Reconfigure surface:
                                    state.resize(state.size);
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    eprintln!("Out of memory!");
                                    event_loop.exit();
                                }
                                Err(wgpu::SurfaceError::Timeout) => {
                                    eprintln!("Surface timeout!");
                                }
                                Err(e) => {
                                    eprintln!("Render error: {:?}", e);
                                }
                            }
                        }
                    }
                    WindowEvent::Resized(new_size) => {
                        if let Some(state) = self.state.as_mut() {
                            state.resize(new_size);
                        }
                    }
                    WindowEvent::CloseRequested => {
                        event_loop.exit();
                    }
                    _ => {}
                }
            }
        }

        // ---> ESC-Key to close (special case):
        if let WindowEvent::KeyboardInput { event: key_event, .. } = &event {
            if let winit::keyboard::PhysicalKey::Code(KeyCode::Escape) = key_event.physical_key {
                if key_event.state == winit::event::ElementState::Pressed {
                    event_loop.exit();
                }
            }
        }
    }
}
///// APP STRUCTURE ////////////////////////////////////////////////////////////////////////////////

///// MAIN PROGRAM /////////////////////////////////////////////////////////////////////////////////
fn main() {
    let event_loop = EventLoop::new().unwrap();
    
    let mut app = App::default();
    
    event_loop.run_app(&mut app).unwrap();
}
///// MAIN PROGRAM /////////////////////////////////////////////////////////////////////////////////
