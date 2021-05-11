/// Copy pasted from the dump from the vulkano_shader::shader macro and modified to be able to load and compile glsl code

use std::usize;
use vulkano::descriptor::descriptor::DescriptorDesc; 
use vulkano::descriptor::descriptor::DescriptorDescTy;
use vulkano::descriptor::descriptor::DescriptorBufferDesc;
use vulkano::descriptor::descriptor::DescriptorImageDesc;
use vulkano::descriptor::descriptor::DescriptorImageDescDimensions; 
use vulkano::descriptor::descriptor::DescriptorImageDescArray;
use vulkano::descriptor::descriptor::ShaderStages;
use vulkano::descriptor::pipeline_layout::PipelineLayoutDesc;
use vulkano::descriptor::pipeline_layout::PipelineLayoutDescPcRange;

use shaderc;
use relative_path::{RelativePath, RelativePathBuf};

use std::fs::read;

pub struct Shader {
    shader : std::sync::Arc<vulkano::pipeline::shader::ShaderModule>,
    layout: MainLayout
} 

impl Shader {
    #[doc = r" Loads the shader in Vulkan as a `ShaderModule`."]
    #[inline]
    #[allow(unsafe_code)] pub fn
    load(device: std::sync::Arc<vulkano::device::Device>, filename: &str, layout: MainLayout) -> Result<Shader, String> {
        if !device.enabled_features().shader_storage_image_extended_formats {
            panic !
            ("Device feature {:?} required",
             "shader_storage_image_extended_formats");
        }
		
        let src = match read(filename) {
            Err(e) => return Err(String::from(format!("Failed to load {}: {:?}", filename, e))),
            Ok(data) => {
                match String::from_utf8(data) {
                    Ok(s) => s,
                    Err(e) => return Err(String::from(format!("Failed to parse source from {}: {:?}", filename, e)))
                }
            }
        };

        let mut c = shaderc::Compiler::new().unwrap();

        let mut compile_options = shaderc::CompileOptions::new().unwrap();
        compile_options.set_include_callback(|requested_name, include_type, requesting_name, _depth| {

            let requesting_path = RelativePath::new(requesting_name);

            let filename = match include_type {
                shaderc::IncludeType::Relative => {
                    match requesting_path.parent() {
                        Some(parent_dir) => {
                            let relative_to_parent = parent_dir.join_normalized(requested_name);
                            relative_to_parent
                        },
                        None => return Err(String::from(format!("No parent for {}", requesting_path)))
                    }
                },
                shaderc::IncludeType::Standard => {
                    RelativePathBuf::from_path(requested_name).unwrap()
                },
            };

            let content = match read(filename.as_str()) {
                Err(e) => return Err(String::from(format!("Cannot include {} to {} using path {}: {:?}", requested_name, requesting_name, filename, e))),
                Ok(data) => {
                    match String::from_utf8(data) {
                        Ok(s) => s,
                        Err(e) => return Err(String::from(format!("Cannot parse include from {} to {} using path {}: {:?}", requested_name, requesting_name, filename, e)))
                    }
                }
            };
            
            Ok(
                shaderc::ResolvedInclude {
                    resolved_name: String::from(requested_name),
                    content
                }
            )
        });

        let artifact = match c.compile_into_spirv(
            src.as_str(),
            shaderc::ShaderKind::Compute,
            filename,
            "main",
            Some(&compile_options)
        ) {
            Ok(a) => a,
            Err(e) => return Err(String::from(format!("Failed to compile {}: {:?}", filename, e)))
        };
		 
		unsafe {
            Ok(Shader {
				shader: vulkano::pipeline::shader::ShaderModule::new(device, artifact.as_binary_u8()).unwrap(),
                layout
            })
        }

    } 
	
	#[doc = r" Returns the module that was created."]
	#[allow(dead_code)]
    #[inline]
	pub fn module(& self) -> &std::sync::Arc <vulkano::pipeline::shader::ShaderModule> { 
		&self.shader
	}

    #[doc = r" Returns a logical struct describing the entry point named `{ep_name}`."]
    #[inline]
	#[allow(unsafe_code)]
	pub fn main_entry_point(&self) -> vulkano::pipeline::shader::ComputeEntryPoint<(), MainLayout> {
        unsafe {
            #[allow(dead_code)]
			static NAME : [u8 ; 5usize] = [109u8, 97u8, 105u8, 110u8, 0]; // Entry point function must be "main"
			self.shader.compute_entry_point(
				std::ffi::CStr::from_ptr(NAME.as_ptr() as * const _),
				self.layout.clone()
			)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MainInput;

#[allow(unsafe_code)] 
unsafe impl::vulkano::pipeline::shader::ShaderInterfaceDef for MainInput {
    type Iter = MainInputIter;
	fn elements(&self) -> MainInputIter { MainInputIter { num : 0 } }
}

#[derive(Debug, Copy, Clone)]
pub struct MainInputIter { num : u16 }

impl Iterator for MainInputIter {
    type Item = vulkano::pipeline::shader::ShaderInterfaceDefEntry;
    #[inline]
	fn next(&mut self) -> Option<Self::Item> { None }
    #[inline]
	fn size_hint(&self) -> (usize, Option<usize>) { 
		let len = 0usize - self.num as usize;
		(len, Some(len))
	}
}

impl ExactSizeIterator for MainInputIter { }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MainOutput;

#[allow(unsafe_code)]
unsafe impl vulkano::pipeline::shader::ShaderInterfaceDef for MainOutput {
    type Iter = MainOutputIter;
	fn elements(&self) -> MainOutputIter { MainOutputIter { num: 0 } }
}

#[derive(Debug, Copy, Clone)]
pub struct MainOutputIter { num : u16 }

impl Iterator for MainOutputIter {
    type Item = vulkano::pipeline::shader::ShaderInterfaceDefEntry;
    #[inline] 
	fn next(& mut self) -> Option<Self::Item> { None }
    #[inline]
	fn size_hint(& self) -> (usize, Option<usize>) {
		let len = 0usize - self.num as usize;
		(len, Some(len)) 
	}
} 

impl ExactSizeIterator for MainOutputIter { } 

#[derive(Debug, Clone)]
pub struct MainLayout {
    pub stages: ShaderStages,
    pub sets: Vec<Vec<DescriptorDesc>>,
    pub push_constants_ranges: Vec<PipelineLayoutDescPcRange>
}

impl MainLayout {
    pub fn new() -> Self {
        Self {
            stages: ShaderStages {
                compute: true,
                .. ShaderStages::none()
            },
            sets: Vec::<Vec<DescriptorDesc>>::new(),
            push_constants_ranges: Vec::<PipelineLayoutDescPcRange>::new()
        }
    }

    /// Add a descriptor to the given set, binding is implicitly defined through the add order
    pub fn add_image(&mut self, set: usize) {
        let desc = DescriptorDesc {
            ty: DescriptorDescTy::Image(
                DescriptorImageDesc {
                    sampled : false, 
                    dimensions: DescriptorImageDescDimensions::TwoDimensional,
                    format: None,
                    multisampled: false,
                    array_layers: DescriptorImageDescArray::Arrayed { max_layers: Some(1) },
                }
            ),
            array_count: 1u32,
            stages: self.stages.clone(),
            readonly: false,
        };

        self.add_desc(set, desc);
    }

    pub fn add_buffer(&mut self, set: usize, readonly: bool) {
        let desc = DescriptorDesc {
            ty: DescriptorDescTy::Buffer(DescriptorBufferDesc {dynamic: None, storage: true}),
            array_count: 1,
            stages: ShaderStages::compute(),
            readonly
        };

        self.add_desc(set, desc);
    }

    pub fn add_desc(&mut self, set: usize, desc: DescriptorDesc) {
        let set = set as isize;
        if !(set < self.sets.len() as isize) {
            for _ in 0..(set - (self.sets.len() as isize - 1)) {
                self.sets.push(Vec::<DescriptorDesc>::new())
            }
        }
        self.sets[set as usize].push(desc)
    }

    pub fn add_push_constant_range(&mut self, offset: usize, size: usize) {
        self.push_constants_ranges.push(PipelineLayoutDescPcRange {
            offset,
            size,
            stages: ShaderStages::compute()
        });
    }
}

#[allow(unsafe_code)]
unsafe impl PipelineLayoutDesc for MainLayout {
    fn num_sets(&self) -> usize { self.sets.len() }

	fn num_bindings_in_set(&self, set: usize) -> Option<usize> { 
        match self.sets.get(set) {
            Some(s) => Some(s.len()),
            None => None
        }
	} 
	fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
        match self.sets.get(set) {
            None => None,
            Some(s) => match s.get(binding) {
                Some(desc) => Some(desc.clone()),
                None => None
            }
        }
    }
	
	fn num_push_constants_ranges(&self) -> usize { self.push_constants_ranges.len() }

	fn push_constants_range(&self, num: usize) -> Option<PipelineLayoutDescPcRange> {
        match self.push_constants_ranges.get(num) {
            None => None,
            Some(r) => Some(*r)
        }
    }
}