use std::{fmt, path::Path};

use image::RgbaImage;

use crate::{
    data::{
        PLAYER_BULLET_IMAGE_DIRECTORY, STAGE_ENEMY_IMAGE_DIRECTORY, player_bullet_image_data,
        stage_enemy_character_ids, stage_enemy_image_data,
    },
    sprite::{SpriteError, SpriteSheet},
};

#[derive(Debug)]
pub enum AssetError {
    Sprite(SpriteError),
    Image(image::ImageError),
}

impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sprite(error) => write!(formatter, "sprite asset error: {error}"),
            Self::Image(error) => write!(formatter, "background asset error: {error}"),
        }
    }
}

impl std::error::Error for AssetError {}

impl From<SpriteError> for AssetError {
    fn from(error: SpriteError) -> Self {
        Self::Sprite(error)
    }
}

impl From<image::ImageError> for AssetError {
    fn from(error: image::ImageError) -> Self {
        Self::Image(error)
    }
}

pub struct StageAssets {
    pub player_variant_sheets: Vec<SpriteSheet>,
    pub stage_enemy_sheets: Vec<Vec<SpriteSheet>>,
    pub boss_sheet: SpriteSheet,
    pub item_sheet: SpriteSheet,
    pub player_bullet_sheets: Vec<SpriteSheet>,
    pub enemy_bullet_variant_sheets: Vec<SpriteSheet>,
    pub background_atlas: RgbaImage,
}

pub struct AssetCatalog;

#[derive(Default)]
#[allow(dead_code)]
pub struct StageAssetCache {
    pub stage_id: Option<u8>,
    pub assets: Option<StageAssets>,
}

#[allow(dead_code)]
impl StageAssetCache {
    pub fn load_stage_one(&mut self) -> Result<&StageAssets, AssetError> {
        if self.stage_id != Some(1) {
            self.assets = Some(AssetCatalog::load_stage_one()?);
            self.stage_id = Some(1);
        }
        Ok(self.assets.as_ref().expect("stage assets loaded"))
    }

    pub fn unload(&mut self) {
        self.stage_id = None;
        self.assets = None;
    }
}

impl AssetCatalog {
    pub fn load_stage_one() -> Result<StageAssets, AssetError> {
        Self::load_stage_id(1)
    }

    pub fn load_stage_id(stage_id: u8) -> Result<StageAssets, AssetError> {
        Self::load_stage(
            "assets/characters/boss.gif",
            "assets/characters/growth_item.gif",
            format!("assets/backgrounds/stage{:02}_atlas.png", stage_id),
        )
    }

    pub fn load_stage(
        boss_path: impl AsRef<Path>,
        item_path: impl AsRef<Path>,
        background_path: impl AsRef<Path>,
    ) -> Result<StageAssets, AssetError> {
        let player_variant_sheets = (0..=4)
            .map(|level| {
                SpriteSheet::from_gif_path(
                    format!("assets/characters/player_level_{level:02}.gif"),
                    32,
                    32,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut stage_enemy_sheets = Vec::with_capacity(6);
        for stage_id in 1..=6 {
            let sheets = stage_enemy_character_ids(stage_id)
                .into_iter()
                .filter_map(|character_id| stage_enemy_image_data(stage_id, character_id))
                .map(|data| {
                    SpriteSheet::from_gif_path(
                        format!("{STAGE_ENEMY_IMAGE_DIRECTORY}/{}", data.image_file_name),
                        32,
                        32,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            stage_enemy_sheets.push(sheets);
        }
        let boss_sheet = SpriteSheet::from_gif_path(boss_path, 64, 64)?;
        let item_sheet = SpriteSheet::from_gif_path(item_path, 32, 32)?;
        let player_bullet_sheets = (1..=5)
            .filter_map(player_bullet_image_data)
            .map(|data| {
                SpriteSheet::from_gif_path(
                    format!("{PLAYER_BULLET_IMAGE_DIRECTORY}/{}", data.image_file_name),
                    16,
                    16,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let enemy_bullet_variant_sheets = (6..=16)
            .map(|bullet_id| {
                SpriteSheet::from_gif_path(
                    format!("assets/bullets/enemy_bullet_{bullet_id:02}.gif"),
                    8,
                    8,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let background_atlas = image::open(background_path)?.into_rgba8();
        Ok(StageAssets {
            player_variant_sheets,
            stage_enemy_sheets,
            boss_sheet,
            item_sheet,
            player_bullet_sheets,
            enemy_bullet_variant_sheets,
            background_atlas,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_stage_one_assets() {
        let assets = AssetCatalog::load_stage_one().expect("stage one assets should load");
        assert_eq!(assets.player_variant_sheets.len(), 5);
        assert_eq!(assets.stage_enemy_sheets.len(), 6);
        for (stage_index, sheets) in assets.stage_enemy_sheets.iter().enumerate() {
            assert_eq!(
                sheets.len(),
                stage_enemy_character_ids(stage_index as u8 + 1).len()
            );
        }
        assert_eq!(assets.boss_sheet.columns, 2);
        assert_eq!(assets.item_sheet.columns, 2);
        assert_eq!(assets.player_bullet_sheets.len(), 5);
        assert!(assets.player_bullet_sheets.iter().all(|sheet| {
            sheet.frame_width == 16
                && sheet.frame_height == 16
                && sheet.columns == 2
                && sheet.rows == 1
                && sheet.animation_count() == 1
        }));
        assert_eq!(assets.enemy_bullet_variant_sheets.len(), 11);
        assert_eq!(assets.boss_sheet.frame_width, 64);
        assert_eq!(assets.boss_sheet.frame_height, 64);
        assert_eq!(assets.background_atlas.dimensions(), (64, 64));
    }

    #[test]
    fn reuses_and_unloads_stage_cache() {
        let mut cache = StageAssetCache::default();
        let first = cache
            .load_stage_one()
            .unwrap()
            .background_atlas
            .dimensions();
        let second = cache
            .load_stage_one()
            .unwrap()
            .background_atlas
            .dimensions();
        assert_eq!(first, second);
        cache.unload();
        assert!(cache.assets.is_none());
    }
}
