use image::{imageops::FilterType, ImageBuffer};

use super::{color::Color, piece::Piece, state::GameState};

pub fn render(state: GameState) -> image::ImageResult<ImageBuffer<image::Rgba<u16>, Vec<u16>>> {
    let mut board = image::open("src/assets/board.png")?.to_rgba16();

    for index in (0..64).rev() {
        let (piece, color) = state.chess_board.piece_and_color_at_cell(index).unwrap();
        if piece == Piece::NONE || color == Color::NONE {
            continue;
        }
        let (x, y) = image_coordinates_from_index(index);

        let path = format!("src/assets/{}", piece.get_image_name(color));
        let piece_image = image::open(path)?.to_rgba16();
        image::imageops::overlay(&mut board, &piece_image, x, y);
    }

    let upscale_factor = 4;
    let new_dimensions = (
        board.width() * upscale_factor,
        board.height() * upscale_factor,
    );
    let upscaled_image = image::imageops::resize(
        &board,
        new_dimensions.0,
        new_dimensions.1,
        FilterType::Nearest,
    );

    Ok(upscaled_image)
}

// oben links => 7, -2
// 16 nach rechts
// 12 nach unten
pub fn image_coordinates_from_index(index: u8) -> (i64, i64) {
    let current_row = (index / 8) as i64;
    let current_col = (index % 8) as i64;

    let x = 7 + current_col * 16;
    let y = -2 + (7 - current_row) * 12;

    (x, y)
}
