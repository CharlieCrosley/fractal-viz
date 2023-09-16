use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

mod fractals;
use fractals::Fractals;

const MIN_WIDTH: i32 = 400;
const MIN_HEIGHT: i32 = 300;


fn main() {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let monitor_size = event_loop.primary_monitor().unwrap().size(); 
    
    let window = { 
        let size = LogicalSize::new(MIN_WIDTH as f64, MIN_HEIGHT as f64); // minimum window size
        let scaled_size = LogicalSize::new(monitor_size.width as f64, monitor_size.height as f64); // initial window size
        WindowBuilder::new()
            .with_title("Fractals")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)//.with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor())))
            .with_maximized(true)
            .build(&event_loop)
            .unwrap()
    };
    let window_size = window.inner_size();
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(window_size.width as u32, window_size.height as u32, surface_texture).unwrap()
    };

    // Set the default fractal to render the Mandelbrot set
    let fractal = Fractals::Mandelbrot;
    // Set the default zoom to zero, changes when scrolling mouse wheel
    let mut zoom: f64 = 0.003;
    // When zooming we want to zoom in on the mouse position, so we need to keep track of the mouse position
    let mut offset_x: f64 = 0.0;
    let mut offset_y: f64 = 0.0;
    let zoom_amount = 5.0; // how much to zoom in/out when scrolling the mouse wheel
    let mut drag_select_start = (0.0,0.0);
    let mut drag_select_end = (0.0,0.0);
    let mut flag_render_drag_select = false;

    // store the frame when the user starts dragging the mouse to select an area to zoom in on
    // used to set the pixel buffer to the freeze frame so that the previous frames select box is removed
    let mut freeze_frame: Vec<u8> = vec![0; (4 * window_size.width * window_size.height) as usize]; 

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::RedrawRequested(_) => {
                let (width, height) = (window.inner_size().width, window.inner_size().height);
                if flag_render_drag_select {
                    // reset the pixel buffer to the freeze frame so that the previous frames select box is removed
                    pixels.frame_mut().copy_from_slice(&freeze_frame);
                    let (x1, y1) = (drag_select_start.0, drag_select_start.1);
                    let (x2, y2) = (drag_select_end.0, drag_select_end.1);
                    draw_select_box(pixels.frame_mut(), (x1, y1), (x2, y2), width as u32);
                    /* let (width, height) = (window.inner_size().width, window.inner_size().height);
                    let (x1, y1) = ((drag_select_start.0 / pixel_buffer_size.width as f32) * width as f32, 
                                             (drag_select_start.1 / pixel_buffer_size.height as f32) * height as f32);
                    let (x2, y2) = ((drag_select_end.0 / pixel_buffer_size.width as f32) * width as f32, 
                                             (drag_select_end.1 / pixel_buffer_size.height as f32) * height as f32);

                    draw_select_box(pixels.frame_mut(), (x1, y1), (x2, y2), pixel_buffer_size.width as u32); */

                } else {
                    use std::time::Instant;
                    let now = Instant::now();
                    // Generate and render the fractal here
                    //fractal_container.draw(pixels.frame_mut(), zoom as f64, offset_x, offset_y);
                    match fractal {
                        Fractals::Mandelbrot => {
                            fractals::generate_mandelbrot(pixels.frame_mut(), width as i32, height as i32, zoom, offset_x, offset_y);
                        },
                        Fractals::Julia => {
                            // Generate and render the Julia set here
                        },
                    }

                    let elapsed = now.elapsed();
                    println!("Elapsed: {:.2?}", elapsed);
                }
                
                if let Err(_) = pixels.render() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            },
            _ => {}
        }

        // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(&event) {
            
            // Handle mouse. This is a bit involved since support some simple
            // line drawing (mostly because it makes nice looking patterns).
            let scroll = input.scroll_diff();
            if scroll != 0.0 {
                let zoom_factor = 1.0 + (0.1 * zoom_amount * -scroll.signum());
                zoom *= zoom_factor as f64;
            }
            else if input.mouse_pressed(0) {
                if let Some((x,y)) = input.mouse() {
                    freeze_frame.copy_from_slice(pixels.frame());
                    drag_select_start = (x,y);
                    flag_render_drag_select = true;
                }
                
            }
            else if input.mouse_held(0) {
                if let Some((x,y)) = input.mouse() {
                    drag_select_end = (x,y);
                }
            }
            else if input.mouse_released(0) {
                // zoom here
                flag_render_drag_select = false;

                let (window_width, window_height) = (window.inner_size().width, window.inner_size().height);
                // set offset
                let (start_x, start_y) = drag_select_start;
                let (end_x, end_y) = drag_select_end;
                let box_width = (start_x - end_x).abs();
                let box_height = (start_y - end_y).abs();
                let top_left_box = (start_x.min(end_x), start_y.min(end_y));
                offset_x += ((top_left_box.0 + box_width/2.0)  - window_width as f32 / 2.0) as f64 * zoom;
                offset_y += ((top_left_box.1 + box_height/2.0)  - window_height as f32 / 2.0) as f64 * zoom;
                
                // set zoom
                let box_area = box_width * box_height;
                let screen_area = window_width * window_height;
                let zoom_coeff = 10.0;
                // how many times smaller is the box than the screen
                zoom *= (box_area as f64 / screen_area as f64) * zoom_coeff;

            }
            /* else if input.mouse_pressed(0) { 
                if let Some((x,y)) = input.mouse() {
                    offset_x = (x - fractal_container.width as f32 / 2.0) * zoom + offset_x;
                    offset_y = (y - fractal_container.height as f32 / 2.0) * zoom + offset_y;
                }
            }  */

            else if input.key_pressed(winit::event::VirtualKeyCode::W) || input.key_pressed(winit::event::VirtualKeyCode::Right) {
                offset_y -= 0.5;
            }
            else if input.key_pressed(winit::event::VirtualKeyCode::S) || input.key_pressed(winit::event::VirtualKeyCode::Down) {
                offset_y += 0.5;
            }
            else if input.key_pressed(winit::event::VirtualKeyCode::A) || input.key_pressed(winit::event::VirtualKeyCode::Left) {
                offset_x -= 0.5;
            }
            else if input.key_pressed(winit::event::VirtualKeyCode::D) || input.key_pressed(winit::event::VirtualKeyCode::Right) {
                offset_x += 0.5;
            }
    
            // Resize the window
            else if let Some(size) = input.window_resized() {
                
                if let Err(_) = pixels.resize_surface(size.width, size.height) {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                if let Err(_) = pixels.resize_buffer(size.width, size.height) {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                // resize the frame buffer
                freeze_frame = vec![0; (4 * size.width * size.height) as usize]; 
            }
            else {
                // If no event happened, we wait for the next event...
                // something is causing an event to happen so this is needed to not unecessarily redraw
                return;
            }
            window.request_redraw();
        }
    });
}


/// Draw a box around the selected area.
/// Start and end are the top left and bottom right corners of the box
fn draw_select_box(pixels: &mut [u8], (x1,y1): (f32, f32), (x2,y2): (f32, f32), screen_width: u32) {
    let width = (x2-x1).abs() as usize;
    let screen_width = screen_width as usize;
    let (x1,y1,x2,y2) = (x1 as usize, y1 as usize, x2 as usize, y2 as usize);
    
    // (0,0) is in top left of window
    let top_left_pixel = (y1.min(y2) * screen_width + x1.min(x2)) * 4; // each pixel has r,g,b,a channels
    let bottom_left_pixel = (y1.max(y2) * screen_width + x1.min(x2)) * 4;

    // draw top and bottom borders
    let border_width = 3;
    let border = vec![255; (width+border_width)*4]; // +4 otherwise the corner pixels are not drawn
    for i in 0..border_width {
        let y_offset = i * screen_width * 4;
        pixels[top_left_pixel+y_offset .. top_left_pixel+y_offset+(width+border_width)*4].copy_from_slice(&border);
        pixels[bottom_left_pixel+y_offset .. bottom_left_pixel+y_offset+(width+border_width)*4].copy_from_slice(&border);
    }
    
    // draw left and right borders
    let border = vec![255; border_width*4];
    for y in y1.min(y2)..y1.max(y2) {
        let pixel = (y*screen_width + x1.min(x2)) * 4;
        pixels[pixel .. pixel+(border_width*4)].copy_from_slice(&border);
        pixels[pixel+(width*4) .. pixel+(width*4)+(border_width*4)].copy_from_slice(&border);
    }
}