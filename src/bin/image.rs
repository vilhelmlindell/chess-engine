use image::{RgbImage, Rgb};
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};
use std::fs::File;
use std::io::BufReader;

fn main() {
    // Create a new RgbImage (width: 400, height: 200)
    let mut img = RgbImage::new(400, 200);

    // Fill the image with a background color (white)
    for pixel in img.pixels_mut() {
        *pixel = Rgb([255, 255, 255]); // White color
    }

    // Load the font (you'll need to provide a .ttf file path here)
    let font_data = include_bytes!("assets/OpenSans-Regular.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).expect("Error loading font");

    // Define the scale for the text (size of the font)
    let scale = Scale { x: 50.0, y: 50.0 };

    // Define the position of the text
    let x = 50;
    let y = 80;

    // Draw the text onto the image
    draw_text_mut(
        &mut img,                   // Image to draw on
        Rgb([0, 0, 0]),             // Text color (black)
        x, y,                       // Position to start drawing
        scale,                      // Font size
        &font,                      // Font
        "Hello, Rust!"              // The text
    );

    // Save the image as a PNG file
    img.save("output.png").expect("Failed to save image");
}
