
extern crate cgmath;
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate genmesh;
extern crate amethyst_renderer;
extern crate rand;

use std::time::SystemTime;
use rand::Rng;

use gfx::{Device};
use gfx::traits::FactoryExt;

use cgmath::{Point3, Vector3, Matrix4, EuclideanVector};
use cgmath::{Transform, AffineMatrix3};
use genmesh::generators::SphereUV;
use genmesh::{Triangulate, MapToVertices, Vertices};

use amethyst_renderer::VertexPosNormal as Vertex;
use amethyst_renderer::{ColorFormat, DepthFormat};

fn build_sphere() -> Vec<Vertex> {
    SphereUV::new(16, 16)
        .vertex(|(x, y, z)| Vertex{
            pos: [x, y, z],
            normal: Vector3::new(x, y, z).normalize().into()
        })
        .triangulate()
        .vertices()
        .collect()
}

fn main() {
    let builder = glutin::WindowBuilder::new()
        .with_title("Amethyst Renderer Demo".to_string())
        .with_dimensions(800, 600)
        .with_vsync();

    let (window, mut device, mut factory, main_color, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let combuf = factory.create_command_buffer();

    let sphere = build_sphere();
    let (buffer, slice) = factory.create_vertex_buffer(&sphere);

    let mut scene = amethyst_renderer::Scene{
        fragments: vec![],
        lights: vec![]
    };

    let mut rng = rand::thread_rng();

    for x in -1..2 {
        for y in -1..2 {
            for z in -1..2 {
                let x = x as f32 * 4.;
                let y = y as f32 * 4.;
                let z = z as f32 * 4.;

                let color = [rng.gen_range(0., 1.), rng.gen_range(0., 1.), rng.gen_range(0., 1.), 1.];

                scene.fragments.push(amethyst_renderer::Fragment{
                    buffer: buffer.clone(),
                    slice: slice.clone(),
                    ka: [color[0] * 0.1, color[1] * 0.1, color[2] * 0.1, 1.],
                    kd: color,
                    transform: Matrix4::from_translation(Vector3::new(x, y, z)).into()
                })
            }
        }
    }

    for x in -2..3 {
        for y in -2..3 {
            for z in -2..3 {
                let x = x as f32 * 5.;
                let y = y as f32 * 5.;
                let z = z as f32 * 5.;

                let r = (x + 10.) / 20.;
                let g = (y + 10.) / 20.;
                let b = (z + 10.) / 20.;

                scene.lights.push(amethyst_renderer::Light{
                    color: [r, g, b, 1.],
                    radius: 1.,
                    center: [x, y, z],
                    propagation_constant: 0.,
                    propagation_linear: 0.,
                    propagation_r_square: 1.,
                })
            }
        }
    }


    let mut frame = amethyst_renderer::Frame{
        passes: vec![
            Box::new(amethyst_renderer::Clear{color: [0.1, 0.1, 0.1, 1.]}),
            Box::new(amethyst_renderer::FlatShading{
                camera: format!("main"),
                scene: format!("main")
            }),
        ],
        target: amethyst_renderer::ScreenOutput{
            output: main_color,
            output_depth: main_depth
        },
        scenes: std::collections::HashMap::new(),
        cameras: std::collections::HashMap::new()
    };

    frame.scenes.insert(format!("main"), scene);

    let mut renderer = amethyst_renderer::Renderer::new(combuf);

    renderer.add_method(amethyst_renderer::forward::Clear);
    renderer.add_method(amethyst_renderer::forward::FlatShading::new(&mut factory));
    renderer.add_method(amethyst_renderer::forward::Wireframe::new(&mut factory));

    let start = SystemTime::now();
    let (mut w, mut h) = (800., 600.);
    'main: loop {
        // quit when Esc is pressed.
        for event in window.poll_events() {
            match event {
                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Space)) => {
                    frame.passes = vec![
                        Box::new(amethyst_renderer::Clear{color: [0.1, 0.1, 0.1, 1.]}),
                        Box::new(amethyst_renderer::Wireframe{
                            camera: format!("main"),
                            scene: format!("main")
                        }),
                    ]
                }
                glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Space)) => {
                    frame.passes = vec![
                        Box::new(amethyst_renderer::Clear{color: [0.1, 0.1, 0.1, 1.]}),
                        Box::new(amethyst_renderer::FlatShading{
                            camera: format!("main"),
                            scene: format!("main")
                        }),
                    ]
                }
                glutin::Event::Resized(iw, ih) => {
                    let output = &mut frame.target;
                    w = iw as f32;
                    h = ih as f32;
                    gfx_window_glutin::update_views(
                        &window,
                        &mut output.output,
                        &mut output.output_depth
                    );
                }
                glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                glutin::Event::Closed => break 'main,
                _ => {},
            }
        }

        let diff = start.elapsed().unwrap();
        let diff = diff.as_secs() as f32 + diff.subsec_nanos() as f32 / 1e9;
        let view: AffineMatrix3<f32> = Transform::look_at(
            Point3::new(diff.sin() * 6., diff.cos() * 6., 3.0),
            Point3::new(0f32, 0.0, 0.0),
            Vector3::unit_z(),
        );
        let proj = cgmath::perspective(cgmath::deg(60.0f32), w / h, 1.0, 100.0);
        frame.cameras.insert(
            format!("main"),
            amethyst_renderer::Camera{projection: proj.into(), view: view.mat.into()}
        );

        renderer.submit(&frame, &mut device);
        window.swap_buffers().unwrap();
    }
}