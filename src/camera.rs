use std::mem::size_of;

use crate::vulkano::pipeline::shader::SpecializationConstants as SpecConstsTrait;
use crate::vulkano::pipeline::shader::SpecializationMapEntry;

#[derive(Debug, Copy, Clone)]
#[allow(non_snake_case)]
#[repr(C)] // So the in-memory representation of the structure is compatible with the shader
pub struct Camera {
	pub pos: [f32; 4],
	pub orientation: [f32; 4]
}

unsafe impl SpecConstsTrait for Camera {
    fn descriptors() -> &'static [SpecializationMapEntry] {
        static DESCRIPTORS: [SpecializationMapEntry; 2] = [
			SpecializationMapEntry {
				constant_id: 0,
				offset: 0,
				size: 16
			},
			SpecializationMapEntry {
				constant_id: 0,
				offset: 16,
				size: 16
			}
		];

		&DESCRIPTORS
    }
}