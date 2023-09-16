//mod fractals;
use crate::fractals::*;
//use fractals::{Fractals, Colour};

/* #[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let norm_x = normalize(x_pixel as f64, 0.0, width as f64, MANDELBROT_SCALE_X.0, MANDELBROT_SCALE_X.1);
    }
} */
const MANDELBROT_SCALE_X: (f64, f64) = (-2.0, 0.47);
const MANDELBROT_SCALE_Y: (f64, f64) = (-1.12, 1.12);

fn main() {
    let x = 10.0;
    let norm_x = fractals::normalize(x as f64, 0.0, 400 as f64, MANDELBROT_SCALE_X.0, MANDELBROT_SCALE_X.1);
    let y = 1000.0;
    let norm_y = fractals::normalize(y as f64, 0.0, 300 as f64, MANDELBROT_SCALE_X.0, MANDELBROT_SCALE_X.1);
    println!("norm_x: {}", norm_x);
    println!("norm_y: {}", norm_y);
}