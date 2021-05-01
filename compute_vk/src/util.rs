
use vulkano::device::{Device, Queue, Features, DeviceExtensions};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice, ApplicationInfo};
use vulkano::image::{ImageDimensions, StorageImage, ImageUsage, ImageCreateFlags};
use vulkano::format::{Format, ClearValue};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer};
use vulkano::sync::GpuFuture;

use std::sync::Arc;

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

	// Copy random data for debugging
	let mut cb_builder = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap();
	cb_builder
		.clear_color_image(output.clone(), ClearValue::Float([1.0, 0.0, 1.0, 1.0])).unwrap();
		
	let cb = cb_builder.build().unwrap();

	let finished = cb.execute(queue.clone()).unwrap();
	finished.then_signal_fence_and_flush().unwrap()
		.wait(None).unwrap();

	output
}
