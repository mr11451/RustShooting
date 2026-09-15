use crate::fixed::Q12_4;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerGrowthData {
    pub growth_level: u8,
    pub max_bullet_groups: u8,
    pub bullets_per_group: u8,
    pub bullet_character_id: u16,
    pub speed: Q12_4,
    pub directions: &'static [Direction16],
}

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

pub const PLAYER_GROWTH_DATA: &[PlayerGrowthData] = &[
    PlayerGrowthData {
        growth_level: 0,
        max_bullet_groups: 4,
        bullets_per_group: 1,
        bullet_character_id: 1,
        speed: Q12_4(64),
        directions: PLAYER_DIRECTIONS_LEVEL_0,
    },
    PlayerGrowthData {
        growth_level: 1,
        max_bullet_groups: 6,
        bullets_per_group: 2,
        bullet_character_id: 1,
        speed: Q12_4(67),
        directions: PLAYER_DIRECTIONS_LEVEL_1,
    },
    PlayerGrowthData {
        growth_level: 2,
        max_bullet_groups: 8,
        bullets_per_group: 3,
        bullet_character_id: 1,
        speed: Q12_4(70),
        directions: PLAYER_DIRECTIONS_LEVEL_2,
    },
    PlayerGrowthData {
        growth_level: 3,
        max_bullet_groups: 10,
        bullets_per_group: 4,
        bullet_character_id: 1,
        speed: Q12_4(74),
        directions: PLAYER_DIRECTIONS_LEVEL_3,
    },
    PlayerGrowthData {
        growth_level: 4,
        max_bullet_groups: 12,
        bullets_per_group: 5,
        bullet_character_id: 1,
        speed: Q12_4(77),
        directions: PLAYER_DIRECTIONS_LEVEL_4,
    },
];

pub fn player_growth_data(growth_level: u8) -> &'static PlayerGrowthData {
    &PLAYER_GROWTH_DATA[usize::from(growth_level.min(4))]
}

pub fn direction_velocity(direction: Direction16, speed: Q12_4) -> (Q12_4, Q12_4) {
    let index = direction_index(direction);
    let angle = (index as f32) * std::f32::consts::TAU / 32.0;
    let speed = f32::from(speed.raw());
    (
        Q12_4((angle.sin() * speed).round() as i16),
        Q12_4((-angle.cos() * speed).round() as i16),
    )
}

fn direction_index(direction: Direction16) -> u8 {
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
        hitbox_width: 16,
        hitbox_height: 16,
        damage: 2,
        penetrating: false,
        player_damage: 20,
    },
    BulletCharacterData {
        bullet_character_id: 4,
        fire_sound_id: 1,
        hit_sound_id: 2,
        hitbox_width: 8,
        hitbox_height: 16,
        damage: 2,
        penetrating: true,
        player_damage: 10,
    },
    BulletCharacterData {
        bullet_character_id: 5,
        fire_sound_id: 1,
        hit_sound_id: 2,
        hitbox_width: 8,
        hitbox_height: 8,
        damage: 1,
        penetrating: false,
        player_damage: 15,
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
        speed: Q12_4(20),
        bullet_velocity_x: Q12_4::ZERO,
        bullet_velocity_y: Q12_4(20),
    },
    FirePatternData {
        fire_pattern_id: 2,
        fire_frame: 30,
        spawn_offset_x: Q12_4::ZERO,
        spawn_offset_y: Q12_4::ZERO,
        bullet_character_id: 2,
        direction: Direction16::SouthEast,
        speed: Q12_4(20),
        bullet_velocity_x: Q12_4(14),
        bullet_velocity_y: Q12_4(14),
    },
    FirePatternData {
        fire_pattern_id: 3,
        fire_frame: 30,
        spawn_offset_x: Q12_4::ZERO,
        spawn_offset_y: Q12_4::ZERO,
        bullet_character_id: 2,
        direction: Direction16::SouthWest,
        speed: Q12_4(20),
        bullet_velocity_x: Q12_4(-14),
        bullet_velocity_y: Q12_4(14),
    },
    FirePatternData {
        fire_pattern_id: 4,
        fire_frame: 25,
        spawn_offset_x: Q12_4::ZERO,
        spawn_offset_y: Q12_4::ZERO,
        bullet_character_id: 5,
        direction: Direction16::South,
        speed: Q12_4(30),
        bullet_velocity_x: Q12_4::ZERO,
        bullet_velocity_y: Q12_4(30),
    },
    FirePatternData {
        fire_pattern_id: 10,
        fire_frame: 60,
        spawn_offset_x: Q12_4::ZERO,
        spawn_offset_y: Q12_4(256),
        bullet_character_id: 3,
        direction: Direction16::South,
        speed: Q12_4(20),
        bullet_velocity_x: Q12_4::ZERO,
        bullet_velocity_y: Q12_4(20),
    },
    FirePatternData {
        fire_pattern_id: 11,
        fire_frame: 45,
        spawn_offset_x: Q12_4(128),
        spawn_offset_y: Q12_4(256),
        bullet_character_id: 3,
        direction: Direction16::SouthEast,
        speed: Q12_4(20),
        bullet_velocity_x: Q12_4(14),
        bullet_velocity_y: Q12_4(14),
    },
    FirePatternData {
        fire_pattern_id: 12,
        fire_frame: 35,
        spawn_offset_x: Q12_4(-128),
        spawn_offset_y: Q12_4(256),
        bullet_character_id: 5,
        direction: Direction16::SouthWest,
        speed: Q12_4(25),
        bullet_velocity_x: Q12_4(-18),
        bullet_velocity_y: Q12_4(18),
    },
    FirePatternData {
        fire_pattern_id: 13,
        fire_frame: 40,
        spawn_offset_x: Q12_4::ZERO,
        spawn_offset_y: Q12_4(384),
        bullet_character_id: 3,
        direction: Direction16::South,
        speed: Q12_4(25),
        bullet_velocity_x: Q12_4::ZERO,
        bullet_velocity_y: Q12_4(25),
    },
    FirePatternData {
        fire_pattern_id: 14,
        fire_frame: 30,
        spawn_offset_x: Q12_4::ZERO,
        spawn_offset_y: Q12_4(384),
        bullet_character_id: 5,
        direction: Direction16::South,
        speed: Q12_4(30),
        bullet_velocity_x: Q12_4::ZERO,
        bullet_velocity_y: Q12_4(30),
    },
    FirePatternData {
        fire_pattern_id: 15,
        fire_frame: 20,
        spawn_offset_x: Q12_4::ZERO,
        spawn_offset_y: Q12_4(512),
        bullet_character_id: 3,
        direction: Direction16::South,
        speed: Q12_4(30),
        bullet_velocity_x: Q12_4::ZERO,
        bullet_velocity_y: Q12_4(30),
    },
];

pub fn fire_pattern(fire_pattern_id: u16) -> Option<&'static FirePatternData> {
    FIRE_PATTERNS
        .iter()
        .find(|pattern| pattern.fire_pattern_id == fire_pattern_id)
}

#[allow(dead_code)]
pub const STAGES: &[StageData] = &[
    StageData {
        stage_id: 1,
        play_frames: 5_400,
        intro_frames: 90,
        clear_frames: 90,
        boss_character_id: 100,
        schedule_start_id: 1,
        item_count: 4,
    },
    StageData {
        stage_id: 2,
        play_frames: 5_400,
        intro_frames: 90,
        clear_frames: 90,
        boss_character_id: 101,
        schedule_start_id: 101,
        item_count: 4,
    },
    StageData {
        stage_id: 3,
        play_frames: 5_400,
        intro_frames: 90,
        clear_frames: 90,
        boss_character_id: 102,
        schedule_start_id: 201,
        item_count: 4,
    },
    StageData {
        stage_id: 4,
        play_frames: 5_400,
        intro_frames: 90,
        clear_frames: 90,
        boss_character_id: 103,
        schedule_start_id: 301,
        item_count: 4,
    },
    StageData {
        stage_id: 5,
        play_frames: 5_400,
        intro_frames: 90,
        clear_frames: 90,
        boss_character_id: 104,
        schedule_start_id: 401,
        item_count: 4,
    },
    StageData {
        stage_id: 6,
        play_frames: 5_400,
        intro_frames: 90,
        clear_frames: 90,
        boss_character_id: 105,
        schedule_start_id: 501,
        item_count: 4,
    },
];

#[allow(dead_code)]
pub fn stage_data(stage_id: u8) -> Option<&'static StageData> {
    STAGES.iter().find(|stage| stage.stage_id == stage_id)
}

#[allow(dead_code)]
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
        character_id: 3,
        character_type: CharacterType::Enemy,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 256,
        hitbox_height: 256,
        max_hp: 15,
        contact_damage: 15,
        player_damage: 10,
        score: 150,
        default_orbit_id: 4,
        default_fire_pattern_id: 2,
        bullet_character_id: 2,
        growth_effect_id: 0,
        destroy_effect_id: 1,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 4,
        character_type: CharacterType::Enemy,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 384,
        hitbox_height: 384,
        max_hp: 30,
        contact_damage: 20,
        player_damage: 15,
        score: 300,
        default_orbit_id: 1,
        default_fire_pattern_id: 4,
        bullet_character_id: 5,
        growth_effect_id: 0,
        destroy_effect_id: 1,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 5,
        character_type: CharacterType::Enemy,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 320,
        hitbox_height: 320,
        max_hp: 25,
        contact_damage: 15,
        player_damage: 10,
        score: 250,
        default_orbit_id: 6,
        default_fire_pattern_id: 3,
        bullet_character_id: 2,
        growth_effect_id: 0,
        destroy_effect_id: 1,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 6,
        character_type: CharacterType::Enemy,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 640,
        hitbox_height: 512,
        max_hp: 200,
        contact_damage: 30,
        player_damage: 20,
        score: 2_000,
        default_orbit_id: 7,
        default_fire_pattern_id: 10,
        bullet_character_id: 3,
        growth_effect_id: 0,
        destroy_effect_id: 2,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 100,
        character_type: CharacterType::Boss,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 1024,
        hitbox_height: 640,
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
        character_id: 101,
        character_type: CharacterType::Boss,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 1152,
        hitbox_height: 720,
        max_hp: 1_200,
        contact_damage: 50,
        player_damage: 50,
        score: 15_000,
        default_orbit_id: 3,
        default_fire_pattern_id: 11,
        bullet_character_id: 3,
        growth_effect_id: 0,
        destroy_effect_id: 2,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 102,
        character_type: CharacterType::Boss,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 1280,
        hitbox_height: 768,
        max_hp: 1_500,
        contact_damage: 50,
        player_damage: 50,
        score: 20_000,
        default_orbit_id: 3,
        default_fire_pattern_id: 12,
        bullet_character_id: 5,
        growth_effect_id: 0,
        destroy_effect_id: 2,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 103,
        character_type: CharacterType::Boss,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 1280,
        hitbox_height: 800,
        max_hp: 1_800,
        contact_damage: 50,
        player_damage: 50,
        score: 25_000,
        default_orbit_id: 3,
        default_fire_pattern_id: 13,
        bullet_character_id: 3,
        growth_effect_id: 0,
        destroy_effect_id: 2,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 104,
        character_type: CharacterType::Boss,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 1408,
        hitbox_height: 880,
        max_hp: 2_200,
        contact_damage: 50,
        player_damage: 50,
        score: 30_000,
        default_orbit_id: 3,
        default_fire_pattern_id: 14,
        bullet_character_id: 5,
        growth_effect_id: 0,
        destroy_effect_id: 2,
        destroy_sound_id: 3,
    },
    CharacterTrait {
        character_id: 105,
        character_type: CharacterType::Boss,
        shape_id: 1,
        animation_id: 0,
        hitbox_width: 1536,
        hitbox_height: 960,
        max_hp: 3_000,
        contact_damage: 60,
        player_damage: 60,
        score: 50_000,
        default_orbit_id: 3,
        default_fire_pattern_id: 15,
        bullet_character_id: 3,
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

pub const STAGE_SCHEDULES: &[ScheduleData] = &[
    // Stage 1
    ScheduleData {
        schedule_id: 1,
        stage_id: 1,
        frame: 0,
        spawn_x: Q12_4(1_920),
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
        spawn_x: Q12_4(3_840),
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
        spawn_x: Q12_4(1_920),
        spawn_y: Q12_4(3_840),
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
        spawn_x: Q12_4(5_760),
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
        frame: 2_700,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(5_000),
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
        frame: 3_600,
        spawn_x: Q12_4(2_400),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 1,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 7,
        stage_id: 1,
        frame: 4_500,
        spawn_x: Q12_4(4_800),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 1,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 8,
        stage_id: 1,
        frame: 5_399,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Boss,
        character_id: 100,
        orbit_id: 8,
        difficulty: 4,
        fire_pattern_id: 10,
        background_speed: Q12_4(16),
    },
    // Stage 2
    ScheduleData {
        schedule_id: 101,
        stage_id: 2,
        frame: 0,
        spawn_x: Q12_4(1_500),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 3,
        orbit_id: 4,
        difficulty: 2,
        fire_pattern_id: 2,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 102,
        stage_id: 2,
        frame: 120,
        spawn_x: Q12_4(6_000),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 3,
        orbit_id: 5,
        difficulty: 2,
        fire_pattern_id: 3,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 103,
        stage_id: 2,
        frame: 900,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(3_840),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 2,
        fire_pattern_id: 0,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 104,
        stage_id: 2,
        frame: 1_800,
        spawn_x: Q12_4(3_000),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 4,
        orbit_id: 1,
        difficulty: 2,
        fire_pattern_id: 4,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 105,
        stage_id: 2,
        frame: 2_700,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(5_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 2,
        fire_pattern_id: 0,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 106,
        stage_id: 2,
        frame: 3_600,
        spawn_x: Q12_4(2_000),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 2,
        fire_pattern_id: 0,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 107,
        stage_id: 2,
        frame: 4_500,
        spawn_x: Q12_4(5_000),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 2,
        fire_pattern_id: 0,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 108,
        stage_id: 2,
        frame: 5_399,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Boss,
        character_id: 101,
        orbit_id: 8,
        difficulty: 4,
        fire_pattern_id: 11,
        background_speed: Q12_4(24),
    },
    // Stage 3
    ScheduleData {
        schedule_id: 201,
        stage_id: 3,
        frame: 0,
        spawn_x: Q12_4(2_400),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 5,
        orbit_id: 6,
        difficulty: 3,
        fire_pattern_id: 3,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 202,
        stage_id: 3,
        frame: 900,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(3_840),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 3,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 203,
        stage_id: 3,
        frame: 1_800,
        spawn_x: Q12_4(5_200),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 5,
        orbit_id: 6,
        difficulty: 3,
        fire_pattern_id: 3,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 204,
        stage_id: 3,
        frame: 2_700,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(5_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 3,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 205,
        stage_id: 3,
        frame: 3_600,
        spawn_x: Q12_4(2_000),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 3,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 206,
        stage_id: 3,
        frame: 4_500,
        spawn_x: Q12_4(5_500),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 3,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 207,
        stage_id: 3,
        frame: 5_399,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Boss,
        character_id: 102,
        orbit_id: 8,
        difficulty: 4,
        fire_pattern_id: 12,
        background_speed: Q12_4(16),
    },
    // Stage 4
    ScheduleData {
        schedule_id: 301,
        stage_id: 4,
        frame: 0,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 4,
        orbit_id: 1,
        difficulty: 3,
        fire_pattern_id: 4,
        background_speed: Q12_4(32),
    },
    ScheduleData {
        schedule_id: 302,
        stage_id: 4,
        frame: 900,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(3_840),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 3,
        fire_pattern_id: 0,
        background_speed: Q12_4(32),
    },
    ScheduleData {
        schedule_id: 303,
        stage_id: 4,
        frame: 1_800,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Enemy,
        character_id: 6,
        orbit_id: 7,
        difficulty: 3,
        fire_pattern_id: 10,
        background_speed: Q12_4(32),
    },
    ScheduleData {
        schedule_id: 304,
        stage_id: 4,
        frame: 2_700,
        spawn_x: Q12_4(2_000),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 3,
        fire_pattern_id: 0,
        background_speed: Q12_4(32),
    },
    ScheduleData {
        schedule_id: 305,
        stage_id: 4,
        frame: 3_600,
        spawn_x: Q12_4(5_500),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 3,
        fire_pattern_id: 0,
        background_speed: Q12_4(32),
    },
    ScheduleData {
        schedule_id: 306,
        stage_id: 4,
        frame: 4_500,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(5_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 3,
        fire_pattern_id: 0,
        background_speed: Q12_4(32),
    },
    ScheduleData {
        schedule_id: 307,
        stage_id: 4,
        frame: 5_399,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Boss,
        character_id: 103,
        orbit_id: 8,
        difficulty: 4,
        fire_pattern_id: 13,
        background_speed: Q12_4(32),
    },
    // Stage 5
    ScheduleData {
        schedule_id: 401,
        stage_id: 5,
        frame: 0,
        spawn_x: Q12_4(2_000),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 5,
        orbit_id: 6,
        difficulty: 4,
        fire_pattern_id: 3,
        background_speed: Q12_4(20),
    },
    ScheduleData {
        schedule_id: 402,
        stage_id: 5,
        frame: 900,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(3_840),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 4,
        fire_pattern_id: 0,
        background_speed: Q12_4(20),
    },
    ScheduleData {
        schedule_id: 403,
        stage_id: 5,
        frame: 1_800,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Enemy,
        character_id: 6,
        orbit_id: 7,
        difficulty: 4,
        fire_pattern_id: 10,
        background_speed: Q12_4(20),
    },
    ScheduleData {
        schedule_id: 404,
        stage_id: 5,
        frame: 2_700,
        spawn_x: Q12_4(2_000),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 4,
        fire_pattern_id: 0,
        background_speed: Q12_4(20),
    },
    ScheduleData {
        schedule_id: 405,
        stage_id: 5,
        frame: 3_600,
        spawn_x: Q12_4(5_500),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 4,
        fire_pattern_id: 0,
        background_speed: Q12_4(20),
    },
    ScheduleData {
        schedule_id: 406,
        stage_id: 5,
        frame: 4_500,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(5_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 4,
        fire_pattern_id: 0,
        background_speed: Q12_4(20),
    },
    ScheduleData {
        schedule_id: 407,
        stage_id: 5,
        frame: 5_399,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Boss,
        character_id: 104,
        orbit_id: 8,
        difficulty: 4,
        fire_pattern_id: 14,
        background_speed: Q12_4(20),
    },
    // Stage 6
    ScheduleData {
        schedule_id: 501,
        stage_id: 6,
        frame: 0,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Enemy,
        character_id: 6,
        orbit_id: 7,
        difficulty: 4,
        fire_pattern_id: 10,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 502,
        stage_id: 6,
        frame: 900,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(3_840),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 4,
        fire_pattern_id: 0,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 503,
        stage_id: 6,
        frame: 1_800,
        spawn_x: Q12_4(2_000),
        spawn_y: Q12_4(-384),
        object_type: ObjectType::Enemy,
        character_id: 4,
        orbit_id: 1,
        difficulty: 4,
        fire_pattern_id: 4,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 504,
        stage_id: 6,
        frame: 2_700,
        spawn_x: Q12_4(2_000),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 4,
        fire_pattern_id: 0,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 505,
        stage_id: 6,
        frame: 3_600,
        spawn_x: Q12_4(5_500),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 4,
        fire_pattern_id: 0,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 506,
        stage_id: 6,
        frame: 4_500,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(5_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 4,
        fire_pattern_id: 0,
        background_speed: Q12_4(24),
    },
    ScheduleData {
        schedule_id: 507,
        stage_id: 6,
        frame: 5_399,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Boss,
        character_id: 105,
        orbit_id: 10,
        difficulty: 4,
        fire_pattern_id: 15,
        background_speed: Q12_4(24),
    },
];

#[allow(dead_code)]
pub const STAGE_ONE_SCHEDULE: &[ScheduleData] = &[
    ScheduleData {
        schedule_id: 1,
        stage_id: 1,
        frame: 0,
        spawn_x: Q12_4(1_920),
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
        spawn_x: Q12_4(3_840),
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
        spawn_x: Q12_4(1_920),
        spawn_y: Q12_4(3_840),
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
        spawn_x: Q12_4(5_760),
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
        frame: 2_700,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(5_000),
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
        frame: 3_600,
        spawn_x: Q12_4(2_400),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 1,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 7,
        stage_id: 1,
        frame: 4_500,
        spawn_x: Q12_4(4_800),
        spawn_y: Q12_4(4_000),
        object_type: ObjectType::GrowthItem,
        character_id: 200,
        orbit_id: 0,
        difficulty: 1,
        fire_pattern_id: 0,
        background_speed: Q12_4(16),
    },
    ScheduleData {
        schedule_id: 8,
        stage_id: 1,
        frame: 5_399,
        spawn_x: Q12_4(3_840),
        spawn_y: Q12_4(-512),
        object_type: ObjectType::Boss,
        character_id: 100,
        orbit_id: 8,
        difficulty: 4,
        fire_pattern_id: 10,
        background_speed: Q12_4(16),
    },
];

pub fn stage_schedule(stage_id: u8) -> impl Iterator<Item = &'static ScheduleData> {
    STAGE_SCHEDULES
        .iter()
        .filter(move |schedule| schedule.stage_id == stage_id)
}

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
        target_position: (Q12_4::ZERO, Q12_4::ZERO),
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
        speed: Q12_4::ONE,
        radius: Q12_4(768),
        control_point_1: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_2: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_end: (Q12_4::ZERO, Q12_4::ZERO),
        target_position: (Q12_4::ZERO, Q12_4::ZERO),
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
        control_point_2: (Q12_4::ZERO, Q12_4(1_600)),
        control_point_end: (Q12_4::ZERO, Q12_4(3_000)),
        target_position: (Q12_4::ZERO, Q12_4::ZERO),
        start_angle: None,
        end_angle: None,
        duration_frames: 900,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 0,
        rotation: 0,
    },
    OrbitData {
        orbit_id: 4,
        orbit_type: OrbitType::Straight,
        direction: Direction16::SouthEast,
        speed: Q12_4(20),
        radius: Q12_4::ZERO,
        control_point_1: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_2: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_end: (Q12_4::ZERO, Q12_4::ZERO),
        target_position: (Q12_4::ZERO, Q12_4::ZERO),
        start_angle: None,
        end_angle: None,
        duration_frames: 400,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 0,
        rotation: 0,
    },
    OrbitData {
        orbit_id: 5,
        orbit_type: OrbitType::Straight,
        direction: Direction16::SouthWest,
        speed: Q12_4(20),
        radius: Q12_4::ZERO,
        control_point_1: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_2: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_end: (Q12_4::ZERO, Q12_4::ZERO),
        target_position: (Q12_4::ZERO, Q12_4::ZERO),
        start_angle: None,
        end_angle: None,
        duration_frames: 400,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 0,
        rotation: 0,
    },
    OrbitData {
        orbit_id: 6,
        orbit_type: OrbitType::Bezier,
        direction: Direction16::South,
        speed: Q12_4(16),
        radius: Q12_4::ZERO,
        control_point_1: (Q12_4(-2_000), Q12_4(1_000)),
        control_point_2: (Q12_4(2_000), Q12_4(2_500)),
        control_point_end: (Q12_4(0), Q12_4(5_000)),
        target_position: (Q12_4::ZERO, Q12_4::ZERO),
        start_angle: None,
        end_angle: None,
        duration_frames: 600,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 0,
        rotation: 0,
    },
    OrbitData {
        orbit_id: 7,
        orbit_type: OrbitType::Bezier,
        direction: Direction16::South,
        speed: Q12_4(16),
        radius: Q12_4::ZERO,
        control_point_1: (Q12_4(0), Q12_4(2_000)),
        control_point_2: (Q12_4(0), Q12_4(2_000)),
        control_point_end: (Q12_4(0), Q12_4(1_000)),
        start_angle: None,
        target_position: (Q12_4::ZERO, Q12_4::ZERO),
        end_angle: None,
        duration_frames: 600,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 0,
        rotation: 0,
    },
    OrbitData {
        orbit_id: 8,
        orbit_type: OrbitType::MoveToPosition,
        direction: Direction16::South,
        speed: Q12_4::ZERO,
        radius: Q12_4::ZERO,
        control_point_1: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_2: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_end: (Q12_4::ZERO, Q12_4::ZERO),
        target_position: (BOSS_ORBIT_CENTER_X_Q12, BOSS_ORBIT_CENTER_Y_Q12),
        start_angle: None,
        end_angle: None,
        duration_frames: 90,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 9,
        rotation: 0,
    },
    OrbitData {
        orbit_id: 9,
        orbit_type: OrbitType::Circle,
        direction: Direction16::East,
        speed: Q12_4::ONE,
        radius: BOSS_ORBIT_RADIUS_Q12,
        control_point_1: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_2: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_end: (Q12_4::ZERO, Q12_4::ZERO),
        target_position: (Q12_4::ZERO, Q12_4::ZERO),
        start_angle: Some(0),
        end_angle: None,
        duration_frames: u32::MAX,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 9,
        rotation: 1,
    },
    OrbitData {
        orbit_id: 10,
        orbit_type: OrbitType::Bezier,
        direction: Direction16::South,
        speed: Q12_4(16),
        radius: Q12_4::ZERO,
        control_point_1: (Q12_4::ZERO, Q12_4::ZERO),
        control_point_2: (Q12_4::ZERO, Q12_4(1_600)),
        control_point_end: (Q12_4::ZERO, Q12_4(3_000)),
        target_position: (Q12_4::ZERO, Q12_4::ZERO),
        start_angle: None,
        end_angle: None,
        duration_frames: 900,
        acceleration: Q12_4::ZERO,
        next_orbit_id: 8,
        rotation: 0,
    },
];

pub fn character_trait(character_id: u16) -> Option<&'static CharacterTrait> {
    CHARACTER_TRAITS
        .iter()
        .find(|trait_data| trait_data.character_id == character_id)
}

#[allow(dead_code)]
pub fn orbit(orbit_id: u16) -> Option<&'static OrbitData> {
    ORBIT_DATA.iter().find(|orbit| orbit.orbit_id == orbit_id)
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
    for schedule in STAGE_SCHEDULES {
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
        let level_zero = player_growth_data(0);
        let level_four = player_growth_data(4);

        assert_eq!(level_zero.max_bullet_groups, 4);
        assert_eq!(level_zero.bullets_per_group, 1);
        assert_eq!(level_zero.bullet_character_id, 1);
        assert_eq!(level_four.max_bullet_groups, 12);
        assert_eq!(level_four.bullets_per_group, 5);
        assert_eq!(level_four.bullet_character_id, 1);
        assert_eq!(level_four.speed, Q12_4(77));
        assert_eq!(level_four.directions.len(), 5);
        assert_eq!(level_four.directions[2], Direction16::North);
    }

    #[test]
    fn fire_pattern_direction_converts_to_velocity() {
        let pattern = fire_pattern(1).unwrap();
        assert_eq!(pattern.velocity(), (Q12_4::ZERO, Q12_4(20)));
    }

    #[test]
    fn direction_velocity_supports_thirty_two_directions() {
        let (x, y) = direction_velocity(Direction16::NorthByEast, Q12_4(64));
        assert!(x.raw() > 0);
        assert!(y.raw() < 0);
    }
}
