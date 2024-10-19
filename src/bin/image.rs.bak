use image::{Rgba, RgbaImage};
use rusttype::{point, Font, Scale};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

fn main() {
    // Load the font (you need to provide a .ttf file path)
    let font_path = "/home/vilhelm/dev/rust/chess-engine/assets/OpenSans-Regular.ttf"; // Path to your TTF font file
    let font = Vec::from(std::fs::read(font_path).expect("Error reading font file"));
    let font = Font::try_from_vec(font).expect("Error loading font");

    // Load or create a base image
    let mut image: RgbaImage = RgbaImage::new(400, 200); // Create a blank image with width 400 and height 200

    // Define the text properties
    let scale = Scale { x: 40.0, y: 40.0 }; // Font size
    let color = Rgba([255, 0, 0, 255]); // Red color with no transparency

    // Text to render and position
    let text = "Hello, Rust!";
    let position = point(20.0, 100.0); // The starting point (x, y) of the text

    // Render the text onto the image
    render_text(&mut image, &font, scale, position, color, text);

    // Save the resulting image
    let save_path = Path::new("output_text_image.png");
    image.save(save_path).expect("Failed to save image");

    println!("Image with text saved successfully to {:?}", save_path);
}

fn render_text(image: &mut RgbaImage, font: &Font, scale: Scale, position: rusttype::Point<f32>, color: Rgba<u8>, text: &str) {
    use rusttype::PositionedGlyph;

    // Layout the glyphs for the text
    let glyphs: Vec<PositionedGlyph> = font.layout(text, scale, position).collect();

    // Render each glyph
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            // Draw each pixel of the glyph on the image
            glyph.draw(|x, y, v| {
                let x = x as i32 + bounding_box.min.x;
                let y = y as i32 + bounding_box.min.y;

                if x >= 0 && x < image.width() as i32 && y >= 0 && y < image.height() as i32 {
                    let pixel = image.get_pixel_mut(x as u32, y as u32);
                    let alpha = (v * 255.0) as u8;
                    let blended_color = blend_colors(*pixel, color, alpha);
                    *pixel = blended_color;
                }
            });
        }
    }
}

fn blend_colors(background: Rgba<u8>, foreground: Rgba<u8>, alpha: u8) -> Rgba<u8> {
    let inv_alpha = 255 - alpha;
    Rgba([
        ((foreground[0] as u16 * alpha as u16 + background[0] as u16 * inv_alpha as u16) / 255) as u8,
        ((foreground[1] as u16 * alpha as u16 + background[1] as u16 * inv_alpha as u16) / 255) as u8,
        ((foreground[2] as u16 * alpha as u16 + background[2] as u16 * inv_alpha as u16) / 255) as u8,
        255,
    ])
}
