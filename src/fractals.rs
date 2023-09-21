/* 
Helpful resource for fractals/mandlebrot: https://complex-analysis.com/content/mandelbrot_set.html
*/

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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Fractals {
    Mandelbrot { max_iterations: u32, escape_radius: f64 },
    Julia { max_iterations: u32, escape_radius: f64, c: (f64, f64) },
}


impl Fractals {
    pub fn draw(self, pixels: &mut [u8], width: i32, height: i32, zoom: f64, offset_x: f64, offset_y: f64) {
        match self {
            Fractals::Mandelbrot {max_iterations, escape_radius} => generate_mandelbrot(pixels, width, height, zoom, offset_x, offset_y, escape_radius, max_iterations),
            Fractals::Julia {max_iterations, c, escape_radius} =>  generate_julia(pixels, width, height, zoom, offset_x, offset_y, escape_radius, c, max_iterations)
        }
    }
}

fn generate_julia(pixels: &mut [u8], width: i32, height: i32, zoom: f64, offset_x: f64, offset_y: f64, escape_radius: f64, (cx, cy): (f64, f64), max_iterations: u32) {
    assert!(escape_radius > 0.0);
    //assert!(escape_radius*escape_radius - escape_radius >= f64::sqrt(cx*cx + cy*cy));
    let r = escape_radius * escape_radius;

    for y_pixel in 0..height {
        for x_pixel in 0..width {
            let mut real = (x_pixel - width / 2) as f64 * zoom + offset_x as f64;
            let mut imaginary = (y_pixel - height / 2) as f64 * zoom + offset_y as f64;

            let mut iteration = 0;
            while real * real + imaginary * imaginary < r && iteration < max_iterations {
                let xtemp = real * real - imaginary * imaginary + cx;
                imaginary = 2.0 * real * imaginary + cy;
                real = xtemp;
                iteration = iteration + 1;
            }
            let iteration = iteration as f64;
             // Used to avoid floating point issues with points inside the set.
            /* if iteration < max_iteration as f64 {
                // sqrt of inner term removed using log simplification rules.
                let log_zn = (x*x + y*y).log(10.0) / 2.0;
                let nu = (log_zn / (2.0_f64).log(10.0)).log(10.0) / (2.0_f64).log(10.0);
                // Rearranging the potential function.
                // Dividing log_zn by log(2) instead of log(N = 1<<8)
                // because we want the entire palette to range from the
                // center to radius 2, NOT our bailout radius.
                iteration = iteration + 1.0 - nu;
            } */

            let r = ((0.0730 * iteration).sin() * 255.0) as u8; // casting to u8 will clamp at 255
            let g = ((0.0460 * iteration).sin() * 255.0) as u8;
            let b = ((0.0900 * iteration).sin() * 255.0) as u8;
            let r2 = ((0.0730 * (iteration + 1.0)).sin() * 255.0) as u8;
            let g2 = ((0.0460 * (iteration + 1.0)).sin() * 255.0) as u8;
            let b2 = ((0.0900 * (iteration + 1.0)).sin() * 255.0) as u8;
            let c = Colour::lerp(Colour { red: r, green: g,  blue: b }, Colour { red: r2, green: g2,  blue: b2 }, iteration%1.0);
            let a = 255; //alpha bit
            let pixel_idx = (y_pixel*width + x_pixel) as usize * 4;
            pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[c.red, c.green, c.blue, a]);
        }
        
    }
}


fn generate_mandelbrot(pixels: &mut [u8], width: i32, height: i32, zoom: f64, offset_x: f64, offset_y: f64, escape_radius: f64, max_iterations: u32) {
    let r = escape_radius * escape_radius;
    for y_pixel in 0..height {
        for x_pixel in 0..width {

            // (y_pixel as f64 - height as f64 / 2.0) calculates the vertical distance of the current pixel from the center of the screen.
            // then zoom and offset_y are applied to scale and shift the image.
            let imaginary = (y_pixel - height / 2) as f64 * zoom + offset_y as f64;
            let real = (x_pixel - width / 2) as f64 * zoom + offset_x as f64;

            let mut x = 0.0;
            let mut y = 0.0;
            let mut iteration = 0;
            let mut x2 = 0.0;
            let mut y2 = 0.0;
            while x2 + y2 <= r && iteration < max_iterations {
                y = 2.0 * x * y + imaginary;
                x = x2 - y2 + real;
                x2 = x * x;
                y2 = y * y;
                iteration = iteration + 1;
            }
            let mut iteration = iteration as f64;
             // Used to avoid floating point issues with points inside the set.
            if iteration < max_iterations as f64 {
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
                let a = 255; //alpha bit
                let pixel_idx = (y_pixel*width + x_pixel) as usize * 4;
                pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[c.red, c.green, c.blue, a]);
                //pixels[pixel_idx..pixel_idx+4].copy_from_slice(&[r, g, b, a]);

        }
    }
}