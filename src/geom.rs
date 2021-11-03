pub mod sphere {
	#[repr(C)]
	#[derive(Debug, Copy, Clone)]
	pub struct Sphere {
		pub pos: [f32; 4],
		pub col: [f32; 4],
		pub r: f32,
		pub reflexivity: f32,
		pub diffuse_factor: f32,
		pub texture_index: i32,
	}

	impl Sphere {
		pub fn new(pos: [f32; 3], col: [f32; 4], r: f32, reflexivity: f32, diffuse_factor: f32, texture_index: i32) -> Self {
			let pos = [pos[0], pos[1], pos[2], 0.0];
			Self {
				pos,
				col,
				r,
				reflexivity,
				diffuse_factor,
				texture_index
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
			
			Some(Sphere::new(pos, color.into(), self.r, 0.5, 0.5, 0))
		}
	}

	impl ExactSizeIterator for SphereIter {
		fn len(&self) -> usize {
			(self.len - self.i) as usize
		}
	}
}

pub mod model {
	use compute_vk::vulkano::pipeline::vertex;
use stl_io;
	use tobj;
	use std::fs;
	use std::path::Path;
	use std::fmt::Debug;

	#[repr(C)]
	#[derive(Debug, Clone, Copy)]
	pub struct Model {
		pub pos: [f32; 4],
		pub col: [f32; 4],
		pub reflexivity: f32,
		pub diffuse_factor: f32,
		pub indices_start: u32, // Index of the first indexed triangle of the model in the global indexed triangles array
		pub indices_end: u32, // End of the indexed triangles
		pub vertex_start: u32, // Index of the first vertex
		pub vertex_end: u32, // Last vertex
		pub texture_index: i32,
		_pad: u32
	}

	impl Model {
		pub fn from_obj<P: AsRef<Path> + Debug>(name: P, pos: [f32; 3], col: [f32; 4], reflexivity: f32, diffuse_factor: f32, texture_index: i32, vertices: &mut Vec<[f32; 4]>, uvs: &mut Vec<[f32; 2]>,  indices: &mut Vec<[u32; 4]>, normals: &mut Vec<[f32; 4]>) -> Self {
			let pos = [pos[0], pos[1], pos[2], 0.0];
			let vertices_offset = vertices.len() as u32;
			let indices_start = indices.len() as u32;

			let (models, _mats)  = tobj::load_obj(&name,
				&tobj::LoadOptions {
					single_index: true,
					// triangulate: true,
					// reorder_data: true,
					.. Default::default()
				}
			).expect(&format!("Failed to load {:?}", name));

			for model in models {
				let mesh = model.mesh;

				assert_eq!(mesh.positions.len() % 3, 0);
				assert_eq!(mesh.texcoords.len() % 2, 0);
				assert_eq!(mesh.normals.len() % 3, 0);

				for pos_i in (0..mesh.positions.len()).step_by(3) {
					vertices.push([
						mesh.positions[pos_i],
						mesh.positions[pos_i + 1],
						mesh.positions[pos_i + 2],
						0.0
					])
				}

				for uv_i in (0..mesh.texcoords.len()).step_by(2) { // Step by 2 for UVs
					uvs.push([
						mesh.texcoords[uv_i],
						mesh.texcoords[uv_i + 1]
					])
				}

				for indice_i in (0..mesh.indices.len()).step_by(3) {
					indices.push([
						mesh.indices[indice_i] + vertices_offset,
						mesh.indices[indice_i + 1] + vertices_offset,
						mesh.indices[indice_i + 2] + vertices_offset,
						0
					])
				}

				for normal_i in (0..mesh.normals.len()).step_by(3) {
					normals.push([
						mesh.normals[normal_i],
						mesh.normals[normal_i + 1],
						mesh.normals[normal_i + 2],
						0.0
					])
				}
			}			

			let indices_end = indices.len() as u32; // used in a for loop for iterating over the triangles

			Self {
				pos,
				col,
				reflexivity,
				diffuse_factor,
				indices_start,
				indices_end,
				vertex_start: vertices_offset,
				vertex_end: vertices.len() as u32,
				texture_index,
				_pad: 0
			}
		}

		pub fn from_stl(name: &str, pos: [f32; 3], col: [f32; 4], reflexivity: f32, diffuse_factor: f32, texture_index: i32, vertices: &mut Vec<[f32; 4]>, uvs: &mut Vec<[f32; 2]>, indices: &mut Vec<[u32; 4]>, normals: &mut Vec<[f32; 4]>) -> Self {
			let pos = [pos[0], pos[1], pos[2], 0.0];
			let vertices_offset = vertices.len() as u32;
			let indices_start = indices.len() as u32;
	
			let mut stl_file_reader = fs::File::open(name)
				.expect(&format!("Failed to open {}", name));
			let stl_file = stl_io::read_stl(&mut stl_file_reader)
				.expect(&format!("Failed to read {}", name));
			
			normals.reserve(stl_file.vertices.len());

			for vertex in stl_file.vertices {
				vertices.push([
					vertex[0],
					vertex[1],
					vertex[2],
					0.0
				]);

				uvs.push([0.0, 0.0]);
			}
	
			for indexed_tri in stl_file.faces {
				let new_tri = [
					indexed_tri.vertices[0] as u32 + vertices_offset,
					indexed_tri.vertices[1] as u32 + vertices_offset,
					indexed_tri.vertices[2] as u32 + vertices_offset,
					0
				];

				// This assures compatibility with per-vertex-normal formats(Wavefront)
				// Essentially copy the normal of the indexed_triangle for each of it's vertices
				for i in new_tri.iter() {
					let v = vertices[*i as usize];
					let vi = vertices.iter().position(|&_v| _v == v).unwrap();

					if vi >= normals.len() {
						for j in 0..=(vi - normals.len()) { // grow the normals vector
							normals.push([0.0; 4]);
						}
					}

					normals[vi] = [
						indexed_tri.normal[0],
						indexed_tri.normal[1],
						indexed_tri.normal[2],
						0.0
					];
				}
					
				indices.push(new_tri);
			}
	
			let indices_end = indices.len() as u32; // used in a for loop for iterating over the triangles
	
			Self {
				pos,
				col,
				reflexivity,
				diffuse_factor,
				indices_start,
				indices_end,
				vertex_start: vertices_offset,
				vertex_end: vertices.len() as u32,
				texture_index,
				_pad: 0
			}
		}
	}
}