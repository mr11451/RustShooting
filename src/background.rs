#[allow(dead_code)]
const FIXED_SHIFT: i32 = 4;
#[allow(dead_code)]
const FIXED_SCALE: i32 = 1 << FIXED_SHIFT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub struct TileSet {
    pub texture_id: u16,
    pub tile_width: u16,
    pub tile_height: u16,
    pub atlas_columns: u16,
    pub atlas_rows: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TileMap {
    pub width: u16,
    pub height: u32,
    pub tile_ids: Vec<u16>,
}

impl TileMap {
    pub fn from_text_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let text = std::fs::read_to_string(path)?;
        let rows: Vec<Vec<u16>> = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                line.split_whitespace()
                    .filter_map(|value| value.parse::<u16>().ok())
                    .collect()
            })
            .collect();
        let width = rows.first().map_or(0, Vec::len);
        let height = rows.len();
        let tile_ids = rows.into_iter().flatten().collect();
        Self::new(width as u16, height as u32, tile_ids)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid tile map"))
    }

    pub fn new(width: u16, height: u32, tile_ids: Vec<u16>) -> Option<Self> {
        let expected = usize::from(width).checked_mul(usize::try_from(height).ok()?)?;
        (tile_ids.len() == expected).then_some(Self {
            width,
            height,
            tile_ids,
        })
    }

    pub fn tile_id(&self, row: i32, column: u16) -> u16 {
        if row < 0 || row >= i32::try_from(self.height).unwrap_or(i32::MAX) {
            return 0;
        }

        let index =
            usize::try_from(row).unwrap_or(0) * usize::from(self.width) + usize::from(column);
        self.tile_ids.get(index).copied().unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub struct VisibleTile {
    pub tile_id: u16,
    pub atlas_x: u16,
    pub atlas_y: u16,
    pub screen_x: i32,
    pub screen_y: i32,
}

#[allow(dead_code)]
pub fn visible_tiles(
    tile_set: TileSet,
    tile_map: &TileMap,
    scroll_y_raw: i32,
    viewport_width: u16,
    viewport_height: u16,
) -> Vec<VisibleTile> {
    if tile_set.tile_width == 0
        || tile_set.tile_height == 0
        || tile_set.atlas_columns == 0
        || tile_set.atlas_rows == 0
        || tile_map.width == 0
    {
        return Vec::new();
    }

    let tile_height_raw = i32::from(tile_set.tile_height) * FIXED_SCALE;
    let first_row = floor_div(scroll_y_raw, tile_height_raw);
    let last_row = floor_div(
        scroll_y_raw + i32::from(viewport_height) * FIXED_SCALE - 1,
        tile_height_raw,
    );
    let visible_columns = u32::from(viewport_width).div_ceil(u32::from(tile_set.tile_width));
    let column_count = visible_columns.min(u32::from(tile_map.width));
    let mut result = Vec::new();

    for row in first_row..=last_row {
        for column in 0..u16::try_from(column_count).unwrap_or(u16::MAX) {
            let tile_id = tile_map.tile_id(row, column);
            if tile_id == 0 {
                continue;
            }

            let atlas_index = u32::from(tile_id);
            if atlas_index >= u32::from(tile_set.atlas_columns) * u32::from(tile_set.atlas_rows) {
                continue;
            }

            let screen_y_raw = row * tile_height_raw - scroll_y_raw;
            result.push(VisibleTile {
                tile_id,
                atlas_x: tile_id % tile_set.atlas_columns,
                atlas_y: tile_id / tile_set.atlas_columns,
                screen_x: i32::from(column) * i32::from(tile_set.tile_width),
                screen_y: screen_y_raw / FIXED_SCALE,
            });
        }
    }

    result
}

#[allow(dead_code)]
fn floor_div(value: i32, divisor: i32) -> i32 {
    let quotient = value / divisor;
    let remainder = value % divisor;
    if remainder != 0 && (remainder < 0) != (divisor < 0) {
        quotient - 1
    } else {
        quotient
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_map() -> TileMap {
        TileMap::new(2, 4, vec![1, 2, 3, 0, 4, 5, 0, 6]).unwrap()
    }

    fn sample_set() -> TileSet {
        TileSet {
            texture_id: 1,
            tile_width: 16,
            tile_height: 16,
            atlas_columns: 4,
            atlas_rows: 4,
        }
    }

    #[test]
    fn returns_only_visible_non_empty_tiles() {
        let tiles = visible_tiles(sample_set(), &sample_map(), 16 * FIXED_SCALE, 32, 32);

        assert_eq!(tiles.len(), 3);
        assert_eq!(tiles[0].screen_y, 0);
        assert_eq!((tiles[0].atlas_x, tiles[0].atlas_y), (3, 0));
        assert_eq!(tiles[1].tile_id, 4);
        assert_eq!(tiles[1].screen_y, 16);
    }

    #[test]
    fn handles_negative_scroll_with_floor_division() {
        let tiles = visible_tiles(sample_set(), &sample_map(), -8 * FIXED_SCALE, 16, 16);

        assert_eq!(tiles[0].tile_id, 1);
        assert_eq!(tiles[0].screen_y, 8);
    }

    #[test]
    fn rejects_invalid_map_size() {
        assert!(TileMap::new(2, 2, vec![1, 2, 3]).is_none());
    }
}
