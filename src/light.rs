use nalgebra_glm::{Vec3, Vec4, vec3_to_vec4};

#[derive(Debug, Copy, Clone)]
pub struct PointLight {
	pub pos: [f32; 4],
	pub col: [f32; 3],
	pub intensity: f32
}

impl PointLight {
	pub fn new(pos: Vec3, col: Vec3, intensity: f32) -> Self {
		let pos = Vec4::new(pos.x, pos.y, pos.z, 1.0);
		Self {
			pos: pos.into(),
			col: col.into(),
			intensity
		}
	}
}