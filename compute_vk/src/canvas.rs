use crate::{util, loader};
use vulkano::{buffer::{BufferUsage, CpuAccessibleBuffer}, command_buffer::CommandBuffer, device::{Device, DeviceOwned, Queue}, image::{ImageCreateFlags, ImageDimensions}};
use vulkano::instance::{Instance, ApplicationInfo};
use vulkano::image::{StorageImage, ImageUsage, ImageAccess};
use vulkano::format::{Format, ClearValue};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::sampler::Filter;
use vulkano::sync::GpuFuture;
use vulkano::pipeline::{ComputePipeline, shader::SpecializationConstants};
use vulkano::descriptor::{descriptor_set::UnsafeDescriptorSetLayout, PipelineLayoutAbstract, DescriptorSet, descriptor_set::DescriptorSetDesc};

use vulkano::swapchain;
use swapchain::{Swapchain, SwapchainCreationError, SurfaceTransform, PresentMode, FullscreenExclusive};

use vulkano::sync;

use image;
use chrono;
use vulkano_win::VkSurfaceBuild;
use winit::window::WindowBuilder;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent, DeviceEvent, ElementState};
use winit::dpi::PhysicalSize;

use std::{future::Ready, hash::Hash};
use std::sync::Arc;
use std::time::Instant;
use std::fmt::Debug;
use std::fs;
use std::path;
use std::env::{current_dir, set_current_dir};

// This class will manage the window and present the output of the compute shaders
pub struct Canvas<Ds, Update, Resize, Pc: 'static, DsBuilder> where
	Ds: DescriptorSet + DescriptorSetDesc + DeviceOwned + Eq + Hash + PartialEq + Send + Sync,
	Update: FnMut(Option<&winit::event::Event<()>>, f64) -> (Pc, bool),
	Resize: FnMut(PhysicalSize<u32>, Arc<Device>, Arc<Queue>, Arc<UnsafeDescriptorSetLayout>) -> (Arc<Ds>, Arc<StorageImage<Format>>, [u32; 3], Update) + Clone,
	Pc: SpecializationConstants + Copy + Debug,
	DsBuilder: FnOnce(PhysicalSize<u32>, Arc<Device>, Arc<Queue>, Arc<UnsafeDescriptorSetLayout>) -> (Arc<Ds>, Arc<StorageImage<Format>>, [u32; 3], Update, Resize) + Clone { // DsBuilder is a closure which builds the DescriptorSet
	// TODO: 
	// - implement the class, must be able to create the window, manage events, draw output
	// - take potential multiple compute shaders with their corresponding descriptor sets + input data + output texture
	ds_builder: DsBuilder, // A closure used by `Canvas` to build the input for the compute shader
	
	// Vulkan specific attributes
	instance: Arc<Instance>,
	pub device: Arc<Device>,
	pub queue: Arc<Queue>,
	pub window_size: PhysicalSize<u32>,
	pub shader: Option<loader::Shader>,
}

impl<Ds: 'static, Update: 'static, Resize: 'static, Pc, DsBuilder: 'static> Canvas<Ds, Update, Resize, Pc, DsBuilder> where 
	Ds: DescriptorSet + DescriptorSetDesc + DeviceOwned + Eq + Hash + PartialEq + Send + Sync,
	Update: FnMut(Option<&winit::event::Event<()>>, f64) -> (Pc, bool),
	Resize: FnMut(PhysicalSize<u32>, Arc<Device>, Arc<Queue>, Arc<UnsafeDescriptorSetLayout>) -> (Arc<Ds>, Arc<StorageImage<Format>>, [u32; 3], Update) + Clone,
	Pc: SpecializationConstants + Copy + Debug,
	DsBuilder: FnOnce(PhysicalSize<u32>, Arc<Device>, Arc<Queue>, Arc<UnsafeDescriptorSetLayout>) -> (Arc<Ds>, Arc<StorageImage<Format>>, [u32; 3], Update, Resize) + Clone {
	
	/// #### ds_builder closure arguments
	/// - `PhysicalSize` representing the size of the swapchain and returning a tuple containing the descriptor set for the compute shader and the output image
	/// - `Arc<Device>`
	/// - `Arc<Queue>`
	/// - `Arc<UnsafeDescriptorSetLayout>` should be used to build your descriptor sets
	/// #### ds_builder closure return
	/// - `Arc<Ds>` The descriptor set
	/// - `Arc<StorageImage<Format>>` A storage image which will be shown on screen
	/// - `[u32; 3]` Size of the dispatch
	/// - `Update` Closure
	/// 	- arguments
	///			- `Option<winit::event::WindowEvent` The window event of the current frame
	///		- return
	///			- 'bool' rebuild, used to communicate whether ds_builder should be recalled or not
	/// - `Rebuild` Closure - Used when window is resized to only update certain elements
	/// #### Update closure arguments
	/// - `Option<winit::event::Event>` Event passed to the closure so the user can update accordingly, needs to be passed back to avoid a weird winit bug (can't clone `Event`)
	/// - `f64` "time" used to scale animations
	/// #### Update closure return
	/// - `Option<winit::event::Event<()>>` The same event as the one passed in
	/// - `Pc` push_constant implementing the `SpecializationConstants` trait
	/// - `bool` Indicates whether the ds_builder needs to be recalled or not
	pub fn new(window_size: PhysicalSize<u32>, ds_builder: DsBuilder, app_info: &ApplicationInfo) -> Self {
		let (instance, device, queue) = util::init_vulkano(app_info);

		Self {
			ds_builder,
			instance,
			device,
			queue,
			window_size,
			shader: None,
		}
	}

	pub fn set_shader(&mut self, cs: loader::Shader) {
		self.shader = Some(cs);
	}

	/// #### Arguments
	/// - `animation_fps` target fps for animation, mainly defines how `t` passed to `update` closure is incremented
	pub fn run(self, animation_fps: f64) -> Result<(), String> { // Runs the event_loop
		let shader = match self.shader {
			Some(s) => s,
			None => return Err(String::from("The shader was not set"))
		};

		// Taking ownership of needed attributes
		let device = self.device;
		let queue = self.queue;
		let ds_builder = self.ds_builder;

		let event_loop = EventLoop::<()>::new();

		let surface = WindowBuilder::new()
			.with_inner_size(self.window_size)
			.build_vk_surface(&event_loop, self.instance.clone())
			.expect("Failed to create window surface");

		// Creating the swapchain
		let (mut swapchain, mut images) = {
			let caps = surface.capabilities(device.physical_device()).unwrap();
			let alpha_behavior = caps.supported_composite_alpha.iter().next().unwrap();
			let (format, color_space)  = caps.supported_formats[0];

			Swapchain::new(
				device.clone(),
				surface.clone(),
				caps.min_image_count,
				format,
				surface.window().inner_size().into(),
				1,
				ImageUsage {
					transfer_destination: true,
					color_attachment: true,
					.. ImageUsage::none()
				},
				&queue,
				SurfaceTransform::Identity,
				alpha_behavior,
				PresentMode::Immediate,
				FullscreenExclusive::Default,
				false,
				color_space
			).unwrap()
		};

		println!("Created swapchain with {} images using format {:?}", images.len(), swapchain.format());

		// setting up the compute pipeline
		let compute_pipeline = Arc::new(ComputePipeline::new(
			device.clone(),
			&shader.main_entry_point(),
			&(),
			None
		).unwrap());

		let layout = compute_pipeline.layout().descriptor_set_layout(0).unwrap().to_owned();

		let inner_size = surface.window().inner_size();
		let (mut descriptor_set, mut output_img, mut dispatch, mut update, resize) = (ds_builder.clone())(inner_size, device.clone(), queue.clone(), layout.clone());
		
		let mut image_save_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, // Buffer which will be used to save rendered images
		(0 .. inner_size.width * inner_size.height * 4).map(|_| 255u8))
			.expect("failed to create image_save_buffer");

		let mut dest_image = images[0].clone(); // This arc will reference the image being rendered to every time
		let mut save_image = StorageImage::with_usage( // Image which is potentially used to save on the disk
			device.clone(), ImageDimensions::Dim2d { width: inner_size.width, height: inner_size.height, array_layers: 1 }, Format::R8G8B8A8Unorm,
			ImageUsage {
				transfer_source: true,
				transfer_destination: true,
				.. ImageUsage::none()
			}, 
			ImageCreateFlags::none(),
			vec![queue.family()]
		).unwrap();

		let (mut dest_dim, mut output_dim, scale) = {
			let _dest_dim = images[0].dimensions().width_height();
			let _output_dim = output_img.dimensions().width_height();
			let (w, h) = (_dest_dim[0] as f32, _dest_dim[1] as f32); // _dest_dim is the inner size of the window
			
			(
				[_dest_dim[0] as i32, _dest_dim[1] as i32, 1],
				[_output_dim[0] as i32, _output_dim[1] as i32, 1],
				(w / _output_dim[0] as f32, h / _output_dim[1] as f32)
			)
		};

		// State control variables
		let mut resized = false;
		let mut minimized = false;
		let mut previous_frame_end = Some(sync::now(device.clone()).boxed());
		let mut start = Instant::now();
		let mut frames = 0;
		let dt_log_rate = 300;
		let mut t = 0.0; // Global time for animation, passed to update closure
		let (mut push_constants, mut need_update) = update(None, t);
		let mut recording = false;
		let mut current_frame = 0;
		let root_folder = current_dir().unwrap();
		let mut recording_path = root_folder.clone();

		event_loop.run(move |ev: Event<()>, _, control_flow| {
			let update_res = update(Some(&ev), t);
			push_constants = update_res.0;
			need_update = update_res.1;

			if need_update {
				resized = true;
			}

			match ev {
				Event::WindowEvent { event, .. } => {
					match event {
						WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
						WindowEvent::Resized(s) => {
							resized = true;
							minimized = s.width == 0 || s.height == 0;
						},
						_ => {
							()
						}
					}
				}

				Event::DeviceEvent { event, .. } => {
					match event {
						DeviceEvent::Key(kb_input) => {
							if kb_input.state == ElementState::Released {
								match kb_input.scancode {
									1 => *control_flow = ControlFlow::Exit, // Escape
									60 => { // F2, start or stop recording
										if !recording {
											recording = true;
											let mut timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false);
											timestamp = timestamp.replace(":", "_");
											println!("Using folder name: {}", timestamp);
											recording_path = path::PathBuf::from(format!("Captures/{}", timestamp));
											fs::create_dir_all(&recording_path).unwrap();
											set_current_dir(&recording_path).unwrap();
										} else {
											recording = false;
											set_current_dir(&root_folder).unwrap();
										}
									}
									_ => ()
								}
							}
						},
						_ => ()
					}
				}
				
				Event::RedrawEventsCleared => {
					frames += 1;
					t += 1.0 / animation_fps;

					if frames % dt_log_rate == 0 {
						frames = 0;
						let elapsed = start.elapsed();
						start = Instant::now();
						println!("Frame time: {}ms, FPS: {}", elapsed.as_secs_f32() * 1000.0 / dt_log_rate as f32, dt_log_rate as f32 / elapsed.as_secs_f32());
					}

					if minimized { return; } // Don't try anything if the window is minimized, this prevents the errors creating images with 0 sizes

					
					previous_frame_end.as_mut().unwrap().cleanup_finished();
					if recording {
						let mut builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
						builder
							.copy_image_to_buffer(save_image.clone(), image_save_buffer.clone()).unwrap();
				
						let cb = builder.build().unwrap();
						let finished = cb.execute(queue.clone()).unwrap();
				
						finished
							.then_signal_fence_and_flush().unwrap()
							.wait(None).unwrap();
				
						let inner_size = save_image.dimensions();
						let data = image_save_buffer.read().unwrap();
						let to_save = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(inner_size.width(), inner_size.height(), &data[..]).unwrap();
						to_save.save(format!("{}.png", current_frame)).unwrap();
						current_frame += 1;
			
						if current_frame % (animation_fps as isize / 10) == 0 {
							println!("Generated {} frames", current_frame);
						}
					}
					
					if resized { // Rebuild the output_img and the swapchain
						resized = false;
						let dimensions: [u32; 2] = surface.window().inner_size().into();
						let (new_swapchain, new_images) =
							match swapchain.recreate_with_dimensions(dimensions) {
								Ok(r) => r,
								// This error tends to happen when the user is manually resizing the window.
								// Simply restarting the loop is the easiest way to fix this issue.
								Err(SwapchainCreationError::UnsupportedDimensions) => return,
								Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
							};

						swapchain = new_swapchain;
						images = new_images;

						let _dest_dim = images[0].dimensions();
						let _output_dim = [(_dest_dim.width() as f32 / scale.0).floor() as u32, (_dest_dim.height() as f32 / scale.1).floor() as u32];
						
						// let r = (ds_builder.clone())(surface.window().inner_size(), device.clone(), queue.clone(), layout.clone());
						let inner_size = surface.window().inner_size();
						let r = (resize.clone())(inner_size, device.clone(), queue.clone(), layout.clone());
						
						image_save_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, // Buffer which will be used to save rendered images
						(0 .. inner_size.width * inner_size.height * 4).map(|_| 255u8))
							.expect("failed to create image_save_buffer");

						save_image = StorageImage::with_usage( // Image which is potentially used to save on the disk
							device.clone(), ImageDimensions::Dim2d { width: inner_size.width, height: inner_size.height, array_layers: 1 }, Format::R8G8B8A8Unorm,
							ImageUsage {
								transfer_source: true,
								transfer_destination: true,
								.. ImageUsage::none()
							}, 
							ImageCreateFlags::none(),
							vec![queue.family()]
						).unwrap();
						
						descriptor_set = r.0;
						output_img = r.1;
						dispatch = r.2;
						update = r.3;

						dest_dim = [_dest_dim.width() as i32, _dest_dim.height() as i32, 1];
						output_dim = [_output_dim[0] as i32, _output_dim[1] as i32, 1];
					}

					let (swap_index, suboptimal, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None) {
						Ok(r) => r,
						Err(swapchain::AcquireError::OutOfDate) => {
							resized = true;
							return;
						},
						Err(e) => panic!("Failed to acquire next image: {:?}", e) 
					};

					if suboptimal { // Rebuild if suboptimal
						resized = true;
					}
					
					dest_image = images[swap_index].clone();

					let mut cb_builder = AutoCommandBufferBuilder::primary(device.clone(), queue.family()).unwrap();
					
					cb_builder
						.clear_color_image(output_img.clone(), ClearValue::Float([0.0, 0.0, 0.0, 1.0])).unwrap()
						.dispatch(dispatch, compute_pipeline.clone(), descriptor_set.clone(), push_constants, std::iter::empty()).unwrap()
						.blit_image(
							output_img.clone(),
							[0, 0, 0],
							output_dim,
							0,
							0,
							save_image.clone(),
							[0, 0, 0],
							dest_dim,
							0,
							0,
							1,
							Filter::Nearest
						).unwrap()
						.blit_image(
							output_img.clone(),
							[0, 0, 0],
							output_dim,
							0,
							0,
							dest_image.clone(),
							[0, 0, 0],
							dest_dim,
							0,
							0,
							1,
							Filter::Nearest
						).unwrap();

					let cb = cb_builder.build().unwrap();
					
					let _future = previous_frame_end
						.take().unwrap()
						.join(acquire_future);
					
					let future = _future.then_execute(queue.clone(), cb).unwrap()
						.then_swapchain_present(queue.clone(), swapchain.clone(), swap_index)
						.then_signal_fence_and_flush();
					
					match future {
						Ok(future) => {
							future.wait(None).unwrap();
							previous_frame_end = Some(future.boxed());
						},
						Err(vulkano::sync::FlushError::OutOfDate) => {
							resized = true;
							previous_frame_end = Some(sync::now(device.clone()).boxed());
						},
						Err(e) => println!("Failed to flush future: {:?}", e)
					}
				},
				_ => ()
			}
		})
	}
}