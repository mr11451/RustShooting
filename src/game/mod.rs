pub mod name_entry;
pub mod ranking;
mod state;
mod world;

pub use name_entry::NameEntryState;
#[allow(unused_imports)]
pub use ranking::RankingEntry;
pub use ranking::RankingTable;
pub use state::GameState;
pub use world::World;

use crate::input::InputState;

const RESPAWN_DELAY_FRAMES: u32 = 600;

pub fn update(world: &mut World, input: InputState) {
    if input.quit {
        world.reset_stage();
        world.state = GameState::Title;
        return;
    }

    match world.state {
        GameState::Title => {
            if input.start || input.fire {
                world.restart_from_stage_one();
            } else if world.frame.saturating_add(1) >= 900 {
                world.frame = 0;
                world.state = GameState::Demo;
            }
        }
        GameState::Demo => {
            if input.start || input.fire {
                world.restart_from_stage_one();
            } else if world.frame.saturating_add(1) >= 900 {
                world.frame = 0;
                world.state = GameState::Title;
            }
        }
        GameState::StageIntro if world.frame.saturating_add(1) >= 90 => {
            world.frame = 0;
            world.state = GameState::Playing;
        }
        GameState::StageClear if world.frame.saturating_add(1) >= 90 => {
            if world.stage_id >= 6 {
                world.state = GameState::Ending;
            } else {
                world.stage_id += 1;
                world.advance_to_next_stage();
                world.state = GameState::StageIntro;
            }
        }
        GameState::Respawn if world.frame.saturating_add(1) >= RESPAWN_DELAY_FRAMES => {
            world.reset_stage();
            world.state = GameState::StageIntro;
        }
        GameState::GameOver if world.frame.saturating_add(1) >= 900 => {
            world.reset_stage();
            world.state = GameState::Title;
        }
        GameState::NameEntry => {
            let finished = world.name_entry.update_input(
                input.move_x,
                input.move_y,
                input.fire || input.start,
            );
            if finished {
                world.ranking.insert(world.name_entry.name, world.score);
                let _ = world.ranking.save_default();
                world.restart_from_stage_one();
            } else if world.frame.saturating_add(1) >= 900 {
                world.restart_from_stage_one();
            }
        }
        GameState::Ending if world.frame.saturating_add(1) >= 900 => {
            if world.ranking.is_high_score(world.score) {
                world.begin_name_entry();
            } else {
                world.restart_from_stage_one();
            }
        }
        _ => {}
    }

    if world.state == GameState::Playing {
        world.spawn_scheduled_objects();
        world.update_enemy_movement();
        world.update_background();
        world.update_player(input.move_x, input.move_y);
        world.update_projectiles(input.fire, input.fire_trigger);
        world.update_items();
        world.resolve_collisions();
    } else if world.state == GameState::Respawn {
        world.spawn_scheduled_objects();
        world.update_enemy_movement();
        world.update_background();
        world.update_enemy_projectiles();
    }

    world.update_effects();
    world.frame = world.frame.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioEvent;

    #[test]
    fn playing_world_spawns_stage_one_enemy() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };

        update(&mut world, InputState::default());

        assert_eq!(world.enemies.active_count(), 1);
    }

    #[test]
    fn quit_input_returns_to_title() {
        let mut world = World {
            state: GameState::Playing,
            stage_id: 6,
            ..World::default()
        };

        update(
            &mut world,
            InputState {
                quit: true,
                ..Default::default()
            },
        );

        assert_eq!(world.state, GameState::Title);
        assert_eq!(world.stage_id, 6);
        assert_eq!(world.frame, 0);
    }

    #[test]
    fn starting_from_title_resets_new_game_progress() {
        let mut world = World {
            state: GameState::Title,
            stage_id: 6,
            score: 123_456,
            lives: 1,
            growth_level: 4,
            recovery_stock: 7,
            ..World::default()
        };

        update(
            &mut world,
            InputState {
                start: true,
                ..Default::default()
            },
        );

        assert_eq!(world.state, GameState::StageIntro);
        assert_eq!(world.stage_id, 1);
        assert_eq!(world.score, 0);
        assert_eq!(world.lives, 3);
        assert_eq!(world.growth_level, 0);
        assert_eq!(world.recovery_stock, 0);
    }

    #[test]
    fn scheduled_enemy_moves_down_with_q12_4_velocity() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };

        update(&mut world, InputState::default());
        update(&mut world, InputState::default());

        let enemy = world.enemies.first_active().expect("enemy should exist");
        assert_eq!(enemy.y, crate::fixed::Q12_4(-352));
    }

    #[test]
    fn boss_orbit_loops_inside_screen_bounds() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 100,
            orbit_id: 8,
            hp: 1_000,
            ..Default::default()
        });

        for _ in 0..128 {
            world.update_enemy_movement();
        }

        let boss = world
            .enemies
            .first_active()
            .expect("boss should remain active");
        assert!(boss.x.raw() >= 96 * 16);
        assert!(boss.x.raw() < crate::data::SCREEN_WIDTH_Q12.raw() - 96 * 16);
        assert!(boss.y.raw() >= 64 * 16);
        assert!(boss.y.raw() < crate::data::SCREEN_HEIGHT_Q12.raw() - 64 * 16);
    }

    #[test]
    fn stage_six_boss_uses_dedicated_orbit_data() {
        let boss_schedule = crate::data::stage_schedule(6)
            .find(|schedule| schedule.object_type == crate::data::ObjectType::Boss)
            .expect("stage six boss schedule should exist");

        assert_eq!(boss_schedule.character_id, 105);
        assert_eq!(boss_schedule.orbit_id, 10);
        assert_eq!(
            crate::data::orbit(10).unwrap().orbit_type,
            crate::data::OrbitType::Bezier
        );
    }

    #[test]
    fn circle_orbit_advances_about_one_pixel_per_frame() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 100,
            orbit_id: 9,
            orbit_center_x: crate::data::BOSS_ORBIT_CENTER_X_Q12
                - crate::data::BOSS_ORBIT_RADIUS_Q12,
            orbit_center_y: crate::data::BOSS_ORBIT_CENTER_Y_Q12,
            orbit_origin_x: crate::data::BOSS_ORBIT_CENTER_X_Q12,
            orbit_origin_y: crate::data::BOSS_ORBIT_CENTER_Y_Q12,
            ..Default::default()
        });

        world.update_enemy_movement();
        let first = world.enemies.first_active().unwrap().y;
        world.update_enemy_movement();
        let second = world.enemies.first_active().unwrap().y;

        assert_eq!(second - first, crate::fixed::Q12_4::from_int(1));
    }

    #[test]
    fn circle_orbit_starts_at_previous_trajectory_position() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 100,
            orbit_id: 8,
            orbit_frame: 90,
            orbit_origin_x: crate::fixed::Q12_4(1_000),
            orbit_origin_y: crate::fixed::Q12_4(2_000),
            ..Default::default()
        });

        world.update_enemy_movement();
        let position_after_transition = world.enemies.first_active().unwrap().x;
        world.update_enemy_movement();
        let position_at_circle_start = world.enemies.first_active().unwrap().x;

        assert_eq!(position_at_circle_start, position_after_transition);
    }

    #[test]
    fn enemy_fires_bullet_at_pattern_frame() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };

        for _ in 0..31 {
            update(&mut world, InputState::default());
        }

        assert_eq!(world.enemy_bullets.active_count(), 1);
    }

    #[test]
    fn enemy_bullet_uses_speed_from_pattern_data() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };

        world.spawn_scheduled_objects();
        for _ in 0..30 {
            world.update_enemy_movement();
        }
        let bullet = world
            .enemy_bullets
            .first_active()
            .expect("enemy bullet should be spawned");
        assert_eq!(bullet.velocity_y, crate::fixed::Q12_4(20));
    }

    #[test]
    fn scheduled_growth_item_uses_item_pool() {
        let mut world = World {
            state: GameState::Playing,
            frame: 900,
            ..World::default()
        };
        world.spawn_scheduled_objects();
        assert_eq!(world.items.active_count(), 1);
        assert_eq!(world.enemies.active_count(), 0);
    }

    #[test]
    fn collected_growth_item_increases_growth_level() {
        let mut world = World::default();
        world.items.spawn(crate::runtime::ObjectState {
            character_id: 200,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });
        world.update_items();
        assert_eq!(world.growth_level, 1);
        assert_eq!(world.items.active_count(), 0);
    }

    #[test]
    fn collected_growth_item_recovers_hp_before_leveling_up() {
        let mut world = World {
            hp: 75,
            growth_level: 0,
            recovery_stock: 0,
            ..World::default()
        };
        world.items.spawn(crate::runtime::ObjectState {
            character_id: 200,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });

        world.update_items();

        assert_eq!(world.hp, 100);
        assert_eq!(world.growth_level, 0);
        assert_eq!(world.recovery_stock, 0);
    }

    #[test]
    fn max_growth_converts_items_to_recovery_stock_up_to_nine() {
        let mut world = World {
            growth_level: 4,
            recovery_stock: 8,
            ..World::default()
        };
        for _ in 0..2 {
            world.items.spawn(crate::runtime::ObjectState {
                character_id: 200,
                x: world.player_x,
                y: world.player_y,
                ..Default::default()
            });
        }

        world.update_items();

        assert_eq!(world.growth_level, 4);
        assert_eq!(world.recovery_stock, 9);
        assert_eq!(world.items.active_count(), 0);
    }

    #[test]
    fn player_fire_adds_groups_until_level_limit() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };

        world.update_projectiles(true, true);
        for _ in 0..31 {
            world.update_projectiles(true, false);
        }
        assert_eq!(world.player_bullets.active_count(), 2);
        for _ in 0..90 {
            world.update_projectiles(true, false);
        }
        assert_eq!(world.player_bullets.active_count(), 4);
        world.update_projectiles(true, false);
        assert_eq!(world.player_bullets.active_count(), 4);

        world.player_bullets.clear();
        world.update_projectiles(true, true);
        assert_eq!(world.player_bullets.active_count(), 1);
        world.update_projectiles(true, false);
        assert_eq!(world.player_bullets.active_count(), 1);
    }

    #[test]
    fn player_bullet_speed_increases_by_level() {
        for (expected_speed, growth_level) in
            [(-64i16, 0u8), (-67, 1), (-69, 2), (-73, 3), (-76, 4)]
        {
            let mut world = World {
                state: GameState::Playing,
                growth_level,
                ..World::default()
            };

            world.update_projectiles(false, true);

            let bullet = world
                .player_bullets
                .first_active()
                .expect("bullet should be spawned");
            assert_eq!(bullet.velocity_y.raw(), expected_speed);
        }
    }

    #[test]
    fn trigger_fire_ignores_group_fire_interval() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };

        world.update_projectiles(false, true);
        assert_eq!(world.player_bullets.active_count(), 1);

        world.update_projectiles(false, true);
        assert_eq!(world.player_bullets.active_count(), 2);
    }

    #[test]
    fn max_level_fire_keeps_multiple_volley_bullets() {
        let mut world = World {
            state: GameState::Playing,
            growth_level: 4,
            ..World::default()
        };

        for _ in 0..61 {
            world.update_projectiles(true, false);
        }

        assert_eq!(world.player_bullets.active_count(), 15);
    }

    #[test]
    fn player_fire_queues_sound_id_from_bullet_data() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };

        update(
            &mut world,
            InputState {
                fire: true,
                ..Default::default()
            },
        );

        assert!(matches!(
            world.drain_audio_events().as_slice(),
            [AudioEvent::Sound(1)]
        ));
    }

    #[test]
    fn enemy_bullet_damage_starts_invincibility() {
        let mut world = World {
            state: GameState::Playing,
            hp: 20,
            ..World::default()
        };
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });
        world.resolve_player_hits();
        assert_eq!(world.hp, 10);
        assert_eq!(world.invincible_frames, 90);
        assert!(matches!(
            world.drain_audio_events().as_slice(),
            [AudioEvent::Sound(4)]
        ));
    }

    #[test]
    fn player_and_enemy_bullets_are_destroyed_on_collision() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };
        let x = world.player_x;
        let y = world.player_y;
        world.player_bullets.spawn(crate::runtime::ObjectState {
            character_id: 1,
            x,
            y,
            ..Default::default()
        });
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 6,
            x,
            y,
            ..Default::default()
        });

        world.resolve_collisions();

        assert_eq!(world.player_bullets.active_count(), 0);
        assert_eq!(world.enemy_bullets.active_count(), 0);
    }

    #[test]
    fn player_and_enemy_bullets_are_destroyed_when_crossing_between_frames() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };
        world.player_bullets.spawn(crate::runtime::ObjectState {
            character_id: 1,
            x: crate::fixed::Q12_4(2_000),
            y: crate::fixed::Q12_4(1_000),
            velocity_y: crate::fixed::Q12_4(-64),
            ..Default::default()
        });
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 6,
            x: crate::fixed::Q12_4(2_000),
            y: crate::fixed::Q12_4(936),
            velocity_y: crate::fixed::Q12_4(64),
            ..Default::default()
        });

        world.update_projectiles(false, false);
        world.resolve_collisions();

        assert_eq!(world.player_bullets.active_count(), 0);
        assert_eq!(world.enemy_bullets.active_count(), 0);
    }

    #[test]
    fn homing_enemy_bullet_steers_toward_player() {
        let mut world = World {
            state: GameState::Playing,
            player_x: crate::fixed::Q12_4(2_000),
            player_y: crate::fixed::Q12_4(2_000),
            ..World::default()
        };
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 16,
            x: crate::fixed::Q12_4(1_000),
            y: crate::fixed::Q12_4(2_000),
            velocity_y: crate::fixed::Q12_4(30),
            ..Default::default()
        });

        world.update_projectiles(false, false);

        let bullet = world
            .enemy_bullets
            .first_active()
            .expect("homing bullet should remain active");
        assert!(bullet.velocity_x.raw() > 0);
        assert!(bullet.velocity_y.raw().abs() < 30);
    }

    #[test]
    fn recovery_stock_restores_25_hp_after_damage() {
        let mut world = World {
            state: GameState::Playing,
            hp: 100,
            recovery_stock: 2,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 100,
            x: world.player_x,
            y: world.player_y,
            hp: 1_000,
            ..Default::default()
        });

        world.resolve_player_hits();

        assert_eq!(world.hp, 100);
        assert_eq!(world.recovery_stock, 0);
    }

    #[test]
    fn remaining_recovery_stock_restores_growth_after_hp() {
        let mut world = World {
            state: GameState::Playing,
            hp: 90,
            recovery_stock: 3,
            ..World::default()
        };
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });

        world.resolve_player_hits();

        assert_eq!(world.hp, 100);
        assert_eq!(world.growth_level, 2);
        assert_eq!(world.recovery_stock, 0);
    }

    #[test]
    fn enemy_contact_uses_contact_damage_from_character_data() {
        let mut world = World {
            state: GameState::Playing,
            hp: 100,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 100,
            x: world.player_x,
            y: world.player_y,
            hp: 1_000,
            ..Default::default()
        });

        world.resolve_collisions();

        assert_eq!(world.hp, 50);
        assert_eq!(world.invincible_frames, 90);
    }

    #[test]
    fn damage_can_resume_after_90_invincibility_updates() {
        let mut world = World {
            state: GameState::Playing,
            hp: 100,
            ..World::default()
        };
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });
        world.resolve_player_hits();
        assert_eq!(world.hp, 90);

        for _ in 0..90 {
            world.resolve_player_hits();
        }

        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });
        world.resolve_player_hits();
        assert_eq!(world.hp, 80);
        assert_eq!(world.invincible_frames, 90);
    }

    #[test]
    fn hp_zero_consumes_life_and_restarts_stage() {
        let mut world = World {
            state: GameState::Playing,
            hp: 10,
            recovery_stock: 0,
            player_x: crate::fixed::Q12_4(1_000),
            player_y: crate::fixed::Q12_4(2_000),
            growth_level: 4,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: crate::fixed::Q12_4(100),
            y: crate::fixed::Q12_4(100),
            hp: 10,
            ..Default::default()
        });
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });
        world.resolve_player_hits();
        assert_eq!(world.lives, 2);
        assert_eq!(world.state, GameState::Respawn);
        assert_eq!(world.frame, 0);
        for _ in 0..600 {
            update(&mut world, InputState::default());
        }
        assert_eq!(world.state, GameState::StageIntro);
        assert_eq!(world.hp, 100);
        assert_eq!(world.player_x, crate::data::PLAYER_INITIAL_X_Q12);
        assert_eq!(world.player_y, crate::data::PLAYER_INITIAL_Y_Q12);
        assert_eq!(world.growth_level, 0);
        assert_eq!(world.recovery_stock, 0);
        assert_eq!(world.enemies.active_count(), 0);
        assert_eq!(world.enemy_bullets.active_count(), 0);
    }

    #[test]
    fn recovery_stock_can_prevent_life_loss_at_zero_hp() {
        let mut world = World {
            state: GameState::Playing,
            hp: 10,
            recovery_stock: 2,
            ..World::default()
        };
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });

        world.resolve_player_hits();

        assert_eq!(world.hp, 50);
        assert_eq!(world.lives, 3);
        assert_eq!(world.state, GameState::Playing);
        assert_eq!(world.recovery_stock, 0);
    }

    #[test]
    fn boss_destruction_starts_stage_clear_and_scores() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 100,
            x: world.player_x,
            y: world.player_y,
            hp: 1,
            ..Default::default()
        });
        world.player_bullets.spawn(crate::runtime::ObjectState {
            character_id: 1,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });
        world.resolve_player_bullet_hits();
        assert_eq!(world.state, GameState::StageClear);
        assert_eq!(world.score, 10_000);
        let events = world.drain_audio_events();
        assert!(matches!(
            events.as_slice(),
            [AudioEvent::Sound(2), AudioEvent::Sound(3)]
        ));
    }

    #[test]
    fn growth_level_increases_player_bullet_damage() {
        let mut world = World {
            state: GameState::Playing,
            growth_level: 2,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            hp: 3,
            ..Default::default()
        });
        world.player_bullets.spawn(crate::runtime::ObjectState {
            character_id: 1,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });

        world.resolve_player_bullet_hits();

        assert_eq!(world.enemies.active_count(), 0);
        assert_eq!(world.score, 100);
    }

    #[test]
    fn resolve_collisions_processes_player_and_enemy_interactions() {
        let mut world = World {
            state: GameState::Playing,
            hp: 20,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: crate::fixed::Q12_4(100),
            y: crate::fixed::Q12_4(100),
            hp: 1,
            ..Default::default()
        });
        world.player_bullets.spawn(crate::runtime::ObjectState {
            character_id: 1,
            x: crate::fixed::Q12_4(100),
            y: crate::fixed::Q12_4(100),
            ..Default::default()
        });
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });

        world.resolve_collisions();

        assert_eq!(world.enemies.active_count(), 0);
        assert_eq!(world.score, 100);
        assert_eq!(world.hp, 10);
    }

    #[test]
    fn boss_clear_stops_remaining_player_hit_checks_in_same_frame() {
        let mut world = World {
            state: GameState::Playing,
            hp: 100,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 100,
            x: crate::fixed::Q12_4(100),
            y: crate::fixed::Q12_4(100),
            hp: 1,
            ..Default::default()
        });
        world.player_bullets.spawn(crate::runtime::ObjectState {
            character_id: 1,
            x: crate::fixed::Q12_4(100),
            y: crate::fixed::Q12_4(100),
            ..Default::default()
        });
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });

        world.resolve_collisions();

        assert_eq!(world.state, GameState::StageClear);
        assert_eq!(world.hp, 100);
        assert_eq!(world.enemy_bullets.active_count(), 1);
    }

    #[test]
    fn non_penetrating_bullet_hits_only_one_enemy() {
        let mut world = World::default();
        for _ in 0..2 {
            world.enemies.spawn(crate::runtime::ObjectState {
                character_id: 2,
                x: world.player_x,
                y: world.player_y,
                hp: 1,
                ..Default::default()
            });
        }
        world.player_bullets.spawn(crate::runtime::ObjectState {
            character_id: 1,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });

        world.resolve_player_bullet_hits();

        assert_eq!(world.enemies.active_count(), 1);
        assert_eq!(world.score, 100);
        assert_eq!(world.player_bullets.active_count(), 0);
    }

    #[test]
    fn penetrating_bullet_hits_multiple_enemies() {
        let mut world = World::default();
        for _ in 0..2 {
            world.enemies.spawn(crate::runtime::ObjectState {
                character_id: 2,
                x: world.player_x,
                y: world.player_y,
                hp: 1,
                ..Default::default()
            });
        }
        world.player_bullets.spawn(crate::runtime::ObjectState {
            character_id: 4,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });

        world.resolve_player_bullet_hits();

        assert_eq!(world.enemies.active_count(), 0);
        assert_eq!(world.score, 200);
        assert_eq!(world.player_bullets.active_count(), 1);
    }

    #[test]
    fn simultaneous_enemy_bullets_cause_only_one_damage_per_frame() {
        let mut world = World {
            state: GameState::Playing,
            hp: 100,
            ..World::default()
        };
        for _ in 0..2 {
            world.enemy_bullets.spawn(crate::runtime::ObjectState {
                character_id: 2,
                x: world.player_x,
                y: world.player_y,
                ..Default::default()
            });
        }

        world.resolve_player_hits();

        assert_eq!(world.hp, 90);
        assert_eq!(world.enemy_bullets.active_count(), 1);
        assert_eq!(world.invincible_frames, 90);
    }

    #[test]
    fn enemy_body_has_priority_over_enemy_bullet() {
        let mut world = World {
            state: GameState::Playing,
            hp: 100,
            ..World::default()
        };
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 100,
            x: world.player_x,
            y: world.player_y,
            hp: 1_000,
            ..Default::default()
        });
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });

        world.resolve_collisions();

        assert_eq!(world.hp, 50);
        assert_eq!(world.invincible_frames, 90);
        assert_eq!(world.enemy_bullets.active_count(), 1);
    }

    #[test]
    fn stage_clear_advances_to_next_stage_after_90_frames() {
        let mut world = World {
            state: GameState::StageClear,
            frame: 90,
            stage_id: 1,
            ..World::default()
        };
        update(&mut world, InputState::default());
        assert_eq!(world.stage_id, 2);
        assert_eq!(world.state, GameState::StageIntro);
    }

    #[test]
    fn stage_intro_last_frame_transitions_after_90_updates() {
        let mut world = World {
            state: GameState::StageIntro,
            ..World::default()
        };

        for _ in 0..90 {
            update(&mut world, InputState::default());
        }

        assert_eq!(world.state, GameState::Playing);
        assert_eq!(world.frame, 1);
    }

    #[test]
    fn stage_clear_transitions_after_90_updates() {
        let mut world = World {
            state: GameState::StageClear,
            stage_id: 1,
            ..World::default()
        };

        for _ in 0..90 {
            update(&mut world, InputState::default());
        }

        assert_eq!(world.stage_id, 2);
        assert_eq!(world.state, GameState::StageIntro);
        assert_eq!(world.frame, 1);
    }

    #[test]
    fn game_over_returns_to_title_after_900_updates() {
        let mut world = World {
            state: GameState::GameOver,
            ..World::default()
        };

        for _ in 0..900 {
            update(&mut world, InputState::default());
        }

        assert_eq!(world.state, GameState::Title);
        assert_eq!(world.frame, 1);
    }

    #[test]
    fn final_stage_clear_enters_ending() {
        let mut world = World {
            state: GameState::StageClear,
            frame: 90,
            stage_id: 6,
            ..World::default()
        };
        update(&mut world, InputState::default());
        assert_eq!(world.state, GameState::Ending);
    }

    #[test]
    fn final_stage_clear_enters_ending_after_90_updates() {
        let mut world = World {
            state: GameState::StageClear,
            stage_id: 6,
            ..World::default()
        };

        for _ in 0..90 {
            update(&mut world, InputState::default());
        }

        assert_eq!(world.state, GameState::Ending);
        assert_eq!(world.stage_id, 6);
    }

    #[test]
    fn name_entry_times_out_to_stage_one_after_900_updates() {
        let mut world = World::default();
        world.begin_name_entry();

        for _ in 0..900 {
            update(&mut world, InputState::default());
        }

        assert_eq!(world.state, GameState::StageIntro);
        assert_eq!(world.stage_id, 1);
        assert_eq!(world.frame, 1);
    }

    #[test]
    fn name_entry_does_not_update_gameplay_objects() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };
        world.begin_name_entry();

        update(&mut world, InputState::default());

        assert_eq!(world.state, GameState::NameEntry);
        assert_eq!(world.enemies.active_count(), 0);
        assert_eq!(world.enemy_bullets.active_count(), 0);
        assert_eq!(world.player_bullets.active_count(), 0);
    }

    #[test]
    fn last_life_loss_enters_game_over() {
        let mut world = World {
            state: GameState::Playing,
            hp: 10,
            lives: 1,
            ..World::default()
        };
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });
        world.resolve_player_hits();
        assert_eq!(world.state, GameState::GameOver);
        assert_eq!(world.lives, 0);
    }

    #[test]
    fn last_life_loss_enters_name_entry_when_high_score() {
        let mut world = World {
            state: GameState::Playing,
            hp: 10,
            lives: 1,
            score: 500_000,
            ranking: RankingTable::default(),
            ..World::default()
        };
        world.enemy_bullets.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: world.player_x,
            y: world.player_y,
            ..Default::default()
        });
        world.resolve_player_hits();
        assert_eq!(world.state, GameState::NameEntry);
        assert_eq!(world.lives, 0);
    }

    #[test]
    fn name_entry_saves_to_ranking_and_starts_stage_one() {
        let mut world = World {
            score: 500_000,
            ranking: RankingTable::default(),
            ..World::default()
        };
        world.begin_name_entry();

        // Type 'A'
        update(
            &mut world,
            InputState {
                fire: true,
                ..Default::default()
            },
        );
        update(&mut world, InputState::default());

        // Type 'A'
        update(
            &mut world,
            InputState {
                fire: true,
                ..Default::default()
            },
        );
        update(&mut world, InputState::default());

        // Type 'A'
        update(
            &mut world,
            InputState {
                fire: true,
                ..Default::default()
            },
        );

        assert_eq!(world.state, GameState::StageIntro);
        assert_eq!(world.stage_id, 1);
        assert_eq!(world.ranking.entries[0].name, *b"AAA");
        assert_eq!(world.ranking.entries[0].score, 500_000);
    }

    #[test]
    fn title_and_demo_timeout_transition() {
        let mut world = World {
            state: GameState::Title,
            ..World::default()
        };
        for _ in 0..900 {
            update(&mut world, InputState::default());
        }
        assert_eq!(world.state, GameState::Demo);

        for _ in 0..900 {
            update(&mut world, InputState::default());
        }
        assert_eq!(world.state, GameState::Title);
    }

    #[test]
    fn enemy_after_orbit_completion_moves_with_background_and_despawns_offscreen() {
        let mut world = World {
            state: GameState::Playing,
            background_speed: crate::fixed::Q12_4(16),
            ..World::default()
        };
        // Enemy 2 has hitbox_width=256, hitbox_height=256 (half_w=256, half_h=256).
        // Screen bottom is SCREEN_HEIGHT_Q12 (10_240). Offscreen bottom is y - 256 >= 10_240 => y >= 10_496.
        let spawn_y = crate::data::SCREEN_HEIGHT_Q12 + crate::fixed::Q12_4(230); // 10_470
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 2,
            orbit_id: 1,      // duration_frames: 540
            orbit_frame: 541, // orbit ended
            x: crate::fixed::Q12_4(1_000),
            y: spawn_y,
            ..Default::default()
        });

        assert_eq!(world.enemies.active_count(), 1);

        // Frame 1: y becomes 10_470 + 16 = 10_486. y - 256 = 10_230 < 10_240, still active.
        world.update_enemy_movement();
        assert_eq!(world.enemies.active_count(), 1);

        // Frame 2: y becomes 10_486 + 16 = 10_502. y - 256 = 10_246 >= 10_240, despawns.
        world.update_enemy_movement();
        assert_eq!(world.enemies.active_count(), 0);
    }

    #[test]
    fn enemy_during_orbit_is_not_despawned_offscreen() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };
        // Enemy spawned way above screen at y = -16_000
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 2,
            orbit_id: 1, // duration_frames: 540
            orbit_frame: 10,
            x: crate::fixed::Q12_4(1_000),
            y: crate::fixed::Q12_4(-16_000),
            velocity_y: crate::fixed::Q12_4(16),
            ..Default::default()
        });

        world.update_enemy_movement();
        assert_eq!(world.enemies.active_count(), 1);
    }

    #[test]
    fn full_gameplay_through_all_six_stages_preserves_growth_and_reaches_ending() {
        let mut world = World {
            state: GameState::StageIntro,
            stage_id: 1,
            growth_level: 3,
            recovery_stock: 2,
            ..World::default()
        };

        for expected_stage in 1..=6 {
            assert_eq!(world.stage_id, expected_stage);
            assert_eq!(world.growth_level, 3);
            assert_eq!(world.recovery_stock, 2);

            // Intro completes after 90 frames
            for _ in 0..90 {
                update(&mut world, InputState::default());
            }
            assert_eq!(world.state, GameState::Playing);

            // Defeat the boss of this stage
            let boss_id = match expected_stage {
                1 => 100,
                2 => 101,
                3 => 102,
                4 => 103,
                5 => 104,
                _ => 105,
            };
            world.enemies.spawn(crate::runtime::ObjectState {
                character_id: boss_id,
                x: world.player_x,
                y: world.player_y,
                hp: 1,
                ..Default::default()
            });
            world.player_bullets.spawn(crate::runtime::ObjectState {
                character_id: 1,
                x: world.player_x,
                y: world.player_y,
                ..Default::default()
            });

            world.resolve_collisions();
            assert_eq!(world.state, GameState::StageClear);

            // Clear completes after 90 frames
            for _ in 0..90 {
                update(&mut world, InputState::default());
            }

            if expected_stage < 6 {
                assert_eq!(world.stage_id, expected_stage + 1);
                assert_eq!(world.state, GameState::StageIntro);
            } else {
                assert_eq!(world.state, GameState::Ending);
            }
        }
    }

    #[test]
    fn enemy_and_item_destruction_spawns_effects() {
        let mut world = World {
            state: GameState::Playing,
            ..World::default()
        };

        // Enemy defeat spawns destruction effect
        world.enemies.spawn(crate::runtime::ObjectState {
            character_id: 2,
            x: crate::fixed::Q12_4(100),
            y: crate::fixed::Q12_4(100),
            hp: 1,
            ..Default::default()
        });
        world.player_bullets.spawn(crate::runtime::ObjectState {
            character_id: 1,
            x: crate::fixed::Q12_4(100),
            y: crate::fixed::Q12_4(100),
            ..Default::default()
        });
        world.resolve_collisions();
        assert_eq!(world.effects.active_count(), 1);

        // Update effects advances frame
        world.update_effects();
        assert_eq!(world.effects.active_count(), 1);
    }
}
