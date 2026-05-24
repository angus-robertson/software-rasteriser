use std::sync::Arc;
use std::time::{Duration, Instant};

use pixels::{Pixels, SurfaceTexture};
use pixels::wgpu::Color;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

use renderer::{DrawCommand, RenderTarget, Renderer, SoftwareRenderer, math::Vec2, CullMode, Viewport, Framebuffer, Mesh, Vertex, Camera};
use renderer::math::{Matrix4, Vec3, Vec4};

const SIZE: i32 = 20;
const STEP: f32 = 1.0;
fn main() -> anyhow::Result<()> {
    let model = Mesh::from_file("assets/cube.obj")?;

    println!("{:?}", model);


    let world = World {
        camera: Camera {
            position: Vec3::new(-8.449, -10.032, 8.0),
            yaw: 2.3249977,
            pitch: 0.77900016,
            fov: 60.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
        },
        floor:  Mesh {
            positions: vec![
                Vertex { position: Vec3::new(-5.0, 0.0, -5.0) },
                Vertex { position: Vec3::new( 5.0, 0.0, -5.0) },
                Vertex { position: Vec3::new( 5.0, 0.0,  5.0) },
                Vertex { position: Vec3::new(-5.0, 0.0,  5.0) },
            ],
            indices: vec![
                0, 1, 2,
                0, 2, 3,
            ],
        },
        models: vec![model],
    };

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(world);
    event_loop.run_app(&mut app)?;

    Ok(())
}

const INITIAL_WIDTH: u32 = 765;
const INITIAL_HEIGHT: u32 = 503;



struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    renderer: SoftwareRenderer,
    viewport: Viewport,

    world: World,
    time: f32,

    input: InputState,
    last_mouse: Option<(f32, f32)>,

    last_frame: Instant,
    frame_count: f32,
    fps: f32,
    fps_text: [u8; 16],
    fps_len: usize,
}

impl App {
    fn new(world: World) -> Self {
        Self {
            window: None,
            pixels: None,
            renderer: SoftwareRenderer::new(),
            viewport: Viewport::new(0, INITIAL_WIDTH as i32, 0, INITIAL_HEIGHT as i32),
            world,
            time: 0.0,
            input: InputState::default(),
            last_mouse: None,
            last_frame: Instant::now(),
            frame_count: 0.0,
            fps: 0.0,
            fps_text: [0; 16],
            fps_len: 0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(Window::default_attributes().with_inner_size(LogicalSize::new(self.viewport.width(), self.viewport.height())).with_resizable(false)).expect("failed to create window"));

        let surface_texture = SurfaceTexture::new(self.viewport.width(), self.viewport.height(), window.clone());
        let pixels = Pixels::new(self.viewport.width(), self.viewport.height(), surface_texture).expect("failed to create pixels");

        let size = window.inner_size();
        let center = winit::dpi::PhysicalPosition::new(size.width as f32 / 2.0, size.height as f32 / 2.0);

        window.set_cursor_visible(false);
        let _ = window.set_cursor_position(center);

        self.last_mouse = Some((center.x, center.y));
        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("window {:?} has received the signal to quit.", window_id);
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                // handle timing/fps
                let now = Instant::now();
                let dt = now.duration_since(self.last_frame);
                self.frame_count += 1.0;
                self.time += dt.as_secs_f32();

                if dt >= Duration::from_millis(100) {
                    self.fps = self.frame_count / dt.as_secs_f32();
                    self.frame_count = 0.0;
                    self.last_frame = now;

                    self.fps_len = {
                        let s = format!("FPS: {:.1}", self.fps);
                        let bytes = s.as_bytes();
                        let len = bytes.len().min(16);

                        self.fps_text[..len].copy_from_slice(&bytes[..len]);
                        len
                    }
                }

                // handle input
                self.world.update(&self.input, dt.as_secs_f32());
                self.input.mouse_dx = 0.0;
                self.input.mouse_dy = 0.0;

                if let Some(window) = &self.window {
                    let center = PhysicalPosition::new(
                        self.viewport.width() as f64 / 2.0,
                        self.viewport.height() as f64 / 2.0,
                    );

                    let _ = window.set_cursor_position(center);
                }

                // render
                let pixels = self.pixels.as_mut().unwrap();
                pixels.clear_color(Color::WHITE);
                let width = pixels.texture().width();
                let height = pixels.texture().height();

                let mut buffer = Framebuffer::new(pixels.frame_mut(), width, height);
                let mut frame = RenderTarget::new(&mut buffer, self.viewport);

                self.renderer.clear(&mut frame, [255,255,255, 255]);

                let cube_model =  Matrix4::translate(Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }) * Matrix4::rotate_zx(self.time) * Matrix4::rotate_xy(self.time * 1.61) * Matrix4::scale(0.3);


                self.renderer.draw(&mut frame, DrawCommand::Mesh {
                    mesh: &self.world.floor,
                    camera: &self.world.camera,
                    transform: Matrix4::identity(),
                    colour: [211, 211, 211, 255],
                    cull_mode: CullMode::None,
                });

                for mesh in &self.world.models {
                    self.renderer.draw(&mut frame, DrawCommand::Mesh {
                        mesh: &mesh,
                        camera: &self.world.camera,
                        transform: cube_model,
                        colour: [255, 0, 0, 255],
                        cull_mode: CullMode::Clockwise,
                    });
                }

                // draw FPS
                self.renderer.draw(&mut frame, DrawCommand::Text {
                    text: core::str::from_utf8(&self.fps_text[..self.fps_len]).unwrap(),
                    position: Vec2::new(5.0, 5.0),
                    colour: [0, 0, 0, 255]
                });

                pixels.render().expect("failed to render pixels");
                self.window.as_ref().unwrap().request_redraw();

                // println!("took {:?}ms to render frame", now.elapsed().as_millis());
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);

                let cx = self.viewport.width() as f32 * 0.5;
                let cy = self.viewport.height() as f32 * 0.5;

                self.input.mouse_dx += x - cx;
                self.input.mouse_dy += y - cy;

            }
            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;

                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(c) => {
                        match c {
                            KeyCode::KeyW => self.input.w = pressed,
                            KeyCode::KeyA => self.input.a = pressed,
                            KeyCode::KeyS  => self.input.s = pressed,
                            KeyCode::KeyD  => self.input.d = pressed,
                            KeyCode::Space => self.input.space = pressed,
                            KeyCode::ShiftLeft => self.input.shift = pressed,
                            KeyCode::Escape => event_loop.exit(),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn create_floor(size: i32, step: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let half = size as f32 * step;

    // lines along X
    for z in -size..=size {
        let z = z as f32 * step;

        positions.push(Vertex { position: Vec3::new(-half, 0.0, z) });
        positions.push(Vertex { position: Vec3::new( half, 0.0, z) });

        let base = positions.len() as usize;
        indices.push(base - 2);
        indices.push(base - 1);
    }

    // lines along Z
    for x in -size..=size {
        let x = x as f32 * step;

        positions.push(Vertex { position: Vec3::new(x, 0.0, -half) });
        positions.push(Vertex { position: Vec3::new(x, 0.0,  half) });

        let base = positions.len() as usize;
        indices.push(base - 2);
        indices.push(base - 1);
    }

    Mesh { positions, indices }
}


pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
}

pub struct InputState {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub space: bool,
    pub shift: bool,
    pub mouse_dx: f32,
    pub mouse_dy: f32,
}

impl Default for InputState {
    fn default() -> InputState {
        Self {
            w: false,
            a: false,
            s: false,
            d: false,
            space: false,
            shift: false,
            mouse_dx: 0.0,
            mouse_dy: 0.0,
        }
    }
}

pub struct World {
    camera: Camera,
    models: Vec<Mesh>,
    floor: Mesh,
}

impl World {
    pub fn update(&mut self, input: &InputState, dt: f32) {
        let sensitivity = 0.002;

        self.camera.yaw   += input.mouse_dx * sensitivity;
        self.camera.pitch -= input.mouse_dy * sensitivity;

        // clamp pitch
        self.camera.pitch = self.camera.pitch.clamp(-1.55, 1.55);

        let forward = self.camera.forward();
        let right = self.camera.right();

        let speed = 5.0 * dt;

        if input.w { self.camera.position += speed * forward; }
        if input.s { self.camera.position -= speed * forward; }
        if input.d { self.camera.position -= speed * right; }
        if input.a { self.camera.position += speed * right; }

        if input.shift {self.camera.position.y += speed }
        if input.space {self.camera.position.y -= speed }
    }
}