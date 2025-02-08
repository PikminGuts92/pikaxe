#[derive(Debug, Default)]
pub struct Sphere {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
}

#[derive(Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug)]
pub struct Color3 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color3 {
    pub fn white() -> Color3 {
        Color3 {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }
}

impl Default for Color3 {
    fn default() -> Color3 {
        Color3::white()
    }
}

#[derive(Debug)]
pub struct Color4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color4 {
    pub fn white() -> Color4 {
        Color4 {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }
}

impl Default for Color4 {
    fn default() -> Color4 {
        Color4::white()
    }
}

#[derive(Debug, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Default for Quat {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
}

#[derive(Debug)]
pub struct Matrix {
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m14: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
    pub m24: f32,
    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
    pub m34: f32,
    pub m41: f32,
    pub m42: f32,
    pub m43: f32,
    pub m44: f32,
}

impl Matrix {
    pub fn identity() -> Matrix {
        Matrix {
            m11: 1.0,
            m12: 0.0,
            m13: 0.0,
            m14: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 0.0,
            m24: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
            m34: 0.0,
            m41: 0.0,
            m42: 0.0,
            m43: 0.0,
            m44: 1.0,
        }
    }
}

impl Default for Matrix {
    fn default() -> Matrix {
        Matrix::identity()
    }
}

#[derive(Debug)]
pub struct Matrix3 {
    pub m11: f32,
    pub m12: f32,
    pub m13: f32,
    pub m21: f32,
    pub m22: f32,
    pub m23: f32,
    pub m31: f32,
    pub m32: f32,
    pub m33: f32,
}

impl Matrix3 {
    pub fn identity() -> Matrix3 {
        Matrix3 {
            m11: 1.0,
            m12: 0.0,
            m13: 0.0,
            m21: 0.0,
            m22: 1.0,
            m23: 0.0,
            m31: 0.0,
            m32: 0.0,
            m33: 1.0,
        }
    }
}

impl Default for Matrix3 {
    fn default() -> Matrix3 {
        Matrix3::identity()
    }
}