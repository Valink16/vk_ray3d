use std::path::Path;
use std::sync::Arc;

use crate::vulkano;

use vulkano::device::{Device, Queue};
use vulkano::image::{view::ImageView, immutable::ImmutableImage, ImageDimensions, MipmapsCount};
use vulkano::sampler::Sampler;
use vulkano::format::Format;
use vulkano::sync::GpuFuture;


pub fn load_texture<P>(path: P, _device: Arc<Device>, _queue: Arc<Queue>) -> (Arc<ImageView<Arc<ImmutableImage<Format>>>>, Arc<Sampler>) 
where P: AsRef<Path> {
	let base_texture_image = image::open(path).unwrap().into_rgba8();
	let (w, h) = base_texture_image.dimensions();
	
	println!("W: {}, H: {}", w, h);

	let base_texture_data = base_texture_image.as_raw();

	let (base_texture, init) = ImmutableImage::from_iter(base_texture_data.iter().cloned(), ImageDimensions::Dim2d { width: w, height: h, array_layers: 1}, MipmapsCount::One, Format::R8G8B8A8Unorm, _queue.clone()).unwrap();
	init.then_signal_fence_and_flush().unwrap()
		.wait(None).unwrap();

	let base_texture_view = ImageView::new(base_texture).unwrap();
	let base_texture_sampler = Sampler::simple_repeat_linear(_device.clone());

	(base_texture_view, base_texture_sampler)
}