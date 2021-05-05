use compute_vk::{self, loader, util, vulkano, winit};
use nalgebra_glm::Vec3;

use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::ImageDimensions;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::buffer::{DeviceLocalBuffer, CpuAccessibleBuffer, BufferUsage, BufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer};
use vulkano::sync::GpuFuture;

use winit::dpi::PhysicalSize;

use std::sync::Arc;

mod ray;
mod geom;
mod light;

fn main() {
    let win_size = PhysicalSize::new(800, 600);

    let ds_builder = move |_size: PhysicalSize<u32>, _device: Arc<Device>, _queue: Arc<Queue>, _layout| {
        let scale = 1.0;
        let _size = PhysicalSize::new((_size.width as f32 * scale) as u32, (_size.height as f32 * scale) as u32);
        
        let output_img = util::build_image(_device.clone(), _queue.clone(),
            ImageDimensions::Dim2d { width: _size.width, height: _size.height, array_layers: 1 },
            vulkano::format::Format::B8G8R8A8Unorm
        );
        let output_img_view = ImageView::new(output_img.clone()).unwrap();

        let ray_buffer = {
            let rays = ray::RayIter::new(_size, std::f32::consts::FRAC_PI_2);
            let rays_len = rays.len();

            let source = CpuAccessibleBuffer::from_iter(_device.clone(), BufferUsage::all(), false, rays)
                .expect("Failed to create the source buffer");
            
            let dest = DeviceLocalBuffer::<[ray::Ray]>::array(_device.clone(), rays_len, BufferUsage::all(), vec![_queue.family()].into_iter()).unwrap();
            
            // Fill the device local buffer
            let mut cb_builder = AutoCommandBufferBuilder::new(_device.clone(),  _queue.family()).unwrap();
            cb_builder
                .copy_buffer(source.clone(), dest.clone()).unwrap();
            
            let cb = cb_builder.build().unwrap();
            let exec_future = cb.execute(_queue.clone()).unwrap();

            exec_future
                .then_signal_fence_and_flush().unwrap()
                .wait(None).unwrap();

            println!("Created {} rays using {} MB", rays_len, dest.size() / 1_000_000);
            
            dest
        };

        let sphere_buffer = {
            let sphere_count = 20.0;
            let mut spheres: Vec<geom::Sphere> = geom::SphereIter::new(sphere_count as u32, 0.5).collect();
            spheres.push(geom::Sphere::new(
                [0.0, 0.0, 20.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                10.0
            ));
            let spheres_len = spheres.len();
    
            let dest = CpuAccessibleBuffer::from_iter(_device.clone(), BufferUsage::all(), false, spheres.iter().copied()).unwrap();
            /*
            let source = CpuAccessibleBuffer::from_iter(_device.clone(), BufferUsage::all(), false, spheres.iter().copied()).unwrap();
            let dest = DeviceLocalBuffer::<[geom::Sphere]>::array(_device.clone(), spheres_len, BufferUsage::all(), vec![_queue.family()].into_iter()).unwrap();

            // Fill the device local buffer
            let mut cb_builder = AutoCommandBufferBuilder::new(_device.clone(),  _queue.family()).unwrap();
            cb_builder
                .copy_buffer(source.clone(), dest.clone()).unwrap();
            
            let cb = cb_builder.build().unwrap();
            let exec_future = cb.execute(_queue.clone()).unwrap();

            exec_future
                .then_signal_fence_and_flush().unwrap()
                .wait(None).unwrap();

            println!("Created {} spheres using {} Bytes", spheres_len, dest.size());
            */

            println!("Created {} spheres using {} Bytes", spheres_len, dest.size());

            dest
        };

        let light_buffer = {
            let lights = vec![
                light::PointLight::new(Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), 5000.0),
                // light::PointLight::new(Vec3::new(5.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), 5000.0),
                // light::PointLight::new(Vec3::new(0.0, 0.0, 2.0), Vec3::new(1.0, 1.0, 1.0), 5000.0),
                light::PointLight::new(Vec3::new(0.0, -20.0, 20.0), Vec3::new(1.0, 1.0, 1.0), 10000.0),
            ];

            let light_count = lights.len();

            let dest = CpuAccessibleBuffer::from_iter(_device.clone(), BufferUsage::all(), true, lights.iter().copied()).unwrap();
            
            /*
            let source = CpuAccessibleBuffer::from_iter(_device.clone(), BufferUsage::all(), true, lights.iter().copied()).unwrap();
            let dest = DeviceLocalBuffer::<[light::PointLight]>::array(_device.clone(), light_count, BufferUsage::all(), vec![_queue.family()].into_iter()).unwrap();
            
            // Fill the device local buffer
            let mut cb_builder = AutoCommandBufferBuilder::new(_device.clone(),  _queue.family()).unwrap();
            cb_builder
                .copy_buffer(source.clone(), dest.clone()).unwrap();
            
            let cb = cb_builder.build().unwrap();
            let exec_future = cb.execute(_queue.clone()).unwrap();

            exec_future
                .then_signal_fence_and_flush().unwrap()
                .wait(None).unwrap();

            println!("Created {} spheres using {} Bytes", light_count, dest.size());
            */
            
            println!("Created {} lights using {} Bytes", light_count, dest.size());

            dest
        };

        let ds = PersistentDescriptorSet::start(_layout)
            .add_image(output_img_view).unwrap()
            .add_buffer(ray_buffer.clone()).unwrap()
            .add_buffer(sphere_buffer.clone()).unwrap()
            .add_buffer(light_buffer.clone()).unwrap()
            .build().unwrap();

        let dispatch = [_size.width / 8 + 1, _size.width / 8 + 1, 1];

        let mut t: f32 = 0.0;
        let update = move |ev: winit::event::Event<()>| {

            match light_buffer.write() {
                Ok(mut lb) => {
                    t += 0.02;
                    lb[0].pos[0] = 10.0 * t.cos();
                },
                _ => ()
            }    

            false
        };

        (Arc::new(ds), output_img, dispatch, update)
    };

    let mut canvas = compute_vk::canvas::Canvas::new(win_size, ds_builder, &vulkano::app_info_from_cargo_toml!());
    
    let mut shader_layout = loader::MainLayout::new();
    shader_layout.add_image(0);
    shader_layout.add_buffer(0, false);
    shader_layout.add_buffer(0, false);
    shader_layout.add_buffer(0, false);

    let shader = loader::Shader::load(canvas.device.clone(), "shader/ray3d.glsl", shader_layout)
        .expect("Failed to load the shader");
    canvas.set_shader(shader);

    canvas.run().unwrap();
}
