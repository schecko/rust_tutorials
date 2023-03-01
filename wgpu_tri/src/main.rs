use winit::{ 
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const SHADER: &'static str = r##"
    struct VertexOutput
    {
        @builtin(position) clip_position: vec4<f32>,
    }

    struct VertexInput
    {
        @builtin(vertex_index) vertex_index: u32,
    }

    @vertex
    fn vs_main(in: VertexInput) -> VertexOutput
    {
        var out: VertexOutput;
        let x = f32(1 - i32(in.vertex_index)) * 0.5;
        let y = f32(i32(in.vertex_index & 1u) * 2 - 1) * 0.5;
        out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
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
        out.color = vec4<f32>(0.5, 0.5, 0.5, 1.0);
        return out;
    }
"##;

fn main()
{
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
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
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
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
                render_pass.draw(0..3, 0..1);
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
