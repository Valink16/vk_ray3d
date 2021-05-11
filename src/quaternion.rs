use std::ops::Mul;

use nalgebra_glm as glm;
use glm::{Vec3, dot, cross};

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub v: glm::Vec3,
    pub s: f32
}

impl Quaternion {
    pub fn new(i: f32, j: f32, k: f32, s: f32) -> Self {
        Self {
            v: Vec3::new(i, j, k),
            s
        }
    }

    pub fn from_axis(axis: Vec3, a: f32) -> Self { // Expects a normalized axis vector
        let half_a = a / 2.0;
        Self {
            v: axis * half_a.sin(),
            s: half_a.cos()
        }
    }

    pub fn transform_point(&self, p: Vec3) -> Vec3 {
        let inv = Quaternion { v: -self.v, s: self.s };
        (self * &Quaternion { v: p, s: 0.0 } * inv).v
    }
}

impl Mul for &Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Self) -> Self::Output {
        Quaternion {
            v: self.s * rhs.v + rhs.s * self.v + cross(&self.v, &rhs.v),
            s: self.s * rhs.s - dot(&self.v, &rhs.v)
        }
    }
}

impl Mul for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Self) -> Self::Output {
        Quaternion {
            v: self.s * rhs.v + rhs.s * self.v + cross(&self.v, &rhs.v),
            s: self.s * rhs.s - dot(&self.v, &rhs.v)
        }
    }
}

impl Into<[f32; 4]> for Quaternion {
	fn into(self) -> [f32; 4] {
		[
			self.v.x,
			self.v.y,
			self.v.z,
			self.s
		]
	}
}

impl From<[f32; 4]> for Quaternion {
    fn from(data: [f32; 4]) -> Self {
        Self {
            v: Vec3::new(data[0], data[1], data[2]),
            s: data[3]
        }
    }
}