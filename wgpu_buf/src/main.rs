#![windows_subsystem = "windows"]

use winit::{ 
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wgpu::util::DeviceExt;
use bytemuck::{Zeroable, Pod};

const SHADER: &'static str = r##"
    struct VertexInput
    {
        @location(0) position: vec3<f32>,
        @location(1) color: u32,
    }

    struct VertexOutput
    {
        @builtin(position) clip_position: vec4<f32>,
        @location(0) color: vec4<f32>,
    }

    @vertex
    fn vs_main(in: VertexInput) -> VertexOutput
    {
        var out: VertexOutput;
        out.clip_position = vec4<f32>(in.position, 1.0);
        out.color = vec4<f32>((vec4<u32>(in.color) >> vec4<u32>(0u, 8u, 16u, 24u)) & vec4<u32>(255u)) / 255.0;
        return out;
    }

    struct FragmentOutput
    {
        @location(0) color: vec4<f32>,
    }

    @fragment
    fn fs_main(in: VertexOutput) -> FragmentOutput
    {
        var out: FragmentOutput;
        out.color = in.color;
        return out;
    }
"##;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex
{
    position: [f32; 3],
    color: [u8; 4],
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [255, 0, 0, 255] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [255, 255, 0, 255] },
    Vertex { position: [0.5, -0.5, 0.0], color: [255, 0, 255, 255] },

];

fn main()
{
    //env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::DX12,
        ..Default::default()
    });
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions{
        compatible_surface: Some(&surface),
        ..Default::default()
    })).unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            ..Default::default()
        },
        None
    )).unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = *surface_caps.formats
        .iter()
        .filter(|f| f.describe().srgb)
        .next()
        .unwrap_or(&surface_caps.formats[0]);

    let size = window.inner_size();
    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: Vec::new(),
    };
    surface.configure(&device, &surface_config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Default Shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });
    let vertex_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        }
    );

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[
                wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint8x4],
                }
            ],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None
    });
    event_loop.run(move |event, _, control_flow|
    {
        match event
        {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() =>
            {
                match event
                {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    },
                    WindowEvent::Resized(physical_size) => {
                        if physical_size.width > 0 && physical_size.height > 0 {
                            surface_config.width = physical_size.width;
                            surface_config.height = physical_size.height;
                            surface.configure(&device, &surface_config);
                        }
                    },
                    WindowEvent::ScaleFactorChanged{ new_inner_size, .. } => {
                        if new_inner_size.width > 0 && new_inner_size.height > 0 {
                            surface_config.width = new_inner_size.width;
                            surface_config.height = new_inner_size.height;
                            surface.configure(&device, &surface_config);
                        }
                    },
                    _ => {},
                }
            },
            Event::RedrawRequested(window_id) if window_id == window.id() =>
            {
                let output = match surface.get_current_texture() {
                    Ok(output) => output,
                    _ => return,
                };
                let view = output.texture.create_view(&Default::default());
                let mut encoder = device.create_command_encoder(&Default::default());

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Default Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
                render_pass.set_pipeline(&render_pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.draw(0..VERTICES.len() as u32, 0..1);
                drop(render_pass);

                queue.submit(std::iter::once(encoder.finish()));
                output.present();
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            _ => {}
        }
    });
}
