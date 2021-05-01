use nalgebra_glm::{Vec4, Vec3};
pub struct Sphere {
	pub pos: [f32; 4],
	pub col: [f32; 3],
	pub r: f32
}

impl Sphere {
	pub fn new(pos: [f32; 4], col: [f32; 3], r: f32) -> Self {
		Self {
			pos,
			col,
			r
		}
	}
}

pub struct SphereIter {
	len: u32,
	r: f32,
	i: u32
}

impl SphereIter {
	pub fn new(len: u32, r: f32) -> Self {
		Self {
			len,
			r,
			i: 0
		}
	}
}

impl Iterator for SphereIter {
	type Item = Sphere;
	fn next(&mut self) -> Option<Self::Item> {
		if self.i == self.len {
			return None;
		}

		let delta_dist = self.r * 2.0 + 1.0;

		let pos = [-(delta_dist) * (self.len as f32 / 2.0) + 1.0 + self.i as f32 * delta_dist, 0.0, 10.0, 1.0];
		
		let mut color = Vec3::new(pos[0], pos[1], pos[2]);
		color.normalize_mut();

		println!("{:?}", pos);

		self.i += 1;
		
		Some(Sphere::new(pos, color.into(), self.r))
	}
}

impl ExactSizeIterator for SphereIter {
	fn len(&self) -> usize {
		(self.len - self.i) as usize
	}
}