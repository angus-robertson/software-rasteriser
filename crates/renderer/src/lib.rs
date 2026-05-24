use crate::math::{Matrix4, Vec2, Vec3, Vec4};
pub use crate::mesh::{Mesh, Vertex};
use std::ops::{Add, AddAssign, Mul, SubAssign};

pub mod math;
pub mod mesh;

use font8x8::{BASIC_FONTS, UnicodeFonts};

pub trait Renderer {
    fn clear(&self, target: &mut RenderTarget, colour: [u8; 4]);
    fn draw(&mut self, target: &mut RenderTarget, cmd: DrawCommand);
}

#[derive(Debug, Copy, Clone)]
pub struct Viewport {
    xmin: i32,
    xmax: i32,
    ymin: i32,
    ymax: i32,
}

impl Viewport {
    pub fn new(xmin: i32, xmax: i32, ymin: i32, ymax: i32) -> Self {
        Self {
            xmin,
            xmax,
            ymin,
            ymax,
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        (self.xmax - self.xmin) as u32
    }

    #[inline]
    pub fn height(&self) -> u32 {
        (self.ymax - self.ymin) as u32
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.xmin && x < self.xmax && y >= self.ymin && y < self.ymax
    }

    #[inline(always)]
    pub fn apply(&self, mut v: Vec4) -> Vec4 {
        v.x = self.xmin as f32 + (self.xmax - self.xmin) as f32 * (0.5 + 0.5 * v.x);
        v.y = self.ymin as f32 + (self.ymax - self.ymin) as f32 * (0.5 + 0.5 * v.y);
        v
    }
}

pub struct Framebuffer<'a> {
    pixels: &'a mut [u8],
    width: u32,
    height: u32,
}

impl<'a> Framebuffer<'a> {
    pub fn new(pixels: &'a mut [u8], width: u32, height: u32) -> Self {
        Self {
            pixels,
            width,
            height,
        }
    }
}

pub struct RenderTarget<'a> {
    framebuffer: &'a mut Framebuffer<'a>,
    viewport: Viewport,
}

impl<'a> RenderTarget<'a> {
    pub fn new(framebuffer: &'a mut Framebuffer<'a>, viewport: Viewport) -> Self {
        Self {
            framebuffer,
            viewport,
        }
    }

    #[inline(always)]
    pub fn idx(&self, x: u32, y: u32) -> usize {
        ((y * self.framebuffer.width + x) * 4) as usize
    }

    #[inline(always)]
    fn put(&mut self, x: i32, y: i32, colour: [u8; 4]) {
        if !self.viewport.contains(x, y) {
            return;
        }

        let i = self.idx(x as u32, y as u32);

        self.framebuffer.pixels[i + 0] = colour[0];
        self.framebuffer.pixels[i + 1] = colour[1];
        self.framebuffer.pixels[i + 2] = colour[2];
        self.framebuffer.pixels[i + 3] = colour[3];
    }

    pub fn ndc_to_screen(&self, x: f32, y: f32) -> (i32, i32) {
        let sx = ((x + 1.0) * 0.5 * self.viewport.width() as f32) as i32;
        let sy = ((1.0 - (y + 1.0) * 0.5) * self.viewport.height() as f32) as i32;
        (sx, sy)
    }
}

pub enum DrawCommand<'a> {
    Line {
        from: Vec2,
        to: Vec2,
        colour: [u8; 4],
    },
    Mesh {
        mesh: &'a Mesh,
        camera: &'a Camera,
        colour: [u8; 4],
        transform: Matrix4,
        cull_mode: CullMode,
    },
    Text {
        text: &'a str,
        colour: [u8; 4],
        position: Vec2,
    }
}

#[derive(Debug, Copy, Clone, Default)]
pub enum CullMode {
    None,
    #[default]
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, Copy, Clone)]
pub struct ScreenVertex {
    pub position: Vec2,
    pub depth: f32,
}

pub struct SoftwareRenderer {}

impl SoftwareRenderer {
    pub fn new() -> Self {
        Self {}
    }

    fn draw_line(
        &mut self,
        target: &mut RenderTarget,
        from: (i32, i32),
        to: (i32, i32),
        colour: [u8; 4],
    ) {
        let mut x0 = from.0;
        let mut y0 = from.1;

        let x1 = to.0;
        let y1 = to.1;

        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };

        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };

        let mut err = dx + dy;

        loop {
            target.put(x0, y0, colour);

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = err * 2;

            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    fn draw_text(&mut self, target: &mut RenderTarget, text: &str, position: Vec2, colour: [u8; 4]) {
        for (i, c) in text.chars().enumerate() {
            self.draw_char(target, c, position.x as i32 + (i as i32 * 8), position.y as i32, colour);
        }
    }

    fn draw_char( &mut self, target: &mut RenderTarget, c: char, x: i32, y: i32, colour: [u8; 4]) {
        if let Some(glyph) = BASIC_FONTS.get(c) {
            for (row, bits) in glyph.iter().enumerate() {
                for col in 0..8 {
                    if (bits >> col) & 1 == 1 {
                        target.put(x + col, y + row as i32, colour);
                    }
                }
            }
        }
    }

    fn draw_triangle(
        &mut self,
        target: &mut RenderTarget,
        v0: &mut ClipVertex,
        v1: &mut ClipVertex,
        v2: &mut ClipVertex,
        colour: [u8; 4],
        cull_mode: CullMode,
    ) {
        let mut det012 = Vec4::det2d(v1.position - v0.position, v2.position - v0.position);
        let ccw = det012 < 0.0;

        match cull_mode {
            CullMode::None => {
                if ccw {
                    std::mem::swap(v1, v2);
                    det012 = -det012;
                }
            }
            CullMode::Clockwise => {
                if ccw {
                    std::mem::swap(v1, v2);
                    det012 = -det012;
                }
            }
            CullMode::CounterClockwise => {}
        }

        let xmin = target.viewport.xmin.max(
            v0.position
                .x
                .floor()
                .min(v1.position.x.floor())
                .min(v2.position.x.floor()) as i32,
        );
        let xmax = (target.viewport.xmax - 1).min(
            v0.position
                .x
                .floor()
                .max(v1.position.x.floor())
                .max(v2.position.x.floor()) as i32,
        );
        let ymin = target.viewport.ymin.max(
            v0.position
                .y
                .floor()
                .min(v1.position.y.floor())
                .min(v2.position.y.floor()) as i32,
        );
        let ymax = (target.viewport.ymax - 1).min(
            v0.position
                .y
                .floor()
                .max(v1.position.y.floor())
                .max(v2.position.y.floor()) as i32,
        );

        for y in ymin..=ymax {
            for x in xmin..=xmax {
                let p = Vec4::new(x as f32 + 0.5, y as f32 + 0.5, 0.0, 0.0);

                let det01p = Vec4::det2d(v1.position - v0.position, p - v0.position);
                let det12p = Vec4::det2d(v2.position - v1.position, p - v1.position);
                let det20p = Vec4::det2d(v0.position - v2.position, p - v2.position);

                if det01p >= 0.0 && det12p >= 0.0 && det20p >= 0.0 {
                    target.put(x, y, colour);
                }
            }
        }
    }

    fn draw_mesh(
        &mut self,
        target: &mut RenderTarget,
        camera: &Camera,
        mesh: &Mesh,
        transform: Matrix4,
        colour: [u8; 4],
        cull_mode: CullMode,
    ) {
        let view = camera.to_view_matrix();
        let proj = camera.to_perspective_matrix(1.0);
        let mvp = proj * view * transform;

        let mut buf = VertexBuffer::new();

        for triangle in mesh.indices.chunks_exact(3) {
            buf.clear();

            // clip space verts and apply mvp
            buf.data[0].position = mvp * mesh.positions[triangle[0]].position.as_point();
            buf.data[1].position = mvp * mesh.positions[triangle[1]].position.as_point();
            buf.data[2].position = mvp * mesh.positions[triangle[2]].position.as_point();
            buf.len = 3;

            // we want to clip to ensure all triangles satisfy:
            // -w ≤ x ≤ w
            // -w ≤ y ≤ w
            // -w ≤ z ≤ w
            clip_triangle(&mut buf);

            if buf.len < 3 {
                return;
            }

            for i in 1..(buf.len - 1) {
                let mut v0 = buf.data[0];
                let mut v1 = buf.data[i];
                let mut v2 = buf.data[i + 1];

                // perspective divide and viewport transform
                v0.position = target.viewport.apply(v0.position.perspective_divide());
                v1.position = target.viewport.apply(v1.position.perspective_divide());
                v2.position = target.viewport.apply(v2.position.perspective_divide());

                self.draw_triangle(target, &mut v0, &mut v1, &mut v2, colour, cull_mode)
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ClipVertex {
    pub position: Vec4,
}

impl ClipVertex {
    pub fn new(position: Vec4) -> Self {
        Self { position }
    }
}
struct VertexBuffer {
    data: [ClipVertex; 12],
    len: usize,
}

impl VertexBuffer {
    pub fn new() -> Self {
        Self {
            data: [ClipVertex::new(Vec4::ZERO); 12],
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline(always)]
    pub fn push(&mut self, v: ClipVertex) {
        self.data[self.len] = v;
        self.len += 1;
    }

    #[inline(always)]
    pub fn as_triangles(&self) -> impl Iterator<Item=&[ClipVertex]> {
        self.data[..self.len].chunks(3)
    }
}

/// Checks if a vertex is inside a plane
fn inside(v: &Vec4, plane: usize) -> bool {
    match plane {
        0 => v.x >= -v.w, // left
        1 => v.x <= v.w,  // right
        2 => v.y >= -v.w, // bottom
        3 => v.y <= v.w,  // top
        4 => v.z >= -v.w, // near
        5 => v.z <= v.w,  // far
        _ => unreachable!(),
    }
}

/// Computes the intersection point if an edge crosses a plane
fn intersect(a: Vec4, b: Vec4, plane: usize) -> Vec4 {
    let (da, db) = match plane {
        0 => (a.x + a.w, b.x + b.w),
        1 => (a.w - a.x, b.w - b.x),
        2 => (a.y + a.w, b.y + b.w),
        3 => (a.w - a.y, b.w - b.y),
        4 => (a.z + a.w, b.z + b.w),
        5 => (a.w - a.z, b.w - b.z),
        _ => unreachable!(),
    };

    let t = da / (da - db);

    a + t * (b - a)
}

/// Sutherland-Hodgman clipping algorithm
fn clip_triangle_plane(input: &VertexBuffer, plane: usize, output: &mut VertexBuffer) {
    output.len = 0;

    if input.len < 3 {
        return;
    }

    let verts = &input.data[..input.len];

    for i in 0..input.len {
        let curr = verts[i];
        let prev = verts[(i + input.len - 1) % input.len];

        let curr_in = inside(&curr.position, plane);
        let prev_in = inside(&prev.position, plane);

        // both are inside -> keep current
        if curr_in && prev_in {
            output.push(curr)
        }
        // entering -> add intersection + current
        else if curr_in && !prev_in {
            output.push(ClipVertex {
                position: intersect(prev.position, curr.position, plane),
            });
            output.push(curr);
        }
        // exiting -> add intersection only
        else if !curr_in && prev_in {
            output.push(ClipVertex {
                position: intersect(prev.position, curr.position, plane),
            });
        }
    }
}

fn clip_triangle(buf: &mut VertexBuffer) {
    let mut temp = VertexBuffer::new();

    for plane in 0..6 {
        temp.clear();
        clip_triangle_plane(buf, plane, &mut temp);
        std::mem::swap(buf, &mut temp);
    }
}

impl Renderer for SoftwareRenderer {
    fn clear(&self, target: &mut RenderTarget, colour: [u8; 4]) {
        target
            .framebuffer
            .pixels
            .chunks_exact_mut(4)
            .for_each(|px| px.copy_from_slice(&colour));
    }

    fn draw(&mut self, target: &mut RenderTarget, cmd: DrawCommand) {
        match cmd {
            DrawCommand::Line { from, to, colour } => {
                let (sx0, sy0) = target.ndc_to_screen(from.x, from.y);
                let (sx1, sy1) = target.ndc_to_screen(to.x, to.y);

                Self::draw_line(self, target, (sx0, sy0), (sx1, sy1), colour);
            }
            DrawCommand::Mesh {
                mesh,
                camera,
                colour,
                transform,
                cull_mode,
            } => {
                Self::draw_mesh(self, target, camera, mesh, transform, colour, cull_mode);
            }
            DrawCommand::Text { text, position, colour } => {
                Self::draw_text(self, target, text, position, colour);
            }
        }
    }
}

#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 2.0, 5.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 60.0_f32.to_radians(),
            near: 0.1,
            far: 100.0,
        }
    }
}

impl Camera {
    pub fn forward(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();

        Vec3::new(cp * sy, sp, cp * cy)
    }
    pub fn right(&self) -> Vec3 {
        let forward = self.forward();
        let up = Vec3::new(0.0, 1.0, 0.0);
        up.cross(forward).normalized()
    }

    fn to_view_matrix(&self) -> Matrix4 {
        let forward = self.forward();
        let target = self.position + forward;

        Self::look_at(self.position, target, Vec3::new(0.0, 1.0, 0.0))
    }

    pub fn to_perspective_matrix(&self, aspect: f32) -> Matrix4 {
        Matrix4::perspective(self.near, self.far, self.fov, aspect)
    }
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Matrix4 {
        let f = (target - eye).normalized(); // forward
        let s = f.cross(up).normalized(); // right
        let u = s.cross(f); // corrected up

        Matrix4::new([
            [s.x, s.y, s.z, -s.dot(eye)],
            [u.x, u.y, u.z, -u.dot(eye)],
            [-f.x, -f.y, -f.z, f.dot(eye)],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}
