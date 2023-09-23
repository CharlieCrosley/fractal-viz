/* 
Helpful resource for fractals/mandlebrot: https://complex-analysis.com/content/mandelbrot_set.html
*/

use colorgrad::Gradient;
use num::{complex::{Complex64, ComplexFloat}, traits::Pow};
use rayon::prelude::*;

pub const COLOUR_GRADIENTS: [&str; 8] = ["Magma", "Rainbow", "Plasma", "Inferno", "Viridis", "Cividis", "Turbo", "Sinebow"];

#[derive(Clone,PartialEq, Debug)] 
pub enum Fractals {
    Mandelbrot { max_iterations: u32, escape_radius: f64, colour_gradient: String },
    Julia { max_iterations: u32, escape_radius: f64, c: (f64, f64), colour_gradient: String },
    Newton { max_iterations: u32, colour_gradient: String },
}

fn string_to_colour_gradient(s: &str) -> Gradient {
    if COLOUR_GRADIENTS.contains(&s) {
        match s {
            "Magma" => colorgrad::magma(),
            "Rainbow" => colorgrad::rainbow(),
            "Plasma" => colorgrad::plasma(),
            "Inferno" => colorgrad::inferno(),
            "Viridis" => colorgrad::viridis(),
            "Cividis" => colorgrad::cividis(),
            "Turbo" => colorgrad::turbo(),
            "Sinebow" => colorgrad::sinebow(),
            _ => colorgrad::sinebow(),
        }
    } else {
        colorgrad::sinebow() // default
    }
}

impl Fractals {
    pub fn draw(self, pixels: &mut [u8], width: i32, height: i32, zoom: f64, offset_x: f64, offset_y: f64) {
        match self {
            Fractals::Mandelbrot {max_iterations, escape_radius, colour_gradient} => 
                generate_mandelbrot(pixels, width, height, zoom, offset_x, offset_y, escape_radius, max_iterations, string_to_colour_gradient(&colour_gradient)),
            Fractals::Julia {max_iterations, c, escape_radius, colour_gradient} =>  
                generate_julia(pixels, width, height, zoom, offset_x, offset_y, escape_radius, c, max_iterations, string_to_colour_gradient(&colour_gradient)),
            Fractals::Newton {max_iterations, colour_gradient} => {
                generate_newton(pixels, width, height, zoom, offset_x, offset_y, max_iterations, string_to_colour_gradient(&colour_gradient))}
            
        }
    }
}

// TODO: Allow user to change function
#[inline]
fn newton_func(z: Complex64) -> Complex64 {
    z.pow(3.0) - 1.0 // try this z8 + 3z4 - 4
}
#[inline]
fn newton_func_deriv(z: Complex64) -> Complex64 {
    3.0 * z.pow(2.0)
}

fn generate_newton(pixels: &mut [u8], width: i32, height: i32, zoom: f64, offset_x: f64, offset_y: f64, max_iterations: u32, colour_gradient: Gradient) {
    let roots: [Complex64; 3] = [
        Complex64::new(1.0, 0.0), 
        Complex64::new(-0.5, 3.0.sqrt()/2.0), 
        Complex64::new(-0.5, -3.0.sqrt()/2.0)
    ];
    
    let tolerance = 0.000001;
    // Parallel loop that takes 4 values at a time (r,g,b,a) and processes them in parallel
    pixels.into_par_iter().chunks(4).enumerate().for_each(|(i, mut pixel)| {
        let y_pixel = i as i32 / width;
        let x_pixel = i as i32 % width;
        let real = (x_pixel - width / 2) as f64 * zoom + offset_x as f64;
        let imaginary = (y_pixel - height / 2) as f64 * zoom + offset_y as f64;

        let mut z = Complex64::new(real, imaginary);
        
        let mut iteration = 0;
        let mut found_root = false;
        while iteration < max_iterations && !found_root {
            z -= newton_func(z) / newton_func_deriv(z);
            
            for root in roots.iter() {
                let diff = z - root;
                if diff.re.abs() < tolerance && diff.im.abs() < tolerance {
                    found_root = true;
                    break;
                }
            }
            iteration += 1;
        }
        let iteration = iteration as f32 / max_iterations as f32;
        let [c1, c2, c3, c4] = colour_gradient.at(iteration.into()).to_rgba8();
        *pixel[0] = c1;
        *pixel[1] = c2;
        *pixel[2] = c3;
        *pixel[3] = c4;
    });
}

fn generate_julia(pixels: &mut [u8], width: i32, height: i32, zoom: f64, offset_x: f64, offset_y: f64, escape_radius: f64, (cx, cy): (f64, f64), max_iterations: u32, colour_gradient: Gradient) {
    assert!(escape_radius > 0.0);
    let r = escape_radius * escape_radius;
    pixels.into_par_iter().chunks(4).enumerate().for_each(|(i, mut pixel)| {
        let y_pixel = i as i32 / width;
        let x_pixel = i as i32 % width;
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
        let [c1, c2, c3, c4] = colour_gradient.at((iteration as f32 / max_iterations as f32).into()).to_rgba8();
        *pixel[0] = c1;
        *pixel[1] = c2;
        *pixel[2] = c3;
        *pixel[3] = c4;
    });
}


fn generate_mandelbrot(pixels: &mut [u8], width: i32, height: i32, zoom: f64, offset_x: f64, offset_y: f64, escape_radius: f64, max_iterations: u32, colour_gradient: Gradient) {
    assert!(escape_radius > 0.0);

    let r = escape_radius * escape_radius;
    pixels.into_par_iter().chunks(4).enumerate().for_each(|(i, mut pixel)| {
        let y_pixel = i as i32 / width;
        let x_pixel = i as i32 % width;
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
        let iteration = iteration as f64;
        let [c1, c2, c3, c4] = colour_gradient.at((iteration as f32 / max_iterations as f32).into()).to_rgba8();
        *pixel[0] = c1;
        *pixel[1] = c2;
        *pixel[2] = c3;
        *pixel[3] = c4;
    });
}