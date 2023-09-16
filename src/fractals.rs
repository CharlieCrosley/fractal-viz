
//const MANDELBROT_SCALE_X: (f64, f64) = (-2.0*1.5, 0.47*2.0);
//const MANDELBROT_SCALE_Y: (f64, f64) = (-1.12*2.0, 1.12*2.0);
/* const MANDELBROT_SCALE_X: (f64, f64) = (-2.0, 0.47);
const MANDELBROT_SCALE_Y: (f64, f64) = (-1.12, 1.12); */

#[derive(Debug, Clone, Copy)]
pub struct Colour {
    red: u8,
    green: u8,
    blue: u8,
}

impl Colour {
    /* pub fn new(red: u8, green: u8, blue: u8) -> Colour {
        Colour { red, green, blue }
    } */
    // Linear interpolation between two colors
    pub fn lerp(start: Colour, end: Colour, t: f64) -> Colour {
        let red = (start.red as f64 * (1.0 - t) + end.red as f64 * t) as u8;
        let green = (start.green as f64 * (1.0 - t) + end.green as f64 * t) as u8;
        let blue = (start.blue as f64 * (1.0 - t) + end.blue as f64 * t) as u8;
        Colour { red, green, blue }
    }
}

#[derive(Clone, Copy)]
pub enum Fractals {
    Mandelbrot,
    Julia,
}


pub fn generate_mandelbrot(pixels: &mut [u8], width: i32, height: i32, zoom: f64, offset_x: f64, offset_y: f64) {
    for y_pixel in 0..height {
        for x_pixel in 0..width {

            // (y_pixel as f64 - height as f64 / 2.0) calculates the vertical distance of the current pixel from the center of the screen.
            // then zoom and offset_y are applied to scale and shift the image.
            let imaginary = (y_pixel - height / 2) as f64 * zoom + offset_y as f64;
            let real = (x_pixel - width / 2) as f64 * zoom + offset_x as f64;

            // benchmark: 645ms -> 
            //let (x0, y0) = pixel_to_mandelbrot_scale(x_pixel + pos_x, y_pixel + pos_y, width, height);
            let mut x = 0.0;
            let mut y = 0.0;
            let mut iteration = 0;
            let max_iteration = 100;
            let mut x2 = 0.0;
            let mut y2 = 0.0;
            while x2 + y2 <= 4.0 && iteration < max_iteration {
                y = 2.0 * x * y + imaginary;
                x = x2 - y2 + real;
                x2 = x * x;
                y2 = y * y;
                iteration = iteration + 1;
            }
            let mut iteration = iteration as f64;
             // Used to avoid floating point issues with points inside the set.
            if iteration < max_iteration as f64 {
                // sqrt of inner term removed using log simplification rules.
                let log_zn = (x*x + y*y).log(10.0) / 2.0;
                let nu = (log_zn / (2.0_f64).log(10.0)).log(10.0) / (2.0_f64).log(10.0);
                // Rearranging the potential function.
                // Dividing log_zn by log(2) instead of log(N = 1<<8)
                // because we want the entire palette to range from the
                // center to radius 2, NOT our bailout radius.
                iteration = iteration + 1.0 - nu;

                
            }
            //TODO: adjust mandlebrot scale to zoom in/out
                /* 
                // LIME
                let r = ((0.016 * iteration as u32 as f64 + 0.0).sin() * 255.0) as u8;
                let g = ((0.020 * iteration as u32 as f64 + 0.0).sin() * 255.0) as u8;
                let b = ((0.025 * iteration as u32 as f64 + 0.0).sin() * 255.0) as u8; 
                */
                /*
                BLUE/BLACK
                let r = ((0.016 * iteration as u32 as f64 + 1.5).sin() * 255.0) as u8;
                let g = ((0.030 * iteration as u32 as f64 + 2.0).sin() * 255.0) as u8;
                let b = ((0.035 * iteration as u32 as f64 + 1.0).sin() * 255.0) as u8; */
                /* 
                RED/PINK
                let r = ((0.016 * iteration as u32 as f64 + 1.0).sin() * 255.0) as u8;
                let g = ((0.030 * iteration as u32 as f64 + 2.0).sin() * 255.0) as u8;
                let b = ((0.035 * iteration as u32 as f64 + 1.0).sin() * 255.0) as u8; */

                let r = ((0.0730 * iteration).sin() * 255.0) as u8; // casting to u8 will clamp at 255
                let g = ((0.0460 * iteration).sin() * 255.0) as u8;
                let b = ((0.0900 * iteration).sin() * 255.0) as u8;
                let r2 = ((0.0730 * (iteration + 1.0)).sin() * 255.0) as u8;
                let g2 = ((0.0460 * (iteration + 1.0)).sin() * 255.0) as u8;
                let b2 = ((0.0900 * (iteration + 1.0)).sin() * 255.0) as u8;
                let c = Colour::lerp(Colour { red: r, green: g,  blue: b }, Colour { red: r2, green: g2,  blue: b2 }, iteration%1.0);
                /* let r = ((0.016 * iteration + 4.0).sin() * 230.0) as u8 + 25;
                let g = ((0.013 * iteration + 2.0).sin() * 230.0) as u8 + 25;
                let b = ((0.01 * iteration + 1.0).sin() * 230.0) as u8 + 25; */
                //println!("{} {} {}", r, g, b);
                //println!("{} {} {}", (0.016 * iteration + 4.0).sin(), (0.013 * iteration + 2.0).sin(), (0.01 * iteration + 1.0).sin());
                let a = 255; //alpha bit
                let pixel_idx = (y_pixel*width + x_pixel) as usize * 4;
                pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[c.red, c.green, c.blue, a]);
                //pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[r, g, b, a]);
            

            /* let pixel_idx = (y_pixel*width + x_pixel) as usize * 4;
            let (r,g,b) = integer_to_rgb(iteration, max_iteration);
            pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[r, g, b, 255]); */
        }
    }
}

/* fn integer_to_rgb(value: u32, max_value: u32) -> (u8, u8, u8) {
    // Ensure value is within the valid range
    assert!(value <= max_value);

    if value == max_value {
        // Return black for points that reach the maximum number of iterations
        return (0, 0, 0);
    }

    // Map the number of iterations to a gradient of dark blue
    let r = 0;
    let g = (value % 64 * 2) as u8; // Dark blue-green component
    //let g = 0; // Dark blue-green component
    let b = (value % 200 * 10) as u8; // Dark blue component
    (r, g, b)
} */


/* /// Normalize value x from range [x_min, x_max] to range [lower_bound, upper_bound]
pub fn normalize(x: i32, x_min: i32, x_max: i32, lower_bound: f64, upper_bound: f64) -> f64 {
    (upper_bound - lower_bound) * ((x - x_min) / (x_max - x_min)) as f64 + lower_bound
} */

//fn pixel_to_mandelbrot_scale(x: u32, y: u32, width: u32, height: u32, (mandelbrot_x_min, mandelbrot_x_max): (f64, f64), (mandelbrot_y_min, mandelbrot_y_max): (f64, f64)) -> (f64, f64) {
/* fn pixel_to_mandelbrot_scale(x: u32, y: u32, width: u32, height: u32) -> (f64, f64) {
    /* let x_range = mandelbrot_x_max - mandelbrot_x_min;
    let y_range = mandelbrot_y_max - mandelbrot_y_min;
    
    let x_scaled = (x as f64 / width as f64) * x_range + mandelbrot_x_min;
    let y_scaled = (y as f64 / height as f64) * y_range + mandelbrot_y_min; */
    let x_range = MANDELBROT_SCALE_X.1 - MANDELBROT_SCALE_X.0;
    let y_range = MANDELBROT_SCALE_Y.1 - MANDELBROT_SCALE_Y.0;
    
    let x_scaled = (x as f64 / width as f64) * x_range + MANDELBROT_SCALE_X.0;
    let y_scaled = (y as f64 / height as f64) * y_range + MANDELBROT_SCALE_Y.0;
    
    (x_scaled, y_scaled)
}
 */