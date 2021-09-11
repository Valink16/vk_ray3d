use compute_vk::vulkano::command_buffer::CommandBuffer;
use compute_vk::vulkano::sync::GpuFuture;
use compute_vk::{self, loader, util, vulkano, winit};
use winit::event::Event;
use nalgebra_glm::Vec3;
use image::{self, GenericImageView};

use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageDimensions, ImmutableImage, MipmapsCount, ImageUsage, ImageCreateFlags, ImageLayout};
use vulkano::sampler::Sampler;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use winit::{dpi::PhysicalSize, event};

use std::{f32::consts::PI, sync::Arc};

use crate::{geom::sphere::Sphere, quaternion::Quaternion};

mod ray;
mod geom;
mod light;
mod camera;
mod quaternion;
mod texture;

fn main() {
    let mut vertices = Vec::<[f32; 4]>::new();
    let mut indices = Vec::<[u32; 4]>::new();
    
    let models: Vec<geom::model::Model> = vec![
        // geom::model::Model::new("STL/cube.stl", [-2.0, 0.0, 10.0], [1.0, 1.0, 1.0, 1.0], 0.5, 0.5, &mut vertices, &mut indices),
        geom::model::Model::new("STL/cube.stl", [2.0, 0.0, 10.0], [1.0, 1.0, 1.0, 1.0], 0.1, 0.9, &mut vertices, &mut indices),
        // geom::model::Model::new("STL/pyramid.stl", [-2.0, -1.0, 10.0], [1.0, 1.0, 1.0, 1.0], 0.9, 0.1, &mut vertices, &mut indices),
        // geom::model::Model::new("STL/monkey.stl", [0.0, 1.0, 8.0], [1.0, 1.0, 1.0, 1.0], 0.5, 0.5, &mut vertices, &mut indices),
        // geom::model::Model::new("STL/ground.stl", [0.0, -1.0, 10.0], [1.0, 1.0, 1.0, 1.0], 0.0, 1.0, &mut vertices, &mut indices),
    ];

    /*
    let models = vec![
        geom::model::Model {
            pos: [0.0, 0.0, 10.0, 0.0],
            indices_start: 0,
            indices_end: 2,
            _pad: [0; 2]
        }
    ];

    vertices.push([-1.0, 1.0, 0.0, 0.0]);
    vertices.push([-1.0, -1.0, 0.0, 0.0]);
    vertices.push([10.0, -1.0, 0.0, 0.0]);
    vertices.push([10.0, 1.0, 0.0, 0.0]);
    indices.push([0, 1, 2, 0]);
    indices.push([0, 2, 3, 0]);
    */

    let ds_builder = move |_size: PhysicalSize<u32>, _device: Arc<Device>, _queue: Arc<Queue>, _layout| {
        let scale = 1.0;
        let camera_speed = 0.5;
        let _size = PhysicalSize::new((_size.width as f32 * scale) as u32, (_size.height as f32 * scale) as u32);
        let bu = BufferUsage {
            transfer_destination: true,
            storage_buffer: true,
            .. BufferUsage::none()
        };

        let output_img = util::build_image(_device.clone(), _queue.clone(),
            ImageDimensions::Dim2d { width: _size.width, height: _size.height, array_layers: 1 },
            vulkano::format::Format::B8G8R8A8Unorm
        );

        let output_img_view = ImageView::new(output_img.clone()).unwrap();

        let ray_buffer = {
            let rays = ray::RayIter::new(_size, std::f32::consts::PI * 0.5);
            util::build_local_buffer(_device.clone(), _queue.clone(), BufferUsage::all(), rays).unwrap()
        };

        let sphere_buffer = {
            let mut spheres = vec![
                Sphere::new([0.0, 0.0, 20.0], [0.0, 0.0, 1.0, 1.0], 3.0, 0.5, 0.5, 0),
            ];

            let s = 10;
            for i in 0..s {
                let angle = i as f32 * (2.0 * PI / s as f32);
                spheres.push(Sphere::new([angle.cos() * 4.0, 0.0, angle.sin() * 4.0 + 20.0], [1.0, 1.0, 1.0, 1.0], 0.5, 0.5, 0.5, 1));
            }
    
            util::build_cpu_buffer(_device.clone(), bu, spheres).unwrap()
        };

        let model_buffer = {
            util::build_cpu_buffer(_device.clone(), BufferUsage::all(), models).unwrap()
        };

        let vertex_buffer = util::build_local_buffer(_device.clone(), _queue.clone(), BufferUsage::all(), vertices).unwrap();
        let indice_buffer = util::build_local_buffer(_device.clone(), _queue.clone(), BufferUsage::all(), indices).unwrap();

        let light_buffer = {
            let lights: Vec::<light::PointLight> = vec![
                // light::PointLight::new(Vec3::new(0.0, 10.0, 10.0), Vec3::new(1.0, 1.0, 1.0), 3.0),
                light::PointLight::new(Vec3::new(0.0, 5.0, 10.0), Vec3::new(1.0, 1.0, 1.0), 50.0),
                light::PointLight::new(Vec3::new(-10.0, 10.0, 5.0), Vec3::new(0.0, 0.0, 1.0), 50.0),
                // light::PointLight::new(Vec3::new(0.0, 10.0, 5.0), Vec3::new(0.0, 0.0, 1.0), 20.0),
                // light::PointLight::new(Vec3::new(10.0, 10.0, 10.0), Vec3::new(0.0, 1.0, 0.0), 100.0),
            ];

            util::build_cpu_buffer(_device.clone(), bu, lights).unwrap()
        };

        let dir_light_buffer = {
            let dir_lights: Vec::<light::DirectionalLight> = vec![
                light::DirectionalLight::new(Vec3::new(-1.0, -1.0, -1.0).normalize(), Vec3::new(1.0, 1.0, 1.0)),
                // light::DirectionalLight::new(Vec3::new(-10.0, 10.0, 5.0), Vec3::new(0.0, 0.0, 1.0)),
            ];

            util::build_cpu_buffer(_device.clone(), bu, dir_lights).unwrap()
        };

        let (base_texture_view, base_texture_sampler) = texture::load_texture("Images/blue.png", _device.clone(), _queue.clone());
        let (bw_texture_view, bw_texture_sampler) = texture::load_texture("Images/grid.jpg", _device.clone(), _queue.clone());
        
        let ds = PersistentDescriptorSet::start(_layout)
            .add_image(output_img_view).unwrap()
            .add_buffer(ray_buffer.clone()).unwrap()
            .add_buffer(sphere_buffer.clone()).unwrap()
            .add_buffer(model_buffer.clone()).unwrap()
            .add_buffer(vertex_buffer.clone()).unwrap()
            .add_buffer(indice_buffer.clone()).unwrap()
            .add_buffer(light_buffer.clone()).unwrap()
            .add_buffer(dir_light_buffer.clone()).unwrap()
            .enter_array().unwrap()
            .add_sampled_image(base_texture_view.clone(), base_texture_sampler.clone()).unwrap()
            .add_sampled_image(bw_texture_view.clone(), bw_texture_sampler.clone()).unwrap()
            .leave_array().unwrap()
            // .add_sampled_image(texture_view, sampler).unwrap()
            .build().unwrap();

        let dispatch = [_size.width / 8, _size.width / 8, 1];

        let mut y_angle = 0.0;
        let mut x_angle = 0.0;

        let mut camera = camera::Camera { // Used as push constant
            pos: [0.0, 0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0, 1.0]
        };

        let mut t: f32 = 0.0;
        let update = move |ev: Option<&Event<()>>| {
            let ev = match ev {
                Some(e) => e,
                None => return (camera, false)
            };

            match ev {
                event::Event::DeviceEvent { event, .. } => {
                    let mut camera_movement = Vec3::new(0.0, 0.0, 0.0);
                    match event {
                        event::DeviceEvent::MouseMotion { delta } => {
                            x_angle += delta.1 as f32 * 0.001;
                            y_angle += delta.0 as f32 * 0.001;
                        },
                        event::DeviceEvent::Key(kb_input) => {
                            dbg!(kb_input.scancode);
                            
                            match kb_input.scancode {
                                17 => camera_movement.z += camera_speed, // W
                                31 => camera_movement.z -= camera_speed, // S
                                30 => camera_movement.x -= camera_speed, // A
                                32 => camera_movement.x += camera_speed, // D
                                57 => camera_movement.y += camera_speed, // Space
                                29 => camera_movement.y -= camera_speed, // CTRL
                                _ => ()
                            }
                        }
                        _ => ()
                    }

                    // Update camera
                    let x_axis = Quaternion::from_axis(Vec3::new(1.0, 0.0, 0.0), x_angle);
                    let y_axis = Quaternion::from_axis(Vec3::new(0.0, 1.0, 0.0), y_angle);
                    let camera_quat = y_axis * x_axis;
                    camera.orientation = camera_quat.into(); // Defining camera orientation quaternion
                    let camera_vel = camera_quat.transform_point(camera_movement); // Moving the camera in its looking direction
                    camera.pos[0] += camera_vel.x;
                    camera.pos[1] += camera_vel.y;
                    camera.pos[2] += camera_vel.z;
                },
                event::Event::RedrawEventsCleared => { // Animation things
                    t += 0.001;
                    match sphere_buffer.write() {
                        Ok(mut sb) => {
                            let r = Quaternion::from_axis(Vec3::new(0.0, 1.0, 0.0), 0.001);
                            // let r = Quaternion::new(0.0, 0.0, 0.0, 1.0);

                            for i in 1..sb.len() {
                                let mut pos = Vec3::new(sb[i].pos[0], sb[i].pos[1], sb[i].pos[2]);
                                pos = r.transform_around(pos, [0.0, 0.0, 20.0].into());
                                sb[i].pos = [pos.x, pos.y, pos.z, 0.0];
                            }
                        },
                        _ => ()
                    }

                    match light_buffer.write() {
                        Ok(mut lb) => {
                            /*
                            let r = Quaternion::from_axis([0.0, 1.0, 0.0].into(), 0.001);

                            let mut pos = Vec3::new(lb[1].pos[0], lb[1].pos[1], lb[1].pos[2]);
                            pos = r.transform_around(pos, [0.0, 0.0, 20.0].into());

                            lb[1].pos = [
                                pos.x,
                                pos.y,
                                pos.z + 20.0,
                                0.0
                            ];
                            */
                        },
                        _ => ()
                    }
                },
                _ => (),
            }

            (camera, false)
        };

        (Arc::new(ds), output_img, dispatch, update)
    };

    let win_size = PhysicalSize::new(800, 600);

    let mut canvas = compute_vk::canvas::Canvas::new(win_size, ds_builder, &vulkano::app_info_from_cargo_toml!());
    
    let mut shader_layout = loader::MainLayout::new();
    shader_layout.add_image(0);
    shader_layout.add_buffer(0, false);
    shader_layout.add_buffer(0, false);
    shader_layout.add_buffer(0, false);
    shader_layout.add_buffer(0, false);
    shader_layout.add_buffer(0, false);
    shader_layout.add_buffer(0, false);
    shader_layout.add_buffer(0, false);
    shader_layout.add_sampled_image_array(0, 2, true);
    // shader_layout.add_buffer(0, false);
    shader_layout.add_push_constant_range(0, 32);

    let shader = loader::Shader::load(canvas.device.clone(), "shader/ray3d.glsl", shader_layout)
        .expect("Failed to load the shader");
    canvas.set_shader(shader);

    canvas.run().unwrap();
}
