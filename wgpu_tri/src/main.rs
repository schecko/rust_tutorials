use winit::{ 
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() {
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

                let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
