use gif::{DisposalMethod, Encoder, Frame, Repeat};
use image::{DynamicImage, ImageBuffer, Rgba};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use utoipa::ToSchema;

use crate::error::ApiError;

use super::{color::Color, piece::Piece, state::GameState};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, ToSchema)]
pub enum RenderStyle {
    PIXEL,
    MODERN,
}

struct StyleConfig {
    asset_path: &'static str,
    top_left_x: i64,
    top_left_y: i64,
    step_x: i64,
    step_y: i64,
    /// Width / Height
    board_size: (u16, u16),
    piece_size: (u16, u16),
    filter: image::imageops::FilterType,
}

impl StyleConfig {
    fn new(style: &RenderStyle) -> Self {
        match style {
            RenderStyle::PIXEL => Self {
                asset_path: "src/assets/pixel/",
                top_left_x: 7,
                top_left_y: -2,
                step_x: 16,
                step_y: 12,
                board_size: (568, 568),
                piece_size: (16, 32),
                filter: image::imageops::FilterType::Nearest,
            },
            RenderStyle::MODERN => Self {
                asset_path: "src/assets/modern/",
                top_left_x: 115,
                top_left_y: 114,
                step_x: 102,
                step_y: 102,
                board_size: (1024, 1024),
                piece_size: (85, 85),
                filter: image::imageops::FilterType::CatmullRom,
            },
        }
    }
}

pub fn render(state: &GameState, color: Color, style: &RenderStyle) -> Result<Vec<u8>, ApiError> {
    let config = StyleConfig::new(style);

    let mut board = image::open(format!(
        "{}board_{}.png",
        config.asset_path,
        if color == Color::WHITE {
            "white"
        } else {
            "black"
        }
    ))?
    .to_rgba8();
    let chess_board = if color == Color::WHITE {
        state.chess_board.clone()
    } else {
        state.chess_board.rotate()
    };

    for index in (0..64).rev() {
        let (piece, piece_color) = chess_board.piece_and_color_at_cell(index).unwrap();
        if piece == Piece::NONE || piece_color == Color::NONE {
            continue;
        }
        let (x, y) = calculate_coordinates(index, &config);
        let path = format!("{}{}", config.asset_path, piece.get_image_name(piece_color));
        let piece_image = image::open(path)?
            .resize(
                config.piece_size.0 as u32,
                config.piece_size.1 as u32,
                config.filter,
            )
            .to_rgba8();
        image::imageops::overlay(&mut board, &piece_image, x, y);
    }

    let upscaled_image = image::imageops::resize(
        &board,
        config.board_size.0 as u32,
        config.board_size.1 as u32,
        config.filter,
    );

    Ok(upscaled_image.into_raw())
}

pub fn render_board_png(
    game_state: &GameState,
    color: Color,
    style: &RenderStyle,
) -> Result<Vec<u8>, ApiError> {
    let config = StyleConfig::new(style);

    let raw_data = render(game_state, color, style)?;
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
        config.board_size.0 as u32,
        config.board_size.1 as u32,
        raw_data,
    )
    .ok_or(ApiError::ServerError(
        "Failed to create image buffer.".to_string(),
    ))?;
    let dynamic_image = DynamicImage::ImageRgba8(buffer);
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    dynamic_image.write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(png_bytes)
}

pub fn render_history_gif(
    game_state: &GameState,
    color: Color,
    style: &RenderStyle,
) -> Result<Vec<u8>, ApiError> {
    let config = StyleConfig::new(style);

    // TODO: Use `with_capacity`. At least estimate the upper bound.
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    {
        let mut encoder = Encoder::new(&mut cursor, config.board_size.0, config.board_size.1, &[])?;
        encoder.set_repeat(Repeat::Infinite)?;

        let mut state = GameState::new()?;
        let mut initial_image = render(&state, color, style)?;
        let mut initial_frame = Frame::from_rgba_speed(
            config.board_size.0,
            config.board_size.1,
            &mut initial_image,
            10,
        );
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

            let mut frame_image = render(&state, color, style)?;
            let mut frame = Frame::from_rgba_speed(
                config.board_size.0,
                config.board_size.1,
                &mut frame_image,
                10,
            );
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

fn calculate_coordinates(index: u8, config: &StyleConfig) -> (i64, i64) {
    let current_row = (index / 8) as i64;
    let current_col = (index % 8) as i64;
    let x = config.top_left_x + current_col * config.step_x;
    let y = config.top_left_y + (7 - current_row) * config.step_y;
    (x, y)
}
