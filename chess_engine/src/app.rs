//use crate::{board::Board, piece::Piece};
//use eframe::{App, CreationContext};
//use egui::{CentralPanel, Context, Image, Pos2, Rect, TextureHandle, TextureOptions, Vec2};
//use std::env;
//
//fn load_image_from_path(path: &std::path::Path) -> Result<egui::ColorImage, image::ImageError> {
//    let image = image::io::Reader::open(path)?.decode()?;
//    let size = [image.width() as _, image.height() as _];
//    let image_buffer = image.to_rgba8();
//    let pixels = image_buffer.as_flat_samples();
//    Ok(egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
//}
//
//pub struct ChessApp {
//    board_texture: TextureHandle,
//    piece_textures: [TextureHandle; 12],
//    board: Board,
//}
//
//impl ChessApp {
//    pub fn new(creation_context: &CreationContext) -> Self {
//        let path = env::current_dir().unwrap().join("assets");
//        let options = TextureOptions::NEAREST;
//        let load_texture_from_name = |name: &str| -> TextureHandle {
//            let image = load_image_from_path(path.join(name).as_path()).unwrap();
//            creation_context.egui_ctx.load_texture(name, image, options)
//        };
//
//        let board_texture = load_texture_from_name("board.png");
//        let piece_textures: [TextureHandle; 12] = [
//            load_texture_from_name("wp.png"),
//            load_texture_from_name("wn.png"),
//            load_texture_from_name("wb.png"),
//            load_texture_from_name("wr.png"),
//            load_texture_from_name("wq.png"),
//            load_texture_from_name("wk.png"),
//            load_texture_from_name("bp.png"),
//            load_texture_from_name("bn.png"),
//            load_texture_from_name("bb.png"),
//            load_texture_from_name("br.png"),
//            load_texture_from_name("bq.png"),
//            load_texture_from_name("bk.png"),
//        ];
//
//        let board = Board::start_pos();
//
//        Self { board_texture, piece_textures, board }
//    }
//}
//
//impl App for ChessApp {
//    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
//        CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
//            ui.image(self.board_texture.id(), self.board_texture.size_vec2());
//            for file in 0..8 {
//                for rank in 0..8 {
//                    let square = rank * 8 + file;
//                    if let Some(piece) = self.board.squares[square] {
//                        let piece_texture = &self.piece_textures[piece];
//                        let square_length = self.board_texture.size_vec2().x / 8.0;
//                        let image = Image::new(piece_texture.id(), piece_texture.size_vec2());
//                        ui.put(
//                            Rect {
//                                min: Pos2::new(square_length * file as f32, square_length * rank as f32),
//                                max: Pos2::new(square_length * (file as f32 + 1.0), square_length * (rank as f32 + 1.0)),
//                            },
//                            image,
//                        );
//                    }
//                }
//            }
//        });
//        ctx.input(|input| {
//            if input.pointer.primary_pressed() {
//                let square_length = self.board_texture.size_vec2().x / 8.0;
//            }
//        })
//    }
//}
