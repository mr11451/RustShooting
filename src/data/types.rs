use crate::fixed::Q12_4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharacterTrait {
    pub character_id: u16,
    pub character_type: super::CharacterType,
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
    pub frame: u32,
    pub spawn_x: Q12_4,
    pub spawn_y: Q12_4,
    pub object_type: super::ObjectType,
    pub character_id: u16,
    pub orbit_id: u16,
    pub difficulty: u8,
    pub fire_pattern_id: u16,
    pub background_speed: Q12_4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FireAngleMode {
    Fixed,
    AimAtPlayer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirePatternData {
    pub fire_pattern_id: u16,
    pub fire_frame: u32,
    pub spawn_offset_x: Q12_4,
    pub spawn_offset_y: Q12_4,
    pub bullet_character_id: u16,
    pub direction: super::Direction16,
    pub speed: Q12_4,
    pub bullet_velocity_x: Q12_4,
    pub bullet_velocity_y: Q12_4,
    pub angle_mode: FireAngleMode,
    pub volley_count: u8,
    pub direction_step: i8,
    pub repeat_interval_frames: u32,
    pub volley_interval_frames: u32,
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
    pub homing: bool,
    pub player_damage: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerBulletImageData {
    pub bullet_character_id: u16,
    pub image_file_name: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerGrowthData {
    pub growth_level: u8,
    pub max_bullet_groups: u8,
    pub bullets_per_group: u8,
    pub bullet_character_id: u16,
    pub speed: Q12_4,
    pub visual_width: u16,
    pub visual_height: u16,
    pub hitbox_half_width: Q12_4,
    pub hitbox_half_height: Q12_4,
    pub move_speed: Q12_4,
    pub directions: &'static [super::Direction16],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnemyVisualData {
    pub character_id: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StageEnemyImageData {
    pub stage_id: u8,
    pub character_id: u16,
    pub image_file_name: &'static str,
}
