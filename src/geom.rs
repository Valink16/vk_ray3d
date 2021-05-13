pub mod sphere {
	#[derive(Debug, Copy, Clone)]
	#[repr(C)]
	pub struct Sphere {
		pub pos: [f32; 4],
		pub col: [f32; 4],
		pub r: f32,
		pub reflexivity: f32,
		pub diffuse_factor: f32,
		_pad: f32
	}

	impl Sphere {
		pub fn new(pos: [f32; 4], col: [f32; 4], r: f32, reflexivity: f32, diffuse_factor: f32) -> Self {
			Self {
				pos,
				col,
				r,
				reflexivity,
				diffuse_factor,
				_pad: 0.0
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

			let delta_dist = self.r * 2.5;

			let pos = [-(delta_dist) * (self.len as f32 / 2.0) + self.i as f32 * delta_dist, 0.0, 10.0, 1.0];
			
			// let mut color = Vec4::new(pos[0].abs(), 0.0, pos[2] / 20.0, 1.0);
			// color.normalize_mut();
			let color = [1.0, 1.0, 1.0, 1.0];

			println!("{:?}", pos);

			self.i += 1;
			
			Some(Sphere::new(pos, color.into(), self.r, 0.5, 0.5))
		}
	}

	impl ExactSizeIterator for SphereIter {
		fn len(&self) -> usize {
			(self.len - self.i) as usize
		}
	}
}
