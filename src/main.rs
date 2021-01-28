#![allow(dead_code)]
mod support;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::ContextBuilder;
use support::{load, GL_CONTEXT};

fn main() {
    let el = EventLoop::new();
    let wb = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_inner_size(glutin::dpi::PhysicalSize::new(800, 600))
        .with_decorations(true);

    let windowed_context = ContextBuilder::new().build_windowed(wb, &el).unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    println!(
        "Pixel format of the window's GL context: {:?}",
        windowed_context.get_pixel_format()
    );

    load(&windowed_context.context());

    let mut redraw = 0;
    let gl = unsafe { GL_CONTEXT.take_context() };
    el.run(move |event, _, control_flow| {
        // println!("{:?}", event);
        *control_flow = ControlFlow::Wait;

        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                println!("{:?}", event);
                if redraw != -1 {
                    // gl.draw_frame(Color { red: 1.0, green: 0.5, blue: 0.7, alpha: 1.0 });
                    gl.viewport();

                    gl.rect().unwrap_or_else(|err| {
                        println!("{}", err);
                    });
                } else {
                    gl.viewport();
                }
                windowed_context.swap_buffers().unwrap();
                redraw = redraw + 1;
            }
            _ => (),
        }
    });
}
