use gif::{DisposalMethod, Encoder, Frame, Repeat};
use image::{DynamicImage, ImageBuffer, Rgba};
use std::io::Cursor;

use crate::error::ApiError;

use super::{color::Color, piece::Piece, state::GameState};

pub fn render(state: &GameState, color: Color) -> Result<Vec<u8>, ApiError> {
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
        image::imageops::FilterType::Nearest,
    );

    Ok(upscaled_image.into_raw())
}

pub fn render_board_png(game_state: &GameState, color: Color) -> Result<Vec<u8>, ApiError> {
    let raw_data = render(game_state, color)?;
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(568, 568, raw_data).ok_or(
        ApiError::ServerError("Failed to create image buffer.".to_string()),
    )?;
    let dynamic_image = DynamicImage::ImageRgba8(buffer);
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    dynamic_image.write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(png_bytes)
}

pub fn render_history_gif(game_state: &GameState, color: Color) -> Result<Vec<u8>, ApiError> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    {
        let mut encoder = Encoder::new(&mut cursor, 568, 568, &[])?;
        encoder.set_repeat(Repeat::Infinite)?;

        let mut state = GameState::new()?;
        let mut initial_image = render(&state, color)?;
        let mut initial_frame = Frame::from_rgba_speed(568, 568, &mut initial_image, 10);
        initial_frame.dispose = DisposalMethod::Background;
        initial_frame.delay = 100;
        encoder.write_frame(&initial_frame)?;

        for (i, (from, to)) in game_state.move_log.iter().enumerate() {
            if *from == 64 {
                state.castle_kingside(Color::from(*to as usize))?;
            } else if *from == 65 {
                state.castle_queenside(Color::from(*to as usize))?;
            } else {
                state.make_move(*from, *to)?;
            }

            let mut frame_image = render(&state, color)?;
            let mut frame = Frame::from_rgba_speed(568, 568, &mut frame_image, 10);
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
