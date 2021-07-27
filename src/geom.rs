pub mod sphere {
	#[repr(C)]
	#[derive(Debug, Copy, Clone)]
	pub struct Sphere {
		pub pos: [f32; 4],
		pub col: [f32; 4],
		pub r: f32,
		pub reflexivity: f32,
		pub diffuse_factor: f32,
		_pad: f32
	}

	impl Sphere {
		pub fn new(pos: [f32; 3], col: [f32; 4], r: f32, reflexivity: f32, diffuse_factor: f32) -> Self {
			let pos = [pos[0], pos[1], pos[2], 0.0];
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

			let pos = [-(delta_dist) * (self.len as f32 / 2.0) + self.i as f32 * delta_dist, 0.0, 10.0];
			
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

pub mod model {
	use stl_io;
	use std::fs;

	#[repr(C)]
	#[derive(Debug, Clone, Copy)]
	pub struct Model {
		pub pos: [f32; 4],
		pub indices_start: u32, // Index of the first indexed triangle of the model in the global indexed triangles array
		pub indices_end: u32, // End of the indexed triangles
		pub _pad: [u32; 2]
	}

	impl Model {
		pub fn new(name: &str, pos: [f32; 3], vertices: &mut Vec<[f32; 4]>, indices: &mut Vec<[u32; 4]>) -> Self {
			let pos = [pos[0], pos[1], pos[2], 0.0];
			let vertices_offset = vertices.len() as u32;
			let indices_start = indices.len() as u32;

			let mut stl_file_reader = fs::File::open(name)
				.expect(&format!("Failed to open {}", name));
			let stl_file = stl_io::read_stl(&mut stl_file_reader)
				.expect(&format!("Failed to read {}", name));
			

			for vertex in stl_file.vertices {
				vertices.push([
					vertex[0],
					vertex[2], // Swap y and z
					vertex[1],
					0.0
				])
			}

			for indexed_tri in stl_file.faces {
				indices.push([
					indexed_tri.vertices[0] as u32 + vertices_offset,
					indexed_tri.vertices[1] as u32 + vertices_offset,
					indexed_tri.vertices[2] as u32 + vertices_offset,
					0
				])
			}

			let indices_end = indices.len() as u32; // used in a for loop for iterating over the triangles

			Self {
				pos,
				indices_start,
				indices_end,
				_pad: [0; 2]
			}
		}
	}
}