use crate::fixed::Q12_4;

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
    Background,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction16 {
    North,
    NorthNorthEast,
    NorthEast,
    EastNorthEast,
    East,
    EastSouthEast,
    SouthEast,
    SouthSouthEast,
    South,
    SouthSouthWest,
    SouthWest,
    WestSouthWest,
    West,
    WestNorthWest,
    NorthWest,
    NorthNorthWest,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OrbitType {
    Straight,
    Circle,
    Bezier,
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
    pub start_angle: Option<u16>,
    pub end_angle: Option<u16>,
    pub duration_frames: u32,
    pub acceleration: Q12_4,
    pub next_orbit_id: u16,
    pub rotation: i8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
pub struct BackgroundData {
    pub background_id: u16,
    pub tile_width: u16,
    pub tile_height: u16,
    pub atlas_columns: u16,
    pub atlas_rows: u16,
    pub scroll_speed: Q12_4,
}

pub const STAGE_ONE_BACKGROUND: BackgroundData = BackgroundData {
    background_id: 1,
    tile_width: 16,
    tile_height: 16,
    atlas_columns: 4,
    atlas_rows: 4,
    scroll_speed: Q12_4(16),
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharacterTrait {
    pub character_id: u16,
    pub character_type: CharacterType,
    pub shape_id: u16,
    pub animation_id: u16,
    pub hitbox_width: i16,
    pub hitbox_height: i16,
    pub max_hp: u16,
    pub contact_damage: u16,
    pub player_damage: u16,
    pub score: u32,
    pub default_orbit_id: u16,
    pub default_fire_pattern_id: u16,
    pub bullet_character_id: u16,
    pub growth_effect_id: u16,
    pub destroy_effect_id: u16,
    pub destroy_sound_id: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduleData {
    pub schedule_id: u16,
    pub stage_id: u8,
    pub frame: u32,
    pub spawn_x: Q12_4,
    pub spawn_y: Q12_4,
    pub object_type: ObjectType,
    pub character_id: u16,
    pub orbit_id: u16,
    pub difficulty: u8,
    pub fire_pattern_id: u16,
    pub background_speed: Q12_4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirePatternData {
    pub fire_pattern_id: u16,
    pub fire_frame: u32,
    pub spawn_offset_x: Q12_4,
    pub spawn_offset_y: Q12_4,
    pub bullet_character_id: u16,
    pub direction: Direction16,
    pub speed: Q12_4,
    pub bullet_velocity_x: Q12_4,
    pub bullet_velocity_y: Q12_4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BulletCharacterData {
    pub bullet_character_id: u16,
    pub fire_sound_id: u16,
    pub hit_sound_id: u16,
    pub hitbox_width: i16,
    pub hitbox_height: i16,
    pub damage: u16,
    pub penetrating: bool,
    pub player_damage: u16,
}

pub const BULLET_CHARACTER_DATA: &[BulletCharacterData] = &[
    BulletCharacterData {
        bullet_character_id: 1,
        fire_sound_id: 1,
        hit_sound_id: 2,
        hitbox_width: 8,
        hitbox_height: 16,
        damage: 1,
        penetrating: false,
        player_damage: 10,
    },
    BulletCharacterData {
        bullet_character_id: 2,
        fire_sound_id: 1,
        hit_sound_id: 2,
        hitbox_width: 8,
        hitbox_height: 16,
        damage: 1,
        penetrating: false,
        player_damage: 10,
    },
    BulletCharacterData {
        bullet_character_id: 3,
        fire_sound_id: 1,
        hit_sound_id: 2,
        hitbox_width: 8,
        hitbox_height: 16,
        damage: 1,
        penetrating: false,
        player_damage: 10,
    },
    BulletCharacterData {
        bullet_character_id: 4,
        fire_sound_id: 1,
        hit_sound_id: 2,
        hitbox_width: 8,
        hitbox_height: 16,
        damage: 1,
        penetrating: true,
        player_damage: 10,
    },
];

pub fn bullet_character(bullet_character_id: u16) -> Option<&'static BulletCharacterData> {
    BULLET_CHARACTER_DATA
        .iter()
        .find(|bullet| bullet.bullet_character_id == bullet_character_id)
}

pub const FIRE_PATTERNS: &[FirePatternData] = &[
    FirePatternData {
        fire_pattern_id: 1,
        fire_frame: 30,
        spawn_offset_x: Q12_4::ZERO,
        spawn_offset_y: Q12_4::ZERO,
        bullet_character_id: 2,
        direction: Direction16::South,
        speed: Q12_4(16),
        bullet_velocity_x: Q12_4::ZERO,
        bullet_velocity_y: Q12_4(16),
    },
    FirePatternData {
        fire_pattern_id: 10,
        fire_frame: 60,
        spawn_offset_x: Q12_4::ZERO,
        spawn_offset_y: Q12_4(256),
        bullet_character_id: 3,
        direction: Direction16::South,
        speed: Q12_4(16),
        bullet_velocity_x: Q12_4::ZERO,
        bullet_velocity_y: Q12_4(16),
    },
];

pub fn fire_pattern(fire_pattern_id: u16) -> Option<&'static FirePatternData> {
    FIRE_PATTERNS
        .iter()
        .find(|pattern| pattern.fire_pattern_id == fire_pattern_id)
}

pub const STAGE_ONE: StageData = StageData {
    stage_id: 1,
    play_frames: 5_400,
    intro_frames: 90,
    clear_frames: 90,
    boss_character_id: 100,
    schedule_start_id: 1,
    item_count: 4,
};

pub const CHARACTER_TRAITS: &[CharacterTrait] = &[
    CharacterTrait {
        character_id: 1,
        character_type: CharacterType::Player,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 256,
        hitbox_height: 256,
        max_hp: 100,
        contact_damage: 0,
        player_damage: 0,
        score: 0,
        default_orbit_id: 0,
        default_fire_pattern_id: 0,
        bullet_character_id: 1,
        growth_effect_id: 0,
        destroy_effect_id: 0,
        destroy_sound_id: 0,
    },
    CharacterTrait {
        character_id: 2,
        character_type: CharacterType::Enemy,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 256,
        hitbox_height: 256,
        max_hp: 10,
        contact_damage: 10,
        player_damage: 10,
        score: 100,
        default_orbit_id: 1,
        default_fire_pattern_id: 1,
        bullet_character_id: 2,
        growth_effect_id: 0,
        destroy_effect_id: 1,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 100,
        character_type: CharacterType::Boss,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 1024,
        hitbox_height: 768,
        max_hp: 1_000,
        contact_damage: 50,
        player_damage: 50,
        score: 10_000,
        default_orbit_id: 3,
        default_fire_pattern_id: 10,
        bullet_character_id: 2,
        growth_effect_id: 0,
        destroy_effect_id: 2,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 200,
        character_type: CharacterType::GrowthItem,
        shape_id: 0,
        animation_id: 0,
        hitbox_width: 192,
        hitbox_height: 192,
        max_hp: 0,
        contact_damage: 0,
        player_damage: 0,
        score: 0,
        default_orbit_id: 0,
        default_fire_pattern_id: 0,
        bullet_character_id: 0,
        growth_effect_id: 1,
        destroy_effect_id: 0,
        destroy_sound_id: 0,
    },
];

pub const STAGE_ONE_SCHEDULE: &[ScheduleData] = &[
    ScheduleData {
        schedule_id: 1,
        stage_id: 1,
        frame: 0,
        spawn_x: Q12_4(1_280),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 2,
        orbit_id: 1,
        difficulty: 1,
        fire_pattern_id: 1,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 2,
        stage_id: 1,
        frame: 90,
        spawn_x: Q12_4(5_120),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 2,
        orbit_id: 2,
        difficulty: 1,
        fire_pattern_id: 1,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 3,
        stage_id: 1,
        frame: 900,
        spawn_x: Q12_4(2_560),
        spawn_y: Q12_4(1_920),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 1,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 4,
        stage_id: 1,
        frame: 1_800,
        spawn_x: Q12_4(7_680),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 2,
        orbit_id: 1,
        difficulty: 1,
        fire_pattern_id: 1,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 5,
        stage_id: 1,
        frame: 3_600,
        spawn_x: Q12_4(5_120),
        spawn_y: Q12_4(1_920),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 1,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 6,
        stage_id: 1,
        frame: 5_399,
        spawn_x: Q12_4(5_120),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Boss,
        character_id: 100,
        orbit_id: 3,
        difficulty: 4,
        fire_pattern_id: 10,
        background_speed: Q12_4(16),
    },
];

pub const ORBIT_DATA: &[OrbitData] = &[
    OrbitData {
        orbit_id: 1,
        orbit_type: OrbitType::Straight,
        direction: Direction16::South,
        speed: Q12_4(16),
        radius: Q12_4::ZERO,
        control_point_1: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_2: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_end: (Q12_4::ZERO, Q12_4::ZERO),
        start_angle: None,
        end_angle: None,
        duration_frames: 540,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 0,
        rotation: 0,
    },
    OrbitData {
        orbit_id: 2,
        orbit_type: OrbitType::Circle,
        direction: Direction16::East,
        speed: Q12_4(16),
        radius: Q12_4(768),
        control_point_1: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_2: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_end: (Q12_4::ZERO, Q12_4::ZERO),
        start_angle: Some(0),
        end_angle: Some(16_384),
        duration_frames: 180,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 1,
        rotation: 1,
    },
    OrbitData {
        orbit_id: 3,
        orbit_type: OrbitType::Bezier,
        direction: Direction16::South,
        speed: Q12_4(16),
        radius: Q12_4::ZERO,
        control_point_1: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_2: (Q12_4(1_600), Q12_4::ZERO),
        control_point_end: (Q12_4(5_120), Q12_4(2_560)),
        start_angle: None,
        end_angle: None,
        duration_frames: 900,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 0,
        rotation: 0,
    },
];

pub fn character_trait(character_id: u16) -> Option<&'static CharacterTrait> {
    CHARACTER_TRAITS
        .iter()
        .find(|trait_data| trait_data.character_id == character_id)
}

pub fn orbit(orbit_id: u16) -> Option<&'static OrbitData> {
    ORBIT_DATA.iter().find(|orbit| orbit.orbit_id == orbit_id)
}

pub fn validate_stage_one() -> Result<(), String> {
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
    for schedule in STAGE_ONE_SCHEDULE {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_one_contains_final_frame_boss() {
        let boss = STAGE_ONE_SCHEDULE
            .iter()
            .find(|entry| entry.object_type == ObjectType::Boss)
            .expect("stage one boss should exist");
        assert_eq!(boss.frame, STAGE_ONE.play_frames - 1);
        assert_eq!(boss.character_id, STAGE_ONE.boss_character_id);
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
        assert!(validate_stage_one().is_ok());
    }

    #[test]
    fn fire_pattern_direction_converts_to_velocity() {
        let pattern = fire_pattern(1).unwrap();
        assert_eq!(pattern.velocity(), (Q12_4::ZERO, Q12_4(16)));
    }
}
