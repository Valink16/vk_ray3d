use vulkano::device::{Device, Queue, Features, DeviceExtensions};
use vulkano::instance::{Instance, PhysicalDevice, ApplicationInfo};
use vulkano::image::{ImageDimensions, StorageImage, ImageUsage, ImageCreateFlags};
use vulkano::buffer::{CpuAccessibleBuffer, DeviceLocalBuffer, BufferUsage, BufferAccess};
use vulkano::format::{Format, ClearValue};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer};
use vulkano::sync::GpuFuture;
use vulkano::memory::Content;

use std::sync::Arc;
use std::any::type_name;

pub fn init_vulkano(app_info: &ApplicationInfo) -> (Arc<Instance>, Arc<Device>, Arc<Queue>) {
	let instance = Instance::new(
		Some(app_info),
		&vulkano_win::required_extensions(),
		None
	).expect("Failed to create Vulkan instance");

	// Getting the physical devices
	let mut _device = PhysicalDevice::enumerate(&instance)
		.next().expect("No devices available");

	println!("Using device {}, type: {:?}", _device.name(), _device.ty());

	// Find a queue_family supporting graphics
	let queue_family = _device.queue_families().find(|&q| {
		q.supports_graphics()
	}).expect("Couldn't find any queue family supporting graphical operations");

	let _features = Features {
		shader_storage_image_extended_formats: true,
		.. Features::none()
	};

	let _ext = DeviceExtensions {
		khr_storage_buffer_storage_class: true, // Needed for creating buffers on GPU memory
		khr_swapchain: true,
		.. DeviceExtensions::none()
	};

	let (device, mut _queues) = Device::new(_device, &_features, &_ext, [(queue_family, 0.5)].iter().cloned())
		.expect("Failed to create a device");

	// Getting a queue
	let queue = _queues.next().unwrap();

	(instance, device, queue)
}

/// Creates the output image and returns the arc
pub fn build_image(device: Arc<Device>, queue: Arc<Queue>, size: ImageDimensions, format: Format) -> Arc<StorageImage<Format>> {
	let output = StorageImage::with_usage(
		device.clone(),
		size,
		format,
		ImageUsage {
			// sampled: true, // To be able to read from it from a shader
			transfer_destination: true, // To be able to copy a buffer to it (debug reasons, may remove later)
			transfer_source: true, // To be able to blit from this to the swapchain image
			storage: true, // To be able to use this in a shader
			.. ImageUsage::none()
		},
		ImageCreateFlags::none(),
		Some(queue.family())
	).expect("Failed to create output image");

	// Copy some data for debugging
	let mut cb_builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
	cb_builder
		.clear_color_image(output.clone(), ClearValue::Float([1.0, 0.0, 1.0, 1.0])).unwrap();
		
	let cb = cb_builder.build().unwrap();

	let finished = cb.execute(queue.clone()).unwrap();
	finished.then_signal_fence_and_flush().unwrap()
		.wait(None).unwrap();

	output
}

/// Builds and returns a `CpuAccessibleBuffer` as an `Arc` using any data that can be iterated
pub fn build_cpu_buffer<T>(device: Arc<Device>, usage: BufferUsage, data: T) -> Result<Arc<CpuAccessibleBuffer<[<T as IntoIterator>::Item]>>, String> where
T: IntoIterator + 'static, T::IntoIter: ExactSizeIterator, T::Item: Content + Send + Sync + 'static {
	let data_iter = data.into_iter();
	let data_length = data_iter.len();
	match CpuAccessibleBuffer::from_iter(device, usage, true, data_iter) {
		Ok(b) => {
			println!("Created {} {}s using {} bytes", data_length, type_name::<T::Item>(), b.size());
			Ok(b)
		},
		Err(e) => Err(String::from(format!("Failed to create the cpu accessible buffer, {:?}", e)))
	}
}

/// Builds and returns a `DeviceLocalBuffer` as an `Arc` using any data that can be iterated
pub fn build_local_buffer<T>(device: Arc<Device>, queue: Arc<Queue>, usage: BufferUsage, data: T) -> Result<Arc<DeviceLocalBuffer<[<T as IntoIterator>::Item]>>, String> where
T: IntoIterator + 'static, T::IntoIter: ExactSizeIterator, T::Item: Content + Copy + Send + Sync + 'static {
	let data_iter = data.into_iter();
	let data_length = data_iter.len();
	match CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), true, data_iter) {
		Ok(source) => {
			let dest = match DeviceLocalBuffer::<[<T as IntoIterator>::Item]>::array(device.clone(), data_length, usage, vec![queue.family()].into_iter()) {
				Ok(d) => d,
				Err(e) => return Err(String::from(format!("Failed to create the local device buffer, {:?}", e)))
			};
            
            // Fill the device local buffer
            let mut cb_builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
            cb_builder
                .copy_buffer(source.clone(), dest.clone()).unwrap();
            
            let cb = cb_builder.build().unwrap();
            let exec_future = cb.execute(queue.clone()).unwrap();

            exec_future
                .then_signal_fence_and_flush().unwrap()
                .wait(None).unwrap();

            println!("Created {} {}s using {} bytes", data_length, type_name::<T::Item>(), dest.size());

			Ok(dest)
		},
		Err(e) => Err(String::from(format!("Failed to create the cpu accessible buffer, {:?}", e)))
	}
}
