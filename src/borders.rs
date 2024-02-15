use image::GenericImageView;

fn average_color(img: &image::DynamicImage) -> [u8; 3] {
    let mut acc = [0; 4];
    let (width, _) = img.dimensions();

    for x in 0..width {
        let pixel = img.get_pixel(x, 0);
        for c in 0..4 {
            acc[c] += pixel[c] as u32;
        }
    }

    let pixel_count = width;
    [
        (acc[0] / pixel_count) as u8,
        (acc[1] / pixel_count) as u8,
        (acc[2] / pixel_count) as u8,
    ]
}

fn color_to_hex(color: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2])
}

pub(crate) fn borders(input: String) {
    let mut img = image::open(input).unwrap();
    let (width, height) = img.dimensions();
    let first_row = img.crop(0, 0, width, 1);
    let last_row = img.crop(0, height - 1, width, 1);

    let first_row_average_color = average_color(&first_row);
    let last_row_average_color = average_color(&last_row);

    println!(
        "First row average color: {}",
        color_to_hex(first_row_average_color)
    );

    println!(
        "Last row average color: {}",
        color_to_hex(last_row_average_color)
    );
}
