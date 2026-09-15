// Player bullet images and growth data.
use super::*;

pub const PLAYER_BULLET_IMAGE_DIRECTORY: &str = "assets/bullets";

const PLAYER_DIRECTIONS_LEVEL_0: &[Direction16] = &[Direction16::North];
const PLAYER_DIRECTIONS_LEVEL_1: &[Direction16] = &[Direction16::North, Direction16::North];
const PLAYER_DIRECTIONS_LEVEL_2: &[Direction16] = &[
    Direction16::NorthByWest,
    Direction16::North,
    Direction16::NorthByEast,
];
const PLAYER_DIRECTIONS_LEVEL_3: &[Direction16] = &[
    Direction16::NorthByWest,
    Direction16::North,
    Direction16::North,
    Direction16::NorthByEast,
];
const PLAYER_DIRECTIONS_LEVEL_4: &[Direction16] = &[
    Direction16::NorthByWest,
    Direction16::North,
    Direction16::North,
    Direction16::North,
    Direction16::NorthByEast,
];

pub const PLAYER_BULLET_IMAGE_DATA: &[PlayerBulletImageData] = &[
    PlayerBulletImageData {
        bullet_character_id: 1,
        image_file_name: "player_bullet_01.gif",
    },
    PlayerBulletImageData {
        bullet_character_id: 2,
        image_file_name: "player_bullet_02.gif",
    },
    PlayerBulletImageData {
        bullet_character_id: 3,
        image_file_name: "player_bullet_03.gif",
    },
    PlayerBulletImageData {
        bullet_character_id: 4,
        image_file_name: "player_bullet_04.gif",
    },
    PlayerBulletImageData {
        bullet_character_id: 5,
        image_file_name: "player_bullet_05.gif",
    },
];

pub fn player_bullet_image_data(
    bullet_character_id: u16,
) -> Option<&'static PlayerBulletImageData> {
    PLAYER_BULLET_IMAGE_DATA
        .iter()
        .find(|data| data.bullet_character_id == bullet_character_id)
}

pub const PLAYER_GROWTH_DATA: &[PlayerGrowthData] = &[
    PlayerGrowthData {
        growth_level: 0,
        max_bullet_groups: 4,
        bullets_per_group: 1,
        bullet_character_id: 1,
        speed: Q12_4(64),
        visual_width: 28,
        visual_height: 28,
        hitbox_half_width: 128,
        hitbox_half_height: 128,
        move_speed: Q12_4(32),
        directions: PLAYER_DIRECTIONS_LEVEL_0,
    },
    PlayerGrowthData {
        growth_level: 1,
        max_bullet_groups: 6,
        bullets_per_group: 2,
        bullet_character_id: 2,
        speed: Q12_4(67),
        visual_width: 30,
        visual_height: 30,
        hitbox_half_width: 144,
        hitbox_half_height: 144,
        move_speed: Q12_4(36),
        directions: PLAYER_DIRECTIONS_LEVEL_1,
    },
    PlayerGrowthData {
        growth_level: 2,
        max_bullet_groups: 8,
        bullets_per_group: 3,
        bullet_character_id: 3,
        speed: Q12_4(70),
        visual_width: 32,
        visual_height: 32,
        hitbox_half_width: 160,
        hitbox_half_height: 160,
        move_speed: Q12_4(40),
        directions: PLAYER_DIRECTIONS_LEVEL_2,
    },
    PlayerGrowthData {
        growth_level: 3,
        max_bullet_groups: 10,
        bullets_per_group: 4,
        bullet_character_id: 4,
        speed: Q12_4(74),
        visual_width: 36,
        visual_height: 36,
        hitbox_half_width: 176,
        hitbox_half_height: 176,
        move_speed: Q12_4(44),
        directions: PLAYER_DIRECTIONS_LEVEL_3,
    },
    PlayerGrowthData {
        growth_level: 4,
        max_bullet_groups: 12,
        bullets_per_group: 5,
        bullet_character_id: 5,
        speed: Q12_4(77),
        visual_width: 40,
        visual_height: 40,
        hitbox_half_width: 192,
        hitbox_half_height: 192,
        move_speed: Q12_4(48),
        directions: PLAYER_DIRECTIONS_LEVEL_4,
    },
];

pub fn player_growth_data(growth_level: u8) -> &'static PlayerGrowthData {
    &PLAYER_GROWTH_DATA[usize::from(growth_level.min(4))]
}
