// TODO: Implement
pub fn gaussian(input: String) {
    let img = image::open(input).unwrap();
    println!("{} × {}", img.width(), img.height());
}
