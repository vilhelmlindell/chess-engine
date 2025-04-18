use chess_engine::board::{bitboard::Bitboard, piece_move::Square};
use chess_engine::board::piece::Piece;
use chess_engine::board::Board;
use image::{imageops, DynamicImage, GenericImageView, RgbImage, Rgba, RgbaImage};
use imageproc::drawing::{draw_text_mut, text_size};
use std::sync::LazyLock;
use std::collections::HashMap;
use std::path::Path;
use ab_glyph::{Font, FontArc, PxScale, PxScaleFont, ScaleFont};

pub static PIECE_IMAGES: LazyLock<HashMap<Piece, RgbaImage>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(Piece::WhitePawn, load_image("assets/wp.png"));
    map.insert(Piece::WhiteKnight, load_image("assets/wn.png"));
    map.insert(Piece::WhiteBishop, load_image("assets/wb.png"));
    map.insert(Piece::WhiteRook, load_image("assets/wr.png"));
    map.insert(Piece::WhiteQueen, load_image("assets/wq.png"));
    map.insert(Piece::WhiteKing, load_image("assets/wk.png"));
    map.insert(Piece::BlackPawn, load_image("assets/bp.png"));
    map.insert(Piece::BlackKnight, load_image("assets/bn.png"));
    map.insert(Piece::BlackBishop, load_image("assets/bb.png"));
    map.insert(Piece::BlackRook, load_image("assets/br.png"));
    map.insert(Piece::BlackQueen, load_image("assets/bq.png"));
    map.insert(Piece::BlackKing, load_image("assets/bk.png"));
    map
});

pub static BOARD_IMAGE: LazyLock<RgbaImage> = LazyLock::new(|| load_image("assets/board.png"));
pub static FONT: LazyLock<PxScaleFont<FontArc>> = LazyLock::new(|| {
    let font_data = include_bytes!("../../assets/OpenSans-Regular.ttf");
    let font = FontArc::try_from_vec(font_data.to_vec()).expect("Failed to load font");
    let scale = PxScale::from(100.0);
    font.into_scaled(scale)
});

fn load_image<P: AsRef<Path>>(path: P) -> RgbaImage {
    image::open(path).expect("Failed to load image").to_rgba8()
}

fn lerp_color(a: Rgba<u8>, b: Rgba<u8>, t: f32) -> Rgba<u8> {
    Rgba([
        (a[0] as f32 * (1.0 - t) + b[0] as f32 * t) as u8,
        (a[1] as f32 * (1.0 - t) + b[1] as f32 * t) as u8,
        (a[2] as f32 * (1.0 - t) + b[2] as f32 * t) as u8,
        a[3], // Keep original alpha
    ])
}

fn draw_board(board: &Board) -> RgbaImage {
    let mut board_image = BOARD_IMAGE.clone();

    for (i, piece_option) in board.squares.iter().enumerate() {
        let rank = (i as u32) / 8;
        let file = (i as u32) % 8;
        let square_size = board_image.width() / 8;

        let x_center = file * square_size + square_size / 2;
        let y_center = rank * square_size + square_size / 2;

        // Draw the piece image
        if let Some(piece) = piece_option {
            let piece_image = PIECE_IMAGES[piece].clone();
            let x_center = x_center - piece_image.width() / 2;
            let y_center = y_center - piece_image.height() / 2;
            imageops::overlay(&mut board_image, &piece_image, x_center as i64, y_center as i64);
        }
    }
    board_image
}

fn get_bitboard_array(bitboard: Bitboard) -> [u64; 64] {
    let mut array = [0; 64];
    for i in 0..64 {
        if bitboard.bit(i) != 0 {
            array[i] = 1;
        }
    }
    array
}

fn draw_bitboard(image: &mut RgbaImage, bitboard: Bitboard) {
    for (i, _bit) in get_bitboard_array(bitboard).iter().enumerate() {
        draw_text_on_square(image, i, &_bit.to_string());
    }
}

fn draw_text_on_square(image: &mut RgbaImage, square: Square, text: &str) {
    let rank = (square as u32) / 8;
    let file = (square as u32) % 8;
    let square_size = image.width() / 8;

    let x_center = file * square_size + square_size / 2;
    let y_center = rank * square_size + square_size / 2;

    let text_size = text_size(FONT.scale, &FONT.font, &text);
    draw_text_mut(
        image,
        Rgba([255, 255, 255, 255]),
        x_center as i32 - text_size.0 as i32 / 2,
        y_center as i32 - text_size.1 as i32 / 2,
        FONT.scale.x,
        &FONT.font,
        &text,
    );
}

fn main() {
    let board = Board::start_pos();
    let mut image = draw_board(&board);

    let tint = Rgba([0, 0, 0, 255]); // Black color for tint
    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel = image.get_pixel(x, y);
            let tinted_pixel = lerp_color(*pixel, tint, 0.5);
            image.put_pixel(x, y, tinted_pixel);
        }
    }

    let bitboard = board.piece_squares[Piece::WhitePawn];

    draw_bitboard(&mut image, bitboard);
    image.save("chess_position.png").expect("Failed to save board");
}
