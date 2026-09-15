use crate::{
    audio::AudioEvent,
    collision::{Hitbox, overlaps},
    data::{
        CharacterType, ORBIT_DATA, ObjectType, OrbitType, PLAYER_BULLET_SPEED_BY_LEVEL_Q12,
        PLAYER_INITIAL_X_Q12, PLAYER_INITIAL_Y_Q12, SCREEN_HEIGHT_Q12, SCREEN_WIDTH_Q12,
        ScheduleData, bullet_character, character_trait, fire_pattern, stage_schedule,
    },
    runtime::{
        ENEMY_BULLET_CAPACITY, ENEMY_CAPACITY, EffectPool, EffectState, ITEM_CAPACITY, ObjectPool,
        PLAYER_BULLET_CAPACITY,
    },
};

use super::state::GameState;

const PLAYER_MAX_HP: u16 = 100;
const RECOVERY_PER_STOCK: u16 = 25;
const PLAYER_GROUP_FIRE_INTERVAL_FRAMES: u16 = 30;

#[derive(Clone, Copy)]
struct CollisionSnapshot {
    index: usize,
    character_id: u16,
    x: crate::fixed::Q12_4,
    y: crate::fixed::Q12_4,
}

#[derive(Debug)]
pub struct World {
    pub state: GameState,
    pub frame: u32,
    pub stage_id: u8,
    pub hp: u16,
    pub score: u32,
    pub lives: u8,
    pub enemies: ObjectPool<ENEMY_CAPACITY>,
    pub player_bullets: ObjectPool<PLAYER_BULLET_CAPACITY>,
    pub enemy_bullets: ObjectPool<ENEMY_BULLET_CAPACITY>,
    pub items: ObjectPool<ITEM_CAPACITY>,
    pub effects: EffectPool,
    pub next_player_bullet_group_id: u32,
    pub player_group_fire_cooldown: u16,
    pub audio_events: Vec<AudioEvent>,
    pub background_scroll: crate::fixed::Q12_4,
    pub background_speed: crate::fixed::Q12_4,
    pub player_x: crate::fixed::Q12_4,
    pub player_y: crate::fixed::Q12_4,
    pub growth_level: u8,
    pub recovery_stock: u8,
    pub invincible_frames: u16,
    pub ranking: super::RankingTable,
    pub name_entry: super::NameEntryState,
}

impl Default for World {
    fn default() -> Self {
        Self {
            state: GameState::Title,
            frame: 0,
            stage_id: 1,
            hp: 100,
            score: 0,
            lives: 3,
            enemies: ObjectPool::default(),
            player_bullets: ObjectPool::default(),
            enemy_bullets: ObjectPool::default(),
            items: ObjectPool::default(),
            effects: EffectPool::default(),
            next_player_bullet_group_id: 1,
            player_group_fire_cooldown: 0,
            audio_events: Vec::new(),
            background_scroll: crate::fixed::Q12_4::ZERO,
            background_speed: crate::fixed::Q12_4(16),
            player_x: PLAYER_INITIAL_X_Q12,
            player_y: PLAYER_INITIAL_Y_Q12,
            growth_level: 0,
            recovery_stock: 0,
            invincible_frames: 0,
            ranking: super::RankingTable::load_default(),
            name_entry: super::NameEntryState::default(),
        }
    }
}

impl World {
    pub fn begin_name_entry(&mut self) {
        self.frame = 0;
        self.state = GameState::NameEntry;
        self.name_entry.reset();
    }

    fn character_hitbox(
        character_id: u16,
        x: crate::fixed::Q12_4,
        y: crate::fixed::Q12_4,
    ) -> Option<Hitbox> {
        let trait_data = character_trait(character_id)?;
        Some(Hitbox {
            x,
            y,
            half_width: crate::fixed::Q12_4(trait_data.hitbox_width),
            half_height: crate::fixed::Q12_4(trait_data.hitbox_height),
        })
    }

    fn bullet_hitbox(
        character_id: u16,
        x: crate::fixed::Q12_4,
        y: crate::fixed::Q12_4,
    ) -> Option<Hitbox> {
        let bullet_data = bullet_character(character_id)?;
        Some(Hitbox {
            x,
            y,
            half_width: crate::fixed::Q12_4(bullet_data.hitbox_width),
            half_height: crate::fixed::Q12_4(bullet_data.hitbox_height),
        })
    }

    fn active_player_bullet_group_count(&self) -> usize {
        let mut group_ids = Vec::new();
        self.player_bullets.for_each_active(|bullet| {
            if !group_ids.contains(&bullet.group_id) {
                group_ids.push(bullet.group_id);
            }
        });
        group_ids.len()
    }

    fn recover_hp(&mut self) {
        self.hp = self
            .hp
            .saturating_add(RECOVERY_PER_STOCK)
            .min(PLAYER_MAX_HP);
    }

    pub fn reset_stage(&mut self) {
        self.frame = 0;
        self.hp = PLAYER_MAX_HP;
        self.enemies.clear();
        self.player_bullets.clear();
        self.enemy_bullets.clear();
        self.items.clear();
        self.effects.clear();
        self.next_player_bullet_group_id = 1;
        self.player_group_fire_cooldown = 0;
        self.audio_events.clear();
        self.background_scroll = crate::fixed::Q12_4::ZERO;
        self.player_x = PLAYER_INITIAL_X_Q12;
        self.player_y = PLAYER_INITIAL_Y_Q12;
        self.growth_level = 0;
        self.invincible_frames = 0;
    }

    pub fn advance_to_next_stage(&mut self) {
        self.frame = 0;
        self.hp = PLAYER_MAX_HP;
        self.enemies.clear();
        self.player_bullets.clear();
        self.enemy_bullets.clear();
        self.items.clear();
        self.effects.clear();
        self.next_player_bullet_group_id = 1;
        self.player_group_fire_cooldown = 0;
        self.audio_events.clear();
        self.background_scroll = crate::fixed::Q12_4::ZERO;
        self.player_x = PLAYER_INITIAL_X_Q12;
        self.player_y = PLAYER_INITIAL_Y_Q12;
        self.invincible_frames = 0;
    }

    pub fn spawn_scheduled_objects(&mut self) {
        let scheduled_objects: Vec<ScheduleData> = stage_schedule(self.stage_id)
            .filter(|entry| entry.frame == self.frame)
            .copied()
            .collect();
        for schedule in scheduled_objects {
            if schedule.background_speed != crate::fixed::Q12_4::ZERO {
                self.background_speed = schedule.background_speed;
            }
            self.spawn_schedule_entry(&schedule);
        }
    }

    pub fn update_enemy_movement(&mut self) {
        self.enemies.for_each_active_mut(|enemy| {
            let orbit_opt = ORBIT_DATA
                .iter()
                .find(|orbit| orbit.orbit_id == enemy.orbit_id);
            match orbit_opt {
                Some(orbit) if enemy.orbit_frame > orbit.duration_frames => {
                    if orbit.next_orbit_id != 0 {
                        let preserve_origin = orbit.next_orbit_id == enemy.orbit_id;
                        enemy.orbit_id = orbit.next_orbit_id;
                        enemy.orbit_frame = 0;
                        if !preserve_origin {
                            enemy.orbit_origin_x = enemy.x;
                            enemy.orbit_origin_y = enemy.y;
                        }
                        return;
                    }
                    enemy.y += self.background_speed;
                    let (half_w, half_h) = character_trait(enemy.character_id)
                        .map_or((256, 256), |t| (t.hitbox_width, t.hitbox_height));
                    if enemy.x.raw() + half_w < 0
                        || enemy.x.raw() - half_w >= SCREEN_WIDTH_Q12.raw()
                        || enemy.y.raw() + half_h < 0
                        || enemy.y.raw() - half_h >= SCREEN_HEIGHT_Q12.raw()
                    {
                        enemy.active = false;
                    }
                }
                Some(orbit) if orbit.orbit_type == OrbitType::MoveToPosition => {
                    let duration = orbit.duration_frames.max(1);
                    let t =
                        ((enemy.orbit_frame.min(duration) as i64) * 1_000 / duration as i64) as i32;
                    enemy.x = lerp_component(enemy.orbit_origin_x, orbit.target_position.0, t);
                    enemy.y = lerp_component(enemy.orbit_origin_y, orbit.target_position.1, t);
                }
                Some(orbit) if orbit.orbit_type == OrbitType::Circle => {
                    let points = [
                        (48, 0),
                        (44, 18),
                        (34, 34),
                        (18, 44),
                        (0, 48),
                        (-18, 44),
                        (-34, 34),
                        (-44, 18),
                        (-48, 0),
                        (-44, -18),
                        (-34, -34),
                        (-18, -44),
                        (0, -48),
                        (18, -44),
                        (34, -34),
                        (44, -18),
                    ];
                    let (x, y) = points[(enemy.orbit_frame as usize) % points.len()];
                    enemy.x = enemy.orbit_origin_x
                        + crate::fixed::Q12_4((x * i32::from(orbit.radius.raw()) / 48) as i16);
                    enemy.y = enemy.orbit_origin_y
                        + crate::fixed::Q12_4((y * i32::from(orbit.radius.raw()) / 48) as i16);
                }
                Some(orbit) if orbit.orbit_type == OrbitType::Bezier => {
                    let duration = orbit.duration_frames.max(1);
                    let t =
                        ((enemy.orbit_frame.min(duration) as i64) * 1_000 / duration as i64) as i32;
                    enemy.x = bezier_component(
                        enemy.orbit_origin_x,
                        orbit.control_point_1.0,
                        orbit.control_point_2.0,
                        orbit.control_point_end.0,
                        t,
                    );
                    enemy.y = bezier_component(
                        enemy.orbit_origin_y,
                        orbit.control_point_1.1,
                        orbit.control_point_2.1,
                        orbit.control_point_end.1,
                        t,
                    );
                }
                Some(orbit) if orbit.orbit_type == OrbitType::Straight => {
                    enemy.x += enemy.velocity_x;
                    enemy.y += enemy.velocity_y;
                }
                _ => {
                    enemy.x += enemy.velocity_x;
                    enemy.y += enemy.velocity_y;
                }
            }
            enemy.orbit_frame = enemy.orbit_frame.saturating_add(1);
        });
        let pending: Vec<crate::runtime::ObjectState> = {
            let mut bullets = Vec::new();
            self.enemies.for_each_active_mut(|enemy| {
                if let Some(pattern) = fire_pattern(enemy.fire_pattern_id)
                    && enemy.orbit_frame == pattern.fire_frame
                {
                    bullets.push(crate::runtime::ObjectState {
                        character_id: pattern.bullet_character_id,
                        x: enemy.x + pattern.spawn_offset_x,
                        y: enemy.y + pattern.spawn_offset_y,
                        velocity_x: pattern.velocity().0,
                        velocity_y: pattern.velocity().1,
                        ..Default::default()
                    });
                }
            });
            bullets
        };
        for bullet in pending {
            self.enemy_bullets.spawn(bullet);
        }
    }

    pub fn update_player(&mut self, move_x: i8, move_y: i8) {
        self.player_x = Self::clamp_q12(
            self.player_x + crate::fixed::Q12_4(i16::from(move_x) * 32),
            0,
            SCREEN_WIDTH_Q12.raw(),
        );
        self.player_y = Self::clamp_q12(
            self.player_y + crate::fixed::Q12_4(i16::from(move_y) * 32),
            0,
            SCREEN_HEIGHT_Q12.raw(),
        );
    }

    pub fn update_projectiles(&mut self, fire_held: bool, fire_trigger: bool) {
        self.player_bullets.for_each_active_mut(|bullet| {
            bullet.x += bullet.velocity_x;
            bullet.y += bullet.velocity_y;
            if bullet.y.raw() < -512
                || bullet.x.raw() < -128
                || bullet.x.raw() >= SCREEN_WIDTH_Q12.raw() + 128
            {
                bullet.active = false;
            }
        });
        self.enemy_bullets.for_each_active_mut(|bullet| {
            bullet.x += bullet.velocity_x;
            bullet.y += bullet.velocity_y;
            if bullet.y.raw() < -512
                || bullet.y.raw() >= SCREEN_HEIGHT_Q12.raw() + 512
                || bullet.x.raw() < -128
                || bullet.x.raw() >= SCREEN_WIDTH_Q12.raw() + 128
            {
                bullet.active = false;
            }
        });

        if self.player_group_fire_cooldown > 0 {
            self.player_group_fire_cooldown -= 1;
        }

        let max_player_bullet_groups = match self.growth_level {
            0 => 4,
            1 => 6,
            2 => 8,
            3 => 10,
            _ => 12,
        };
        let interval_elapsed = self.player_group_fire_cooldown == 0;
        if (fire_trigger || (fire_held && interval_elapsed))
            && self.active_player_bullet_group_count() < max_player_bullet_groups
        {
            let shot_count = match self.growth_level {
                0 => 1,
                1 => 2,
                2 => 3,
                3 => 4,
                _ => 5,
            };
            let group_id = self.next_player_bullet_group_id;
            self.next_player_bullet_group_id =
                self.next_player_bullet_group_id.wrapping_add(1).max(1);
            let bullet_speed = PLAYER_BULLET_SPEED_BY_LEVEL_Q12
                [usize::from(self.growth_level.min(4))];
            for index in 0..shot_count {
                let offset_pixels = (index as i16 * 16) - ((shot_count - 1) as i16 * 8);
                let _ = self.player_bullets.spawn(crate::runtime::ObjectState {
                    character_id: 1,
                    group_id,
                    x: self.player_x + crate::fixed::Q12_4::from_int(offset_pixels),
                    y: self.player_y,
                    velocity_y: bullet_speed,
                    ..Default::default()
                });
            }
            if let Some(sound_id) = bullet_character(1).map(|data| data.fire_sound_id) {
                self.audio_events.push(AudioEvent::Sound(sound_id));
            }
            self.player_group_fire_cooldown = PLAYER_GROUP_FIRE_INTERVAL_FRAMES;
        }
    }

    pub fn update_items(&mut self) {
        let player_box = Self::character_hitbox(1, self.player_x, self.player_y)
            .expect("player character trait should exist");
        let mut collected = 0u8;
        self.items.for_each_active_mut(|item| {
            item.y += item.velocity_y;
            let Some(item_box) = Self::character_hitbox(item.character_id, item.x, item.y) else {
                return;
            };
            if overlaps(player_box, item_box) {
                item.active = false;
                collected = collected.saturating_add(1);
            } else if item.y.raw() < -256 || item.y.raw() >= SCREEN_HEIGHT_Q12.raw() + 256 {
                item.active = false;
            }
        });
        if collected > 0 {
            if self.hp < PLAYER_MAX_HP {
                for _ in 0..collected {
                    self.recover_hp();
                }
            } else if self.growth_level < 4 {
                self.growth_level = self.growth_level.saturating_add(collected).min(4);
            } else {
                self.recovery_stock = self.recovery_stock.saturating_add(collected).min(9);
            }
            let _ = self.effects.spawn(EffectState {
                effect_id: 1, // growth/item effect
                x: self.player_x,
                y: self.player_y,
                frame: 0,
                max_frames: 20,
                size: 24,
                ..Default::default()
            });
        }
    }

    pub fn update_effects(&mut self) {
        self.effects.for_each_active_mut(|effect| {
            effect.frame = effect.frame.saturating_add(1);
            if effect.frame >= effect.max_frames {
                effect.active = false;
            }
        });
    }

    fn clamp_q12(value: crate::fixed::Q12_4, minimum: i16, maximum: i16) -> crate::fixed::Q12_4 {
        crate::fixed::Q12_4(value.raw().clamp(minimum, maximum))
    }

    pub fn update_background(&mut self) {
        self.background_scroll += self.background_speed;
    }

    pub fn resolve_player_bullet_hits(&mut self) {
        let bullet_indices = self.player_bullets.active_indices();
        let enemy_snapshots: Vec<CollisionSnapshot> = self
            .enemies
            .active_indices()
            .into_iter()
            .filter_map(|index| {
                self.enemies.get(index).map(|enemy| CollisionSnapshot {
                    index,
                    character_id: enemy.character_id,
                    x: enemy.x,
                    y: enemy.y,
                })
            })
            .collect();
        for bullet_index in bullet_indices {
            let Some(bullet) = self.player_bullets.get(bullet_index).copied() else {
                continue;
            };
            if !bullet.active {
                continue;
            }
            let penetrating = bullet_character(bullet.character_id)
                .is_some_and(|bullet_data| bullet_data.penetrating);
            let mut consumed = false;
            for enemy_snapshot in &enemy_snapshots {
                let Some(enemy) = self.enemies.get(enemy_snapshot.index).copied() else {
                    continue;
                };
                if enemy.active
                    && Self::bullet_hitbox(bullet.character_id, bullet.x, bullet.y)
                        .zip(Self::character_hitbox(
                            enemy_snapshot.character_id,
                            enemy_snapshot.x,
                            enemy_snapshot.y,
                        ))
                        .is_some_and(|(bullet_box, enemy_box)| overlaps(bullet_box, enemy_box))
                {
                    if let Some(enemy) = self.enemies.get_mut(enemy_snapshot.index) {
                        let damage =
                            bullet_character(bullet.character_id).map_or(0, |bullet_data| {
                                bullet_data
                                    .damage
                                    .saturating_add(u16::from(self.growth_level))
                            });
                        enemy.hp = enemy.hp.saturating_sub(damage);
                        if let Some(sound_id) =
                            bullet_character(bullet.character_id).map(|data| data.hit_sound_id)
                        {
                            self.audio_events.push(AudioEvent::Sound(sound_id));
                        }
                        if enemy.hp == 0 {
                            if let Some(trait_data) = character_trait(enemy.character_id) {
                                self.score = self.score.saturating_add(trait_data.score);
                                if trait_data.destroy_effect_id != 0 {
                                    let _ = self.effects.spawn(EffectState {
                                        effect_id: trait_data.destroy_effect_id,
                                        x: enemy.x,
                                        y: enemy.y,
                                        frame: 0,
                                        max_frames: if trait_data.destroy_effect_id == 2 {
                                            75
                                        } else {
                                            20
                                        },
                                        size: if trait_data.destroy_effect_id == 2 {
                                            160
                                        } else {
                                            24
                                        },
                                        ..Default::default()
                                    });
                                }
                                if trait_data.character_type == CharacterType::Boss {
                                    self.state = GameState::StageClear;
                                    self.frame = 0;
                                }
                                if trait_data.destroy_sound_id != 0 {
                                    self.audio_events
                                        .push(AudioEvent::Sound(trait_data.destroy_sound_id));
                                }
                            }
                            enemy.active = false;
                        }
                    }
                    if !penetrating {
                        consumed = true;
                        break;
                    }
                }
            }
            if consumed && let Some(bullet) = self.player_bullets.get_mut(bullet_index) {
                bullet.active = false;
            }
        }
    }

    pub fn resolve_player_hits(&mut self) {
        if self.invincible_frames > 0 {
            self.invincible_frames -= 1;
            return;
        }

        let player_box = Self::character_hitbox(1, self.player_x, self.player_y)
            .expect("player character trait should exist");
        let enemy_snapshots = self.collision_snapshots(&self.enemies);
        let bullet_snapshots = self.collision_snapshots(&self.enemy_bullets);
        let mut damage = None;

        for snapshot in enemy_snapshots {
            if let Some(enemy) = self.enemies.get(snapshot.index).copied()
                && enemy.active
                && Self::character_hitbox(snapshot.character_id, snapshot.x, snapshot.y)
                    .is_some_and(|enemy_box| overlaps(player_box, enemy_box))
            {
                damage = character_trait(snapshot.character_id).map(|data| data.contact_damage);
                break;
            }
        }

        if damage.is_none() {
            for snapshot in bullet_snapshots {
                if let Some(bullet) = self.enemy_bullets.get(snapshot.index).copied()
                    && bullet.active
                    && Self::bullet_hitbox(snapshot.character_id, snapshot.x, snapshot.y)
                        .is_some_and(|bullet_box| overlaps(player_box, bullet_box))
                {
                    if let Some(bullet) = self.enemy_bullets.get_mut(snapshot.index) {
                        bullet.active = false;
                    }
                    damage = bullet_character(snapshot.character_id).map(|data| data.player_damage);
                    break;
                }
            }
        }

        if let Some(amount) = damage {
            self.hp = self.hp.saturating_sub(amount);
            self.growth_level = 0;
            self.invincible_frames = 90;
            self.audio_events.push(AudioEvent::Sound(4));
            while self.hp < PLAYER_MAX_HP && self.recovery_stock > 0 {
                self.recover_hp();
                self.recovery_stock -= 1;
            }
            if self.hp == 0 {
                self.lives = self.lives.saturating_sub(1);
                if self.lives == 0 {
                    if self.ranking.is_high_score(self.score) {
                        self.begin_name_entry();
                    } else {
                        self.state = GameState::GameOver;
                    }
                } else {
                    self.reset_stage();
                    self.state = GameState::StageIntro;
                }
            }
        }
    }

    pub fn resolve_collisions(&mut self) {
        self.resolve_player_bullet_hits();
        if self.state == GameState::Playing {
            self.resolve_player_hits();
        }
    }

    fn collision_snapshots<const N: usize>(
        &self,
        pool: &crate::runtime::ObjectPool<N>,
    ) -> Vec<CollisionSnapshot> {
        pool.active_indices()
            .into_iter()
            .filter_map(|index| {
                pool.get(index).map(|object| CollisionSnapshot {
                    index,
                    character_id: object.character_id,
                    x: object.x,
                    y: object.y,
                })
            })
            .collect()
    }

    pub fn drain_audio_events(&mut self) -> Vec<AudioEvent> {
        std::mem::take(&mut self.audio_events)
    }

    fn spawn_schedule_entry(&mut self, schedule: &ScheduleData) {
        let Some(trait_data) = character_trait(schedule.character_id) else {
            return;
        };
        let object = crate::runtime::ObjectState {
            character_id: schedule.character_id,
            orbit_id: schedule.orbit_id,
            fire_pattern_id: schedule.fire_pattern_id,
            x: schedule.spawn_x,
            y: schedule.spawn_y,
            orbit_origin_x: schedule.spawn_x,
            orbit_origin_y: schedule.spawn_y,
            velocity_y: if schedule.orbit_id == 1 {
                crate::fixed::Q12_4(16)
            } else {
                crate::fixed::Q12_4::ZERO
            },
            hp: trait_data.max_hp,
            ..Default::default()
        };
        match schedule.object_type {
            ObjectType::Enemy | ObjectType::Boss => {
                self.enemies.spawn(object);
            }
            ObjectType::GrowthItem => {
                self.items.spawn(object);
            }
            ObjectType::Background => {}
        }
    }
}

impl crate::data::FirePatternData {
    pub fn velocity(&self) -> (crate::fixed::Q12_4, crate::fixed::Q12_4) {
        use crate::data::Direction16::*;
        let speed = self.speed.raw();
        let diagonal = ((speed as i32 * 181) / 256) as i16;
        match self.direction {
            North => (crate::fixed::Q12_4::ZERO, crate::fixed::Q12_4(-speed)),
            NorthNorthEast => (
                crate::fixed::Q12_4(diagonal / 2),
                crate::fixed::Q12_4(-diagonal),
            ),
            NorthEast => (
                crate::fixed::Q12_4(diagonal),
                crate::fixed::Q12_4(-diagonal),
            ),
            EastNorthEast => (
                crate::fixed::Q12_4(speed),
                crate::fixed::Q12_4(-diagonal / 2),
            ),
            East => (crate::fixed::Q12_4(speed), crate::fixed::Q12_4::ZERO),
            EastSouthEast => (
                crate::fixed::Q12_4(speed),
                crate::fixed::Q12_4(diagonal / 2),
            ),
            SouthEast => (crate::fixed::Q12_4(diagonal), crate::fixed::Q12_4(diagonal)),
            SouthSouthEast => (
                crate::fixed::Q12_4(diagonal / 2),
                crate::fixed::Q12_4(diagonal),
            ),
            South => (crate::fixed::Q12_4::ZERO, crate::fixed::Q12_4(speed)),
            SouthSouthWest => (
                crate::fixed::Q12_4(-diagonal / 2),
                crate::fixed::Q12_4(diagonal),
            ),
            SouthWest => (
                crate::fixed::Q12_4(-diagonal),
                crate::fixed::Q12_4(diagonal),
            ),
            WestSouthWest => (
                crate::fixed::Q12_4(-speed),
                crate::fixed::Q12_4(diagonal / 2),
            ),
            West => (crate::fixed::Q12_4(-speed), crate::fixed::Q12_4::ZERO),
            WestNorthWest => (
                crate::fixed::Q12_4(-speed),
                crate::fixed::Q12_4(-diagonal / 2),
            ),
            NorthWest => (
                crate::fixed::Q12_4(-diagonal),
                crate::fixed::Q12_4(-diagonal),
            ),
            NorthNorthWest => (
                crate::fixed::Q12_4(-diagonal / 2),
                crate::fixed::Q12_4(-diagonal),
            ),
        }
    }
}

fn lerp_component(
    start: crate::fixed::Q12_4,
    end: crate::fixed::Q12_4,
    t: i32,
) -> crate::fixed::Q12_4 {
    let t = i64::from(t.clamp(0, 1_000));
    let value =
        i64::from(start.raw()) + (i64::from(end.raw()) - i64::from(start.raw())) * t / 1_000;
    crate::fixed::Q12_4(value.clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16)
}

fn bezier_component(
    p0: crate::fixed::Q12_4,
    p1: crate::fixed::Q12_4,
    p2: crate::fixed::Q12_4,
    p3: crate::fixed::Q12_4,
    t: i32,
) -> crate::fixed::Q12_4 {
    let t = i64::from(t.clamp(0, 1_000));
    let one_minus_t = 1_000 - t;
    let value = one_minus_t.pow(3) * i64::from(p0.raw())
        + 3 * one_minus_t.pow(2) * t * i64::from(p1.raw())
        + 3 * one_minus_t * t.pow(2) * i64::from(p2.raw())
        + t.pow(3) * i64::from(p3.raw());
    crate::fixed::Q12_4(
        (value / 1_000_i64.pow(3)).clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16,
    )
}
