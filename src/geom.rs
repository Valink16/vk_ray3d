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


pub mod poly {
	use std::fs;
	use std::path;
	use stl_io;

	const FACE_COUNT: usize = 100;
	#[derive(Debug)]
	#[repr(C)]
	pub struct Polygon {
		vertices: [[f32; 4]; FACE_COUNT * 3], // *3 So we have 3 vertices for each triangle
		indices: [[u32; 4]; FACE_COUNT], // Array of ivec3 containing indexes of each triangle vertices
		indices_size: u32, // Tells at which index to stop on the indices array
		_pad: [u32; 3]
	}

	impl Polygon {
		pub fn from_file(filename: &str) -> Result<Self, String> {
			let mut f = match fs::File::open(filename) {
				Ok(f) => f,
				Err(e) => return Err(String::from(format!("Failed to open STL file, {:?}", e)))
			};

			let mesh = match stl_io::read_stl(&mut f) {
				Ok(mesh) => mesh,
				Err(e) => return Err(String::from(format!("Failed to read STL file, {:?}", e)))
			};

			let mut vertices: [[f32; 4]; FACE_COUNT * 3] = [[0.0; 4]; FACE_COUNT * 3];
			let mut indices: [[u32; 4]; FACE_COUNT] = [[0; 4]; FACE_COUNT];

			for (i, t) in mesh.faces.iter().enumerate() {
				indices[i] = [
					t.vertices[0] as u32,
					t.vertices[1] as u32,
					t.vertices[2] as u32,
					0
				]
			}

			for (i, v) in mesh.vertices.iter().enumerate() {
				vertices[i] = [
					v[0],
					v[1],
					v[2],
					0.0
				];
			}

			Ok(Self {
				vertices,
				indices,
				indices_size: mesh.faces.len() as u32,
				_pad: [0; 3]
			})
		}
	}
}