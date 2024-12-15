use std::ops::{Add, Div, Mul, Sub};

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    pub const fn from_rgb_slice(slice: &[f32; 3]) -> Self {
        Self::rgb(slice[0], slice[1], slice[2])
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

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r.into(),
            g: self.g.into(),
            b: self.b.into(),
            a: self.a.into(),
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(1.0, 1.0, 1.0, 1.0)
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
