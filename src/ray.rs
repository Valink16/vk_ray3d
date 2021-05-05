use std::iter::Iterator;
use crate::winit::dpi::PhysicalSize;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
	origin: [f32; 4],
	dir: [f32; 4],
}
pub struct RayIter {
	size: PhysicalSize<u32>,
	len: u32,
	i: u32,
	depth: f32
}

impl RayIter {
	pub fn new(size: PhysicalSize<u32>, fov: f32) -> Self {
		Self {
			size,
			len: size.width * size.height,
			i: 0,
			depth: ((fov / 2.0).cos() * size.width as f32) / (2.0 * (fov / 2.0)) // Trigo properties
		}
	}
}

impl Iterator for RayIter {
	type Item = Ray;
	fn next(&mut self) -> Option<Self::Item> {
		if self.i == self.len {
			return None;
		}

		let x = (self.i % self.size.width) as f32 - (self.size.width as f32 / 2.0);
		let y = (self.i / self.size.width) as f32 - (self.size.height as f32 / 2.0);

		let norm = (x * x + y * y + self.depth * self.depth).sqrt();
		let dir = [x / norm, y / norm, self.depth / norm, 0.0];

		self.i += 1;

		Some(Ray {
			origin: [0.0, 0.0, 0.0, 1.0],
			dir,
		})
	}
}

impl ExactSizeIterator for RayIter {
	fn len(&self) -> usize {
		(self.len - self.i) as usize
	}
}

/*
/// Utility function to generate the rays
pub fn generate() {
	let rays

	let depth = ((fov / 2.0).cos() * res.x as f32) / (2.0 * (fov / 2.0)); // Trigo properties
	debug!("Setting rays with depth {}", depth);

	for y in ((-(res.y as isize) / 2)..(res.y as isize - (res.y as isize ) / 2)).rev() {
		let yf = y as f32;

		for x in (-(res.x as isize) / 2)..(res.x as isize - (res.x as isize) / 2) {
			let dir_vector = glm::normalize(&Vec3::new(x as f32, yf, depth));
			// trace!("Creating ray with dir {}, {}, {}", dir_vector.x, dir_vector.y, dir_vector.z);
			rays.push(Ray::new(
				Vec3::new(0.0, 0.0, 0.0),
				dir_vector
			));
		}
	}
	debug!("Created {} rays", rays.len());
}

*/
