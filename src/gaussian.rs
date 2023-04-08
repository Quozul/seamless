use image::{GenericImageView, ImageBuffer, Rgba};
use std::f32::consts::PI;
use std::path::PathBuf;

pub fn gaussian(input: String, size: u32, sigma: f32, output_path: PathBuf) {
    let img = image::open(input).unwrap();
    let mut output = ImageBuffer::new(img.width(), img.height());

    let kernel = gaussian_kernel(size, sigma);

    let half = size as i32 / 2;

    for y in half..img.height() as i32 - half {
        for x in half..img.width() as i32 - half {
            let mut acc = [0.0; 4];
            let mut weight_acc = 0.0;

            for j in -half..half {
                for i in -half..half {
                    let kernel_y = j + half;
                    let kernel_x = i + half;
                    let pixel = img.get_pixel((x + i) as u32, (y + j) as u32);
                    let kernel_weight = kernel[kernel_y as usize][kernel_x as usize];

                    for c in 0..4 {
                        acc[c] += pixel[c] as f32 * kernel_weight;
                    }

                    weight_acc += kernel_weight;
                }
            }

            for c in 0..4 {
                acc[c] /= weight_acc;
            }

            output.put_pixel(
                x as u32,
                y as u32,
                Rgba::from(acc.map(|color| color.round() as u8)),
            );
        }
    }

    output.save(output_path).unwrap();
}

// TODO: Optimize
pub(crate) fn gaussian_kernel_a(x: i32, y: i32, sigma: f32) -> f32 {
    let exponent = -(x * x + y * y) as f32 / (2.0 * sigma * sigma);
    let denominator = 2.0 * PI * sigma * sigma;
    exponent.exp() / denominator
}

fn gaussian_kernel(size: u32, sigma: f32) -> Vec<Vec<f32>> {
    let mut kernel = vec![vec![0.0; size as usize]; size as usize];
    let center = size as i32 / 2;

    for y in 0..size {
        for x in 0..size {
            let x_distance = x as i32 - center;
            let y_distance = y as i32 - center;
            let value = gaussian_kernel_a(x_distance, y_distance, sigma);

            kernel[y as usize][x as usize] = value;
        }
    }

    kernel
}

#[cfg(test)]
mod tests {
    use crate::gaussian::gaussian_kernel_a;

    #[test]
    fn test_gaussian_kernel() {
        // Small hack due to floating point precision
        let test = gaussian_kernel_a(0, 0, 1.5) * 1000.0;
        assert_eq!(test.round(), 71.0);

        let test = gaussian_kernel_a(-1, -1, 1.5) * 1000.0;
        assert_eq!(test.round(), 45.0);

        let test = gaussian_kernel_a(0, -1, 1.5) * 1000.0;
        assert_eq!(test.round(), 57.0);
    }
}
