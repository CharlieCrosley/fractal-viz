//mod custom_window;
mod gui;
mod fractals;

use std::time::Instant;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use gui::Framework;
use fractals::Fractals;

//mod test;

const MIN_WIDTH: i32 = 400;
const MIN_HEIGHT: i32 = 300;
const INIT_ZOOM: f64 = 0.003;


fn main() {
    //test::foo();

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let monitor_size = event_loop.primary_monitor().unwrap().size(); 
    
    let window = { 
        let size = LogicalSize::new(MIN_WIDTH as f64, MIN_HEIGHT as f64); // minimum window size
        let scaled_size = LogicalSize::new(monitor_size.width as f64, monitor_size.height as f64); // initial window size
        WindowBuilder::new()
            .with_title("Fractals")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size)
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

    let scale_factor = window.scale_factor() as f32 * 1.2; // increase the scale factor to make the ui/text bigger
    
    let window_open_size: (f32, f32) = (300.0, 200.0);
    let window_closed_size: (f32, f32) = (85.0, 35.0);
    let window_position: (f32, f32) = (10.0, 10.0);
    let mut framework = Framework::new(
        &event_loop,
        window_size.width,
        window_size.height,
        scale_factor,
        &pixels,
        window_position,
        window_open_size,
        window_closed_size
    );

    // Set the default fractal to render the Mandelbrot set
    let mut fractal = Fractals::Julia {max_iterations: 100, escape_radius: 2.0, c: (-0.8, 0.156)};
    // Set the default zoom to zero, changes when scrolling mouse wheel
    let mut zoom: f64 = INIT_ZOOM;
    // When zooming we want to zoom in on the mouse position, so we need to keep track of the mouse position
    let mut offset_x: f64 = 0.0;
    let mut offset_y: f64 = 0.0;
    let zoom_amount = 5.0; // how much to zoom in/out when scrolling the mouse wheel
    let mut drag_select_start = (0.0,0.0);
    let mut drag_select_end = (0.0,0.0);
    let mut flag_render_drag_select = false;
    let mut flag_generate_new_fractal = true;
    let mut flag_fractal_change = true;

    // store the frame when the user starts dragging the mouse to select an area to zoom in on
    // used to set the pixel buffer to the freeze frame so that the previous frames select box is removed
    let mut freeze_frame: Vec<u8> = pixels.frame().to_vec(); 

    event_loop.run(move |event, _, control_flow| {
         // For everything else, for let winit_input_helper collect events to build its state.
        // It returns `true` when it is time to update our game state and request a redraw.
        if input.update(&event) {
            if mouse_in_ui_window(&input, &mut framework) || framework.get_gui().get_window_open() {
                framework.get_gui().set_last_mouse_move(Instant::now());
                framework.get_gui().set_mouse_in_window(true);
            }
            else {
                framework.get_gui().set_mouse_in_window(false);
            }
            
            // If the user scrolls the mouse wheel, zoom in/out
            let scroll = input.scroll_diff();
            if scroll != 0.0 {
                let zoom_factor = 1.0 + (0.1 * zoom_amount * -scroll.signum());
                zoom *= zoom_factor as f64;
                flag_generate_new_fractal = true;
            }
            // Left click
            else if input.mouse_pressed(0) {
                if !mouse_in_ui_window(&input, &mut framework) {
                    // if the mouse is inside the ui window, don't do anything
                    //return;
                    if let Some((x,y)) = input.mouse() {
                        freeze_frame.copy_from_slice(pixels.frame());
                        drag_select_start = (x,y);
                        drag_select_end = (x,y); // reset the end point to the start point
                        flag_render_drag_select = true;
                    }
                }
                else {
                    freeze_frame.copy_from_slice(pixels.frame());
                }
                
            }
            // Hold left click
            else if input.mouse_held(0) {
                if let Some((x,y)) = input.mouse() {
                    let (width, height) = window.inner_size().into();
                    // clamp the mouse position to the window size
                    drag_select_end = (x.clamp(0.0, width), y.clamp(0.0, height-1.0));
                }
            }
            // Release left click
            else if input.mouse_released(0) {
                // zoom here
                flag_render_drag_select = false;

                if !mouse_in_ui_window(&input, &mut framework) {
                    // if the mouse is inside the ui window, don't do anything
                    //return;
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
                    if box_area >= 100.0 { // if the box is too small, don't zoom
                        let screen_area = window_width * window_height;
                        let zoom_coeff = 10.0;
                        // how many times smaller is the box than the screen
                        zoom *= (box_area as f64 / screen_area as f64) * zoom_coeff;
                    }
                    flag_generate_new_fractal = true;
                }
                else {
                    freeze_frame.copy_from_slice(pixels.frame());
                }
            }
            else if input.key_pressed(winit::event::VirtualKeyCode::W) || input.key_pressed(winit::event::VirtualKeyCode::Right) {
                offset_y -= 0.5 * (zoom / INIT_ZOOM); // adjust the move distance based on the zoom level so that the movements dont become massive
                flag_generate_new_fractal = true;
            }
            else if input.key_pressed(winit::event::VirtualKeyCode::S) || input.key_pressed(winit::event::VirtualKeyCode::Down) {
                offset_y += 0.5 * (zoom / INIT_ZOOM);
                flag_generate_new_fractal = true;
            }
            else if input.key_pressed(winit::event::VirtualKeyCode::A) || input.key_pressed(winit::event::VirtualKeyCode::Left) {
                offset_x -= 0.5 * (zoom / INIT_ZOOM);
                flag_generate_new_fractal = true;
            }
            else if input.key_pressed(winit::event::VirtualKeyCode::D) || input.key_pressed(winit::event::VirtualKeyCode::Right) {
                offset_x += 0.5 * (zoom / INIT_ZOOM);
                flag_generate_new_fractal = true;
            }
    
            // Update the scale factor
            else if let Some(scale_factor) = input.scale_factor() {
                framework.scale_factor(scale_factor);
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
                framework.resize(size.width, size.height);
            }
            window.request_redraw();
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::WindowEvent { event, .. } => {
                framework.handle_event(&event);
            }

            Event::RedrawRequested(_) => {
                let (width, height) = (window.inner_size().width, window.inner_size().height);

                if flag_render_drag_select {
                    // reset the pixel buffer to the freeze frame so that the previous frames select box is removed
                    pixels.frame_mut().copy_from_slice(&freeze_frame);
                    // don't render the select box if the mouse hasn't moved enough
                    if (drag_select_start.0 - drag_select_end.0).abs() > 10.0 && (drag_select_start.1 - drag_select_end.1).abs() > 10.0 {
                        draw_zoom_box(pixels.frame_mut(), drag_select_start, drag_select_end, width as u32);
                    }
                } 
                else if flag_generate_new_fractal || flag_fractal_change {
                    if flag_fractal_change {
                        zoom = INIT_ZOOM;
                        offset_x = 0.0;
                        offset_y = 0.0;
                        flag_fractal_change = false;
                    }

                    //use std::time::Instant;
                    let now = Instant::now();
                    // Generate and render the fractal here
                    fractal.draw(pixels.frame_mut(), width as i32, height as i32, zoom, offset_x, offset_y);

                    let elapsed = now.elapsed();
                    println!("Elapsed: {:.2?}", elapsed);
                    freeze_frame.copy_from_slice(pixels.frame());
                }

                // If these flags are false it means no new frame was generated so we don't need to render the ui
                // neccesary to make ui animations smooth since generating fractal takes too much time per frame
                if !flag_generate_new_fractal && !flag_render_drag_select {
                    pixels.frame_mut().copy_from_slice(&freeze_frame);
                }
                framework.prepare(&window, &mut fractal, &mut flag_fractal_change);
                // Render everything together
                let render_result = pixels.render_with(|encoder, render_target, context| {
                    // Render the fractal
                    context.scaling_renderer.render(encoder, render_target);
                    // Render egui
                    framework.render(encoder, render_target, context);
                    Ok(())
                });

                // Basic error handling
                if let Err(_) = render_result {
                    *control_flow = ControlFlow::Exit;
                }
                
                flag_generate_new_fractal = false;
            },
            _ => {}
        }
    });
}

fn mouse_in_ui_window(input: &WinitInputHelper, framework: &mut Framework) -> bool {
    let ui_window_size = framework.get_gui().get_window_size();
    let ui_window_position = framework.get_gui().get_window_position();
    
    if let Some((x,y)) = input.mouse() {
        
        if x > ui_window_position.0 && x < ui_window_position.0 + ui_window_size.0 && 
        y > ui_window_position.1 && y < ui_window_position.1 + ui_window_size.1 {
            // if the mouse is inside the ui window, don't do anything
            return true;
        }
    }
    return false
}

/// Draw a box around the selected area.
/// Start and end are the top left and bottom right corners of the box
fn draw_zoom_box(pixels: &mut [u8], (x1,y1): (f32, f32), (x2,y2): (f32, f32), screen_width: u32) {
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
        pixels[bottom_left_pixel-y_offset .. bottom_left_pixel-y_offset+(width+border_width)*4].copy_from_slice(&border);
    }
    
    // draw left and right borders
    let border = vec![255; border_width*4];
    for y in y1.min(y2)..y1.max(y2) {
        let pixel = (y*screen_width + x1.min(x2)) * 4;
        pixels[pixel .. pixel+(border_width*4)].copy_from_slice(&border);
        pixels[pixel+(width*4) .. pixel+(width*4)+(border_width*4)].copy_from_slice(&border);
    }
}