use gif::{DisposalMethod, Encoder, Frame, Repeat};
use image::{imageops::FilterType, ImageBuffer};
use std::io::Cursor;

use crate::error::ApiError;

use super::{color::Color, piece::Piece, state::GameState};

pub fn render(
    state: &GameState,
    color: Color,
) -> image::ImageResult<ImageBuffer<image::Rgba<u8>, Vec<u8>>> {
    let mut board = image::open("src/assets/board.png")?.to_rgba8();

    let chess_board = if color == Color::WHITE {
        state.chess_board.clone()
    } else {
        state.chess_board.rotate()
    };

    for index in (0..64).rev() {
        let (piece, color) = chess_board.piece_and_color_at_cell(index).unwrap();
        if piece == Piece::NONE || color == Color::NONE {
            continue;
        }
        let (x, y) = image_coordinates_from_index(index);

        let path = format!("src/assets/{}", piece.get_image_name(color));
        let piece_image = image::open(path)?.to_rgba8();
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

pub fn render_history_gif(game_state: &GameState, color: Color) -> Result<Vec<u8>, ApiError> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    {
        let mut encoder = Encoder::new(&mut cursor, 568, 568, &[])?;
        encoder.set_repeat(Repeat::Infinite)?;

        let mut state = GameState::new()?;
        let initial_image = render(&state, color)?;
        let mut initial_frame = Frame::from_rgba_speed(568, 568, &mut initial_image.into_raw(), 10);
        initial_frame.dispose = DisposalMethod::Background;
        initial_frame.delay = 100;
        encoder.write_frame(&initial_frame)?;

        for (i, (from, to)) in game_state.move_log.iter().enumerate() {
            state.make_move(*from, *to)?;
            let frame_image = render(&state, color)?;
            let mut frame = Frame::from_rgba_speed(568, 568, &mut frame_image.into_raw(), 10);
            frame.dispose = DisposalMethod::Background;

            frame.delay = if i < game_state.move_log.len() - 1 {
                100
            } else {
                500
            };
            encoder.write_frame(&frame)?;
        }
    }

    Ok(buffer)
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
