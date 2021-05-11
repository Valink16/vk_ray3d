use compute_vk::{self, loader, util, vulkano, winit};
use winit::event::Event;
use nalgebra_glm::Vec3;

use vulkano::device::Device;
use vulkano::device::Queue;
use vulkano::image::view::ImageView;
use vulkano::image::ImageDimensions;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::buffer::BufferUsage;
use winit::{dpi::PhysicalSize, event};

use std::sync::Arc;

use crate::{geom::sphere::Sphere, quaternion::Quaternion};

mod ray;
mod geom;
mod light;
mod camera;
mod quaternion;

fn main() {
        let ds_builder = move |_size: PhysicalSize<u32>, _device: Arc<Device>, _queue: Arc<Queue>, _layout| {
        let scale = 1.0;
        let camera_speed = 0.5;
        let _size = PhysicalSize::new((_size.width as f32 * scale) as u32, (_size.height as f32 * scale) as u32);
        
        let output_img = util::build_image(_device.clone(), _queue.clone(),
            ImageDimensions::Dim2d { width: _size.width, height: _size.height, array_layers: 1 },
            vulkano::format::Format::B8G8R8A8Unorm
        );
        let output_img_view = ImageView::new(output_img.clone()).unwrap();

        let ray_buffer = {
            let rays = ray::RayIter::new(_size, std::f32::consts::FRAC_PI_2);
            util::build_local_buffer(_device.clone(), _queue.clone(), BufferUsage::all(), rays).unwrap()
        };

        let sphere_buffer = {
            let spheres = vec![
                Sphere::new([0.0, 0.0, 20.0, 1.0], [1.0, 1.0, 1.0, 1.0], 3.0),
                Sphere::new([0.0, 0.0, 15.0, 1.0], [1.0, 1.0, 1.0, 1.0], 0.5),
            ];
    
            util::build_cpu_buffer(_device.clone(), BufferUsage::all(), spheres).unwrap()
        };

        let light_buffer = {
            let lights = vec![
                light::PointLight::new(Vec3::new(-3.5, 3.5, 0.0), Vec3::new(1.0, 0.0, 0.0), 300.0),
                light::PointLight::new(Vec3::new(3.5, -3.5, 0.0), Vec3::new(0.0, 0.0, 1.0), 300.0),
                // light::PointLight::new(Vec3::new(0.0, 0.0, 2.0), Vec3::new(1.0, 1.0, 1.0), 5000.0),
                // light::PointLight::new(Vec3::new(0.0, -20.0, 20.0), Vec3::new(1.0, 1.0, 1.0), 10000.0),
            ];

            util::build_cpu_buffer(_device.clone(), BufferUsage::all(), lights).unwrap()
        };

        let ds = PersistentDescriptorSet::start(_layout)
            .add_image(output_img_view).unwrap()
            .add_buffer(ray_buffer.clone()).unwrap()
            .add_buffer(sphere_buffer.clone()).unwrap()
            .add_buffer(light_buffer.clone()).unwrap()
            .build().unwrap();

        let dispatch = [_size.width / 8 + 1, _size.width / 8 + 1, 1];

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
                                _ => ()
                            }
                        }
                        _ => ()
                    }

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
                            sb[1].pos[2] = t.cos() * 5.0 + 20.0;
                            sb[1].pos[0] = t.sin() * 5.0;
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
    shader_layout.add_push_constant_range(0, 32);

    let shader = loader::Shader::load(canvas.device.clone(), "shader/ray3d.glsl", shader_layout)
        .expect("Failed to load the shader");
    canvas.set_shader(shader);

    canvas.run().unwrap();
}
