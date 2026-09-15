// Stage metadata and schedule aggregation.
use super::*;

#[path = "stage01.rs"]
pub mod stage01;
#[path = "stage02.rs"]
pub mod stage02;
#[path = "stage03.rs"]
pub mod stage03;
#[path = "stage04.rs"]
pub mod stage04;
#[path = "stage05.rs"]
pub mod stage05;
#[path = "stage06.rs"]
pub mod stage06;

pub use stage01::STAGE_01_SCHEDULE;
pub use stage02::STAGE_02_SCHEDULE;
pub use stage03::STAGE_03_SCHEDULE;
pub use stage04::STAGE_04_SCHEDULE;
pub use stage05::STAGE_05_SCHEDULE;
pub use stage06::STAGE_06_SCHEDULE;

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
pub const STAGE_ONE: StageData = STAGES[0];

pub fn all_schedules() -> impl Iterator<Item = &'static ScheduleData> {
    STAGE_01_SCHEDULE
        .iter()
        .chain(STAGE_02_SCHEDULE)
        .chain(STAGE_03_SCHEDULE)
        .chain(STAGE_04_SCHEDULE)
        .chain(STAGE_05_SCHEDULE)
        .chain(STAGE_06_SCHEDULE)
}

#[allow(dead_code)]
pub const STAGE_ONE_SCHEDULE: &[ScheduleData] = STAGE_01_SCHEDULE;

pub fn stage_schedule(stage_id: u8) -> impl Iterator<Item = &'static ScheduleData> {
    match stage_id {
        1 => STAGE_01_SCHEDULE.iter(),
        2 => STAGE_02_SCHEDULE.iter(),
        3 => STAGE_03_SCHEDULE.iter(),
        4 => STAGE_04_SCHEDULE.iter(),
        5 => STAGE_05_SCHEDULE.iter(),
        6 => STAGE_06_SCHEDULE.iter(),
        _ => [].iter(),
    }
}

pub fn stage_enemy_character_ids(stage_id: u8) -> Vec<u16> {
    let mut character_ids = Vec::new();
    for schedule in stage_schedule(stage_id) {
        if schedule.object_type == ObjectType::Enemy
            && !character_ids.contains(&schedule.character_id)
        {
            character_ids.push(schedule.character_id);
        }
    }
    character_ids
}

#[allow(dead_code)]
pub fn scheduled_enemy_bullet_character_ids() -> Vec<u16> {
    let mut bullet_ids = Vec::new();
    for schedule in all_schedules() {
        if matches!(schedule.object_type, ObjectType::Enemy | ObjectType::Boss)
            && schedule.fire_pattern_id != 0
            && let Some(trait_data) = character_trait(schedule.character_id)
            && !bullet_ids.contains(&trait_data.bullet_character_id)
        {
            bullet_ids.push(trait_data.bullet_character_id);
        }
    }
    bullet_ids.sort_unstable();
    bullet_ids
}
