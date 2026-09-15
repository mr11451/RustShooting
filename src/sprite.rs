use std::{fmt, fs, io::Cursor, path::Path};

use image::{AnimationDecoder, RgbaImage, codecs::gif::GifDecoder};

#[derive(Debug)]
pub enum SpriteError {
    Io(std::io::Error),
    Decode(image::ImageError),
    InvalidTileSize,
    FrameTooSmall { width: u32, height: u32 },
    InconsistentFrameSize,
}

impl fmt::Display for SpriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "failed to read sprite GIF: {error}"),
            Self::Decode(error) => write!(formatter, "failed to decode sprite GIF: {error}"),
            Self::InvalidTileSize => write!(formatter, "sprite tile size must be non-zero"),
            Self::FrameTooSmall { width, height } => {
                write!(
                    formatter,
                    "sprite frame {width}x{height} is smaller than a tile"
                )
            }
            Self::InconsistentFrameSize => {
                write!(formatter, "sprite GIF frames have different sizes")
            }
        }
    }
}

impl std::error::Error for SpriteError {}

impl From<std::io::Error> for SpriteError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<image::ImageError> for SpriteError {
    fn from(error: image::ImageError) -> Self {
        Self::Decode(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteFrame {
    pub animation_id: u16,
    pub tile_id: u16,
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpriteSheet {
    pub frame_width: u32,
    pub frame_height: u32,
    pub columns: u32,
    pub rows: u32,
    pub frames: Vec<SpriteFrame>,
}

impl SpriteSheet {
    pub fn from_gif_path(
        path: impl AsRef<Path>,
        tile_width: u32,
        tile_height: u32,
    ) -> Result<Self, SpriteError> {
        let bytes = fs::read(path)?;
        Self::from_gif_bytes(&bytes, tile_width, tile_height)
    }

    pub fn from_gif_bytes(
        bytes: &[u8],
        tile_width: u32,
        tile_height: u32,
    ) -> Result<Self, SpriteError> {
        if tile_width == 0 || tile_height == 0 {
            return Err(SpriteError::InvalidTileSize);
        }

        let decoder = GifDecoder::new(Cursor::new(bytes))?;
        let animation_frames = decoder.into_frames().collect_frames()?;
        let mut frames = Vec::new();
        let mut sheet_width = None;
        let mut sheet_height = None;
        let mut columns = 0;
        let mut rows = 0;

        for (animation_index, animation_frame) in animation_frames.into_iter().enumerate() {
            let image = animation_frame.into_buffer();
            let (width, height) = image.dimensions();
            if width < tile_width || height < tile_height {
                return Err(SpriteError::FrameTooSmall { width, height });
            }

            if let (Some(expected_width), Some(expected_height)) = (sheet_width, sheet_height) {
                if expected_width != width || expected_height != height {
                    return Err(SpriteError::InconsistentFrameSize);
                }
            } else {
                sheet_width = Some(width);
                sheet_height = Some(height);
                columns = width / tile_width;
                rows = height / tile_height;
            }

            let animation_id = u16::try_from(animation_index).unwrap_or(u16::MAX);
            for row in 0..rows {
                for column in 0..columns {
                    let tile_id = u16::try_from(row * columns + column).unwrap_or(u16::MAX);
                    let tile = image::imageops::crop_imm(
                        &image,
                        column * tile_width,
                        row * tile_height,
                        tile_width,
                        tile_height,
                    )
                    .to_image();
                    frames.push(SpriteFrame {
                        animation_id,
                        tile_id,
                        width: tile_width,
                        height: tile_height,
                        rgba8: tile.into_raw(),
                    });
                }
            }
        }

        Ok(Self {
            frame_width: tile_width,
            frame_height: tile_height,
            columns,
            rows,
            frames,
        })
    }

    pub fn frame(&self, animation_id: u16, tile_id: u16) -> Option<&SpriteFrame> {
        self.frames
            .iter()
            .find(|frame| frame.animation_id == animation_id && frame.tile_id == tile_id)
    }

    #[allow(dead_code)]
    pub fn animation_count(&self) -> usize {
        self.frames
            .iter()
            .map(|frame| frame.animation_id)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }
}

#[allow(dead_code)]
fn _assert_rgba_image(_: &RgbaImage) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_placeholder_player_sprite_sheet() {
        let sheet = SpriteSheet::from_gif_path("assets/characters/player.gif", 32, 32)
            .expect("placeholder player GIF should load");

        assert_eq!(sheet.columns, 2);
        assert_eq!(sheet.rows, 1);
        assert_eq!(sheet.animation_count(), 1);
        assert_eq!(
            sheet.frame(0, 0).map(|frame| frame.rgba8.len()),
            Some(32 * 32 * 4)
        );
    }

    #[test]
    fn loads_placeholder_bullet_sprite_sheet() {
        let sheet = SpriteSheet::from_gif_path("assets/bullets/bullets.gif", 32, 32)
            .expect("placeholder bullet GIF should load");

        assert_eq!(sheet.columns, 3);
        assert_eq!(sheet.rows, 1);
        assert!(sheet.frame(0, 2).is_some());
    }

    #[test]
    fn loads_placeholder_enemy_bullet_sprite_sheet() {
        let sheet = SpriteSheet::from_gif_path("assets/bullets/enemy_bullets.gif", 8, 8)
            .expect("placeholder enemy bullet GIF should load");

        assert_eq!(sheet.columns, 2);
        assert_eq!(sheet.rows, 1);
        assert!(sheet.frame(0, 1).is_some());
    }

    #[test]
    fn loads_placeholder_growth_item_sprite_sheet() {
        let sheet = SpriteSheet::from_gif_path("assets/characters/growth_item.gif", 32, 32)
            .expect("placeholder growth item GIF should load");

        assert_eq!(sheet.columns, 2);
        assert_eq!(sheet.rows, 1);
    }
}
