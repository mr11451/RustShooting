use crate::fixed::Q12_4;

mod types;
pub use types::*;

pub mod bullet;
pub mod character;
pub mod orbit;
pub mod player;
pub mod schedule;

pub use bullet::*;
pub use character::*;
pub use orbit::*;
pub use player::*;
pub use schedule::*;

pub const LOGICAL_WIDTH: f32 = 480.0;
pub const LOGICAL_HEIGHT: f32 = 640.0;
pub const SCREEN_WIDTH: u16 = 480;
pub const SCREEN_HEIGHT: u16 = 640;
pub const SCREEN_WIDTH_Q12: Q12_4 = Q12_4(480 * 16); // 7_680
pub const SCREEN_HEIGHT_Q12: Q12_4 = Q12_4(640 * 16); // 10_240
pub const PLAYER_INITIAL_X_Q12: Q12_4 = Q12_4(240 * 16); // 3_840 (center)
pub const PLAYER_INITIAL_Y_Q12: Q12_4 = Q12_4(560 * 16); // 8_960 (bottom)
pub const BOSS_ORBIT_CENTER_X_Q12: Q12_4 = Q12_4(SCREEN_WIDTH_Q12.raw() / 2);
pub const BOSS_ORBIT_CENTER_Y_Q12: Q12_4 = Q12_4(220 * 16);
pub const BOSS_ORBIT_RADIUS_Q12: Q12_4 = Q12_4(96 * 16);
pub const HUD_HEIGHT: f32 = 16.0;
pub const HUD_Y: f32 = LOGICAL_HEIGHT - HUD_HEIGHT; // 624.0

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterType {
    Player,
    Enemy,
    Boss,
    GrowthItem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectType {
    Enemy,
    Boss,
    GrowthItem,
    #[allow(dead_code)]
    Background,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Direction32 {
    North,
    NorthByEast,
    NorthNorthEast,
    NorthEastByNorth,
    NorthEast,
    NorthEastByEast,
    EastNorthEast,
    EastByNorth,
    East,
    EastBySouth,
    EastSouthEast,
    SouthEastByEast,
    SouthEast,
    SouthEastBySouth,
    SouthSouthEast,
    SouthByEast,
    South,
    SouthByWest,
    SouthSouthWest,
    SouthWestBySouth,
    SouthWest,
    SouthWestByWest,
    WestSouthWest,
    WestBySouth,
    West,
    WestByNorth,
    WestNorthWest,
    NorthWestByWest,
    NorthWest,
    NorthWestByNorth,
    NorthNorthWest,
    NorthByWest,
}

pub use Direction32 as Direction16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrbitType {
    Straight,
    Circle,
    Bezier,
    MoveToPosition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrbitData {
    pub orbit_id: u16,
    pub orbit_type: OrbitType,
    pub direction: Direction16,
    pub speed: Q12_4,
    pub radius: Q12_4,
    pub control_point_1: (Q12_4, Q12_4),
    pub control_point_2: (Q12_4, Q12_4),
    pub control_point_end: (Q12_4, Q12_4),
    pub target_position: (Q12_4, Q12_4),
    pub start_angle: Option<u16>,
    pub end_angle: Option<u16>,
    pub duration_frames: u32,
    pub acceleration: Q12_4,
    pub next_orbit_id: u16,
    pub rotation: i8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub struct StageData {
    pub stage_id: u8,
    pub play_frames: u32,
    pub intro_frames: u16,
    pub clear_frames: u16,
    pub boss_character_id: u16,
    pub schedule_start_id: u16,
    pub item_count: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub struct BackgroundData {
    pub background_id: u16,
    pub tile_width: u16,
    pub tile_height: u16,
    pub atlas_columns: u16,
    pub atlas_rows: u16,
    pub scroll_speed: Q12_4,
}

#[allow(dead_code)]
pub const STAGE_ONE_BACKGROUND: BackgroundData = BackgroundData {
    background_id: 1,
    tile_width: 16,
    tile_height: 16,
    atlas_columns: 4,
    atlas_rows: 4,
    scroll_speed: Q12_4(16),
};

pub fn direction_velocity(direction: Direction16, speed: Q12_4) -> (Q12_4, Q12_4) {
    let index = direction_index(direction);
    let angle = (index as f32) * std::f32::consts::TAU / 32.0;
    let speed = f32::from(speed.raw());
    (
        Q12_4((angle.sin() * speed).round() as i16),
        Q12_4((-angle.cos() * speed).round() as i16),
    )
}

pub fn direction_index(direction: Direction16) -> u8 {
    use Direction16::*;
    match direction {
        North => 0,
        NorthByEast => 1,
        NorthNorthEast => 2,
        NorthEastByNorth => 3,
        NorthEast => 4,
        NorthEastByEast => 5,
        EastNorthEast => 6,
        EastByNorth => 7,
        East => 8,
        EastBySouth => 9,
        EastSouthEast => 10,
        SouthEastByEast => 11,
        SouthEast => 12,
        SouthEastBySouth => 13,
        SouthSouthEast => 14,
        SouthByEast => 15,
        South => 16,
        SouthByWest => 17,
        SouthSouthWest => 18,
        SouthWestBySouth => 19,
        SouthWest => 20,
        SouthWestByWest => 21,
        WestSouthWest => 22,
        WestBySouth => 23,
        West => 24,
        WestByNorth => 25,
        WestNorthWest => 26,
        NorthWestByWest => 27,
        NorthWest => 28,
        NorthWestByNorth => 29,
        NorthNorthWest => 30,
        NorthByWest => 31,
    }
}

pub fn direction_from_index(index: u8) -> Direction16 {
    const DIRECTIONS: [Direction16; 32] = [
        Direction16::North,
        Direction16::NorthByEast,
        Direction16::NorthNorthEast,
        Direction16::NorthEastByNorth,
        Direction16::NorthEast,
        Direction16::NorthEastByEast,
        Direction16::EastNorthEast,
        Direction16::EastByNorth,
        Direction16::East,
        Direction16::EastBySouth,
        Direction16::EastSouthEast,
        Direction16::SouthEastByEast,
        Direction16::SouthEast,
        Direction16::SouthEastBySouth,
        Direction16::SouthSouthEast,
        Direction16::SouthByEast,
        Direction16::South,
        Direction16::SouthByWest,
        Direction16::SouthSouthWest,
        Direction16::SouthWestBySouth,
        Direction16::SouthWest,
        Direction16::SouthWestByWest,
        Direction16::WestSouthWest,
        Direction16::WestBySouth,
        Direction16::West,
        Direction16::WestByNorth,
        Direction16::WestNorthWest,
        Direction16::NorthWestByWest,
        Direction16::NorthWest,
        Direction16::NorthWestByNorth,
        Direction16::NorthNorthWest,
        Direction16::NorthByWest,
    ];
    DIRECTIONS[usize::from(index % 32)]
}

#[allow(dead_code)]
pub fn validate_all_stages() -> Result<(), String> {
    for stage in STAGES {
        let boss_trait = character_trait(stage.boss_character_id)
            .ok_or_else(|| format!("unknown boss character ID: {}", stage.boss_character_id))?;
        if boss_trait.character_type != CharacterType::Boss {
            return Err(format!(
                "character {} is not a Boss",
                stage.boss_character_id
            ));
        }
    }
    for character in CHARACTER_TRAITS {
        if character.bullet_character_id != 0
            && bullet_character(character.bullet_character_id).is_none()
        {
            return Err(format!(
                "unknown bullet character ID: {}",
                character.bullet_character_id
            ));
        }
    }
    for schedule in all_schedules() {
        if character_trait(schedule.character_id).is_none() {
            return Err(format!("unknown character ID: {}", schedule.character_id));
        }
        if schedule.orbit_id != 0 && orbit(schedule.orbit_id).is_none() {
            return Err(format!("unknown orbit ID: {}", schedule.orbit_id));
        }
        if schedule.fire_pattern_id != 0 && fire_pattern(schedule.fire_pattern_id).is_none() {
            return Err(format!(
                "unknown fire pattern ID: {}",
                schedule.fire_pattern_id
            ));
        }
    }
    for pattern in FIRE_PATTERNS {
        if bullet_character(pattern.bullet_character_id).is_none() {
            return Err(format!(
                "unknown fire pattern bullet character ID: {}",
                pattern.bullet_character_id
            ));
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn validate_stage_one() -> Result<(), String> {
    validate_all_stages()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_stages_have_final_frame_boss() {
        assert_eq!(STAGES.len(), 6);
        for stage in STAGES {
            let boss = stage_schedule(stage.stage_id)
                .find(|entry| entry.object_type == ObjectType::Boss)
                .unwrap_or_else(|| panic!("stage {} boss should exist", stage.stage_id));
            assert_eq!(boss.frame, stage.play_frames - 1);
            assert_eq!(boss.character_id, stage.boss_character_id);
        }
    }

    #[test]
    fn character_ids_resolve() {
        assert_eq!(character_trait(1).map(|item| item.max_hp), Some(100));
        assert_eq!(character_trait(999), None);
        assert_eq!(
            bullet_character(1).map(|bullet| bullet.fire_sound_id),
            Some(1)
        );
        assert_eq!(
            bullet_character(2).map(|bullet| bullet.player_damage),
            Some(10)
        );
        assert_eq!(bullet_character(1).map(|bullet| bullet.damage), Some(1));
        assert!(validate_all_stages().is_ok());
    }

    #[test]
    fn player_growth_data_defines_bullet_groups_and_types() {
        let levels = (0..=4).map(player_growth_data).collect::<Vec<_>>();

        assert_eq!(levels[0].max_bullet_groups, 4);
        assert_eq!(levels[0].bullets_per_group, 1);
        assert_eq!(
            levels
                .iter()
                .map(|data| data.bullet_character_id)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        assert_eq!(levels[4].max_bullet_groups, 12);
        assert_eq!(levels[4].bullets_per_group, 5);
        assert_eq!(levels[4].speed, Q12_4(77));
        assert_eq!(levels[4].directions.len(), 5);
        assert_eq!(levels[4].directions[2], Direction16::North);
    }

    #[test]
    fn enemy_traits_define_their_bullet_types() {
        assert_eq!(character_trait(2).unwrap().bullet_character_id, 6);
        assert_eq!(character_trait(4).unwrap().bullet_character_id, 8);
        assert_eq!(character_trait(7).unwrap().bullet_character_id, 16);
        assert_eq!(character_trait(105).unwrap().bullet_character_id, 16);
    }

    #[test]
    fn scheduled_enemies_can_fire_all_enemy_bullet_types() {
        assert_eq!(
            scheduled_enemy_bullet_character_ids(),
            vec![6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]
        );
    }

    #[test]
    fn stage_six_boss_uses_aiming_homing_bullet_setup() {
        let schedule = stage_schedule(6)
            .find(|schedule| schedule.object_type == ObjectType::Boss)
            .expect("stage six boss schedule should exist");
        let boss = character_trait(schedule.character_id).expect("boss trait should exist");

        assert_eq!(schedule.character_id, 105);
        assert_eq!(schedule.fire_pattern_id, 15);
        assert_eq!(
            fire_pattern(schedule.fire_pattern_id).unwrap().angle_mode,
            FireAngleMode::AimAtPlayer
        );
        assert!(bullet_character(boss.bullet_character_id).unwrap().homing);
    }

    #[test]
    fn stage_one_has_non_boss_aiming_and_fixed_fire_enemies() {
        let enemies = stage_schedule(1)
            .filter(|schedule| schedule.object_type == ObjectType::Enemy)
            .collect::<Vec<_>>();
        assert!(enemies.iter().any(|schedule| {
            schedule.character_id == 7
                && fire_pattern(schedule.fire_pattern_id)
                    .is_some_and(|pattern| pattern.angle_mode == FireAngleMode::AimAtPlayer)
        }));
        assert!(enemies.iter().any(|schedule| {
            schedule.character_id != 105
                && fire_pattern(schedule.fire_pattern_id)
                    .is_some_and(|pattern| pattern.angle_mode == FireAngleMode::Fixed)
        }));
    }

    #[test]
    fn non_boss_enemy_has_aimed_but_non_homing_fire() {
        let schedule = stage_schedule(1)
            .find(|schedule| schedule.character_id == 6 && schedule.fire_pattern_id == 15)
            .expect("non-boss aimed straight schedule should exist");
        let enemy = character_trait(schedule.character_id).expect("enemy trait should exist");
        let pattern = fire_pattern(schedule.fire_pattern_id).expect("fire pattern should exist");

        assert_eq!(schedule.object_type, ObjectType::Enemy);
        assert_eq!(pattern.angle_mode, FireAngleMode::AimAtPlayer);
        assert!(!bullet_character(enemy.bullet_character_id).unwrap().homing);
    }

    #[test]
    fn stages_define_enemy_ids_from_their_schedules() {
        assert_eq!(stage_enemy_character_ids(1), vec![2, 3, 4, 5, 6, 7]);
        assert_eq!(stage_enemy_character_ids(2), vec![3, 4]);
        assert_eq!(stage_enemy_character_ids(3), vec![5]);
        assert_eq!(stage_enemy_character_ids(4), vec![4, 6]);
        assert_eq!(stage_enemy_character_ids(5), vec![5, 6]);
        assert_eq!(stage_enemy_character_ids(6), vec![6, 4]);
    }

    #[test]
    fn stage_one_preview_covers_all_enemy_ids_and_orbit_types() {
        let enemies = stage_schedule(1)
            .filter(|schedule| schedule.object_type == ObjectType::Enemy)
            .collect::<Vec<_>>();
        assert!((2..=7).all(|character_id| {
            enemies
                .iter()
                .any(|schedule| schedule.character_id == character_id)
        }));
        assert!(
            [1u16, 2, 3, 4, 5, 6, 7, 8, 9, 10]
                .into_iter()
                .all(|orbit_id| enemies.iter().any(|schedule| schedule.orbit_id == orbit_id))
        );

        assert!(enemies.iter().any(|schedule| {
            orbit(schedule.orbit_id).is_some_and(|data| data.orbit_type == OrbitType::Straight)
        }));
        assert!(enemies.iter().any(|schedule| {
            orbit(schedule.orbit_id).is_some_and(|data| data.orbit_type == OrbitType::Circle)
        }));
        assert!(enemies.iter().any(|schedule| {
            orbit(schedule.orbit_id).is_some_and(|data| data.orbit_type == OrbitType::Bezier)
        }));
        assert!(enemies.iter().any(|schedule| {
            orbit(schedule.orbit_id)
                .is_some_and(|data| data.orbit_type == OrbitType::MoveToPosition)
        }));
    }

    #[test]
    fn stage_enemy_image_paths_are_data_driven() {
        assert_eq!(
            stage_enemy_image_data(1, 2).unwrap().image_file_name,
            "stage01_enemy_02.gif"
        );
        assert!(stage_enemy_image_data(99, 2).is_none());
    }

    #[test]
    fn fire_pattern_direction_converts_to_velocity() {
        let pattern = fire_pattern(1).unwrap();
        assert_eq!(pattern.velocity(), (Q12_4::ZERO, Q12_4(20)));
        assert_eq!(fire_pattern(1).unwrap().angle_mode, FireAngleMode::Fixed);
        assert_eq!(
            fire_pattern(15).unwrap().angle_mode,
            FireAngleMode::AimAtPlayer
        );
        assert_eq!(fire_pattern(10).unwrap().volley_count, 32);
        assert_eq!(fire_pattern(10).unwrap().direction_step, 1);
        assert_eq!(fire_pattern(11).unwrap().direction_step, -1);
        assert_eq!(fire_pattern(10).unwrap().repeat_interval_frames, 240);
        assert_eq!(fire_pattern(10).unwrap().volley_interval_frames, 6);
        assert_eq!(additional_fire_pattern_ids(100), &[11]);
    }

    #[test]
    fn direction_velocity_supports_thirty_two_directions() {
        let (x, y) = direction_velocity(Direction16::NorthByEast, Q12_4(64));
        assert!(x.raw() > 0);
        assert!(y.raw() < 0);
    }
}
