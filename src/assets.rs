use std::{fmt, path::Path};

use image::RgbaImage;

use crate::sprite::{SpriteError, SpriteSheet};

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
    pub player_sheet: SpriteSheet,
    pub enemy_sheet: SpriteSheet,
    pub boss_sheet: SpriteSheet,
    pub item_sheet: SpriteSheet,
    pub bullet_sheet: SpriteSheet,
    pub enemy_bullet_sheet: SpriteSheet,
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
        Self::load_stage(
            "assets/characters/player.gif",
            "assets/characters/enemy_basic.gif",
            "assets/characters/boss.gif",
            "assets/characters/growth_item.gif",
            "assets/bullets/bullets.gif",
            "assets/bullets/enemy_bullets.gif",
            "assets/backgrounds/stage01_atlas.png",
        )
    }

    pub fn load_stage(
        character_path: impl AsRef<Path>,
        enemy_path: impl AsRef<Path>,
        boss_path: impl AsRef<Path>,
        item_path: impl AsRef<Path>,
        bullet_path: impl AsRef<Path>,
        enemy_bullet_path: impl AsRef<Path>,
        background_path: impl AsRef<Path>,
    ) -> Result<StageAssets, AssetError> {
        let player_sheet = SpriteSheet::from_gif_path(character_path, 32, 32)?;
        let enemy_sheet = SpriteSheet::from_gif_path(enemy_path, 32, 32)?;
        let boss_sheet = SpriteSheet::from_gif_path(boss_path, 64, 64)?;
        let item_sheet = SpriteSheet::from_gif_path(item_path, 32, 32)?;
        let bullet_sheet = SpriteSheet::from_gif_path(bullet_path, 32, 32)?;
        let enemy_bullet_sheet = SpriteSheet::from_gif_path(enemy_bullet_path, 8, 8)?;
        let background_atlas = image::open(background_path)?.into_rgba8();
        Ok(StageAssets {
            player_sheet,
            enemy_sheet,
            boss_sheet,
            item_sheet,
            bullet_sheet,
            enemy_bullet_sheet,
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
        assert_eq!(assets.player_sheet.columns, 2);
        assert_eq!(assets.enemy_sheet.columns, 2);
        assert_eq!(assets.boss_sheet.columns, 2);
        assert_eq!(assets.item_sheet.columns, 2);
        assert_eq!(assets.bullet_sheet.columns, 3);
        assert_eq!(assets.enemy_bullet_sheet.columns, 2);
        assert_eq!(assets.enemy_bullet_sheet.frame_width, 8);
        assert_eq!(assets.enemy_bullet_sheet.frame_height, 8);
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
