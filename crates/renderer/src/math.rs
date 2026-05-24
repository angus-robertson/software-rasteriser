use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[inline(always)]
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn det(self, rhs: Vec2) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, rhs: Vec3) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn normalized(self) -> Vec3 {
        self / self.length()
    }

    pub fn cross(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn det2d(self: Vec3, rhs: Vec3) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }

    pub fn as_vector(self: Vec3) -> Vec4 {
        Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: 0.0,
        }
    }

    pub fn as_point(self: Vec3) -> Vec4 {
        Vec4 {
            x: self.x,
            y: self.y,
            z: self.z,
            w: 1.0,
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}


impl<'a, 'b> Sub<&'b Vec3> for &'a Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: &'b Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Vec3 {
        Vec3 {
            x: self * v.x,
            y: self * v.y,
            z: self * v.z,
        }
    }
}

impl Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}
impl Div<f32> for Vec3 {
    type Output = Vec3;

    fn div(self, s: f32) -> Vec3 {
        Vec3 {
            x: self.x / s,
            y: self.y / s,
            z: self.z / s,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Vec4 = Vec4{ x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const ONE: Vec4 = Vec4{ x: 1.0, y: 1.0, z: 1.0, w: 1.0 };

    #[inline(always)]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn dot(self: Vec4, b: Vec4) -> f32 {
        self.x * b.x + self.y * b.y + self.z * b.z + self.w * b.w
    }

    pub fn det2d(a: Vec4, b: Vec4) -> f32 {
        a.x * b.y - a.y * b.x
    }

    pub fn perspective_divide(mut self: Vec4) -> Vec4 {
        let inv = 1.0 / self.w;
        self.w = inv;
        self.x *= inv;
        self.y *= inv;
        self.z *= inv;
        self
    }

    pub fn to_vec3(self: Vec4) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl Sub for Vec4 {
    type Output = Vec4;

    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        Vec4 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
            w: self * rhs.w
        }
    }
}

impl Add for Vec4 {
    type Output = Vec4;

    fn add(self, rhs: Self) -> Self::Output {
        Vec4 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

/// Row-major 4x4 matrix.
#[derive(Debug, Copy, Clone)]
pub struct Matrix4 {
    pub m: [[f32; 4]; 4],
}

impl Matrix4 {
    pub fn new(rows: [[f32; 4]; 4]) -> Matrix4 {
        Matrix4 { m: rows }
    }

    pub const fn identity() -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn scale_vec(s: Vec4) -> Self {
        Self {
            m: [
                [s.x, 0.0, 0.0, 0.0],
                [0.0, s.y, 0.0, 0.0],
                [0.0, 0.0, s.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn scale(s: f32) -> Self {
        Self::scale_vec(Vec4 {
            x: s,
            y: s,
            z: s,
            w: 1.0,
        })
    }

    pub fn translate(v: Vec4) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, v.x],
                [0.0, 1.0, 0.0, v.y],
                [0.0, 0.0, 1.0, v.z],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotate_xy(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();

        Self {
            m: [
                [cos, -sin, 0.0, 0.0],
                [sin, cos, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotate_yz(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();

        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos, -sin, 0.0],
                [0.0, sin, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn rotate_zx(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();

        Self {
            m: [
                [cos, 0.0, sin, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-sin, 0.0, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn perspective(near: f32, far: f32, fov_y: f32, aspect: f32) -> Self {
        let top = near * (fov_y * 0.5).tan();
        let right = top * aspect;

        Self {
            m: [
                [near / right, 0.0, 0.0, 0.0],
                [0.0, near / top, 0.0, 0.0],
                [0.0, 0.0, -(far + near) / (far - near), -2.0 * far * near / (far - near)],
                [0.0, 0.0, -1.0, 0.0],
            ],
        }
    }
}

impl Mul<Vec4> for Matrix4 {
    type Output = Vec4;

    fn mul(self, v: Vec4) -> Vec4 {
        let m = &self.m;

        Vec4 {
            x: m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z + m[0][3] * v.w,
            y: m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z + m[1][3] * v.w,
            z: m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z + m[2][3] * v.w,
            w: m[3][0] * v.x + m[3][1] * v.y + m[3][2] * v.z + m[3][3] * v.w,
        }
    }
}

impl Mul for Matrix4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut m = [[0.0; 4]; 4];

        for row in 0..4 {
            for col in 0..4 {
                m[row][col] =
                    self.m[row][0] * rhs.m[0][col] +
                        self.m[row][1] * rhs.m[1][col] +
                        self.m[row][2] * rhs.m[2][col] +
                        self.m[row][3] * rhs.m[3][col];
            }
        }

        Self { m }
    }
}