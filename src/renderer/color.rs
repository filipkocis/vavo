use std::ops::{Add, Div, Mul, Sub};

#[repr(C)]
#[derive(
    Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable, crate::macros::Reflect,
)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    #[inline]
    pub const fn from_rgb_slice(slice: &[f32; 3]) -> Self {
        Self::rgb(slice[0], slice[1], slice[2])
    }

    #[inline]
    pub const fn from_rgba_slice(slice: &[f32; 4]) -> Self {
        Self::new(slice[0], slice[1], slice[2], slice[3])
    }

    pub fn as_rgba_slice(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn as_rgba_slice_u8(&self) -> [u8; 4] {
        [
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        ]
    }

    pub fn srgb_value_to_linear(value: f32) -> f32 {
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    pub fn to_linear_rgb(&self) -> Self {
        Self {
            r: Self::srgb_value_to_linear(self.r),
            g: Self::srgb_value_to_linear(self.g),
            b: Self::srgb_value_to_linear(self.b),
            a: self.a,
        }
    }
}

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        Self {
            r: value.r.into(),
            g: value.g.into(),
            b: value.b.into(),
            a: value.a.into(),
        }
    }
}

impl From<Color> for glyphon::Color {
    fn from(value: Color) -> Self {
        let color = value.as_rgba_slice_u8();
        Self::rgba(color[0], color[1], color[2], color[3])
    }
}

impl From<glyphon::Color> for Color {
    fn from(value: glyphon::Color) -> Self {
        Self::new(
            value.r() as f32 / 255.0,
            value.g() as f32 / 255.0,
            value.b() as f32 / 255.0,
            value.a() as f32 / 255.0,
        )
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}

impl Add for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

impl Sub for Color {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
            a: self.a - rhs.a,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl Div<f32> for Color {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
            a: self.a / rhs,
        }
    }
}
