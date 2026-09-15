use crate::fixed::Q12_4;

pub const ENEMY_CAPACITY: usize = 16;
pub const PLAYER_BULLET_CAPACITY: usize = 1024;
pub const ENEMY_BULLET_CAPACITY: usize = 128;
pub const ITEM_CAPACITY: usize = 16;
pub const EFFECT_CAPACITY: usize = 32;

#[derive(Clone, Copy, Debug, Default)]
pub struct EffectState {
    pub active: bool,
    pub effect_id: u16,
    pub x: Q12_4,
    pub y: Q12_4,
    pub frame: u16,
    pub max_frames: u16,
    pub size: u16,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ObjectState {
    pub active: bool,
    pub character_id: u16,
    pub group_id: u32,
    pub orbit_id: u16,
    pub orbit_frame: u32,
    pub orbit_origin_x: Q12_4,
    pub orbit_origin_y: Q12_4,
    pub orbit_center_x: Q12_4,
    pub orbit_center_y: Q12_4,
    pub fire_pattern_id: u16,
    pub x: Q12_4,
    pub y: Q12_4,
    pub velocity_x: Q12_4,
    pub velocity_y: Q12_4,
    pub homing_frame: u16,
    pub hp: u16,
}

#[derive(Clone, Debug)]
pub struct EffectPool {
    effects: [EffectState; EFFECT_CAPACITY],
}

impl Default for EffectPool {
    fn default() -> Self {
        Self {
            effects: [EffectState::default(); EFFECT_CAPACITY],
        }
    }
}

impl EffectPool {
    pub fn spawn(&mut self, effect: EffectState) -> Option<usize> {
        let index = self.effects.iter().position(|item| !item.active)?;
        self.effects[index] = EffectState {
            active: true,
            ..effect
        };
        Some(index)
    }

    pub fn clear(&mut self) {
        for effect in &mut self.effects {
            effect.active = false;
        }
    }

    #[allow(dead_code)]
    pub fn active_count(&self) -> usize {
        self.effects.iter().filter(|item| item.active).count()
    }

    pub fn for_each_active(&self, mut visitor: impl FnMut(&EffectState)) {
        for effect in &self.effects {
            if effect.active {
                visitor(effect);
            }
        }
    }

    pub fn for_each_active_mut(&mut self, mut update: impl FnMut(&mut EffectState)) {
        for effect in &mut self.effects {
            if effect.active {
                update(effect);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjectPool<const N: usize> {
    objects: [ObjectState; N],
}

impl<const N: usize> Default for ObjectPool<N> {
    fn default() -> Self {
        Self {
            objects: [ObjectState::default(); N],
        }
    }
}

impl<const N: usize> ObjectPool<N> {
    pub fn spawn(&mut self, object: ObjectState) -> Option<usize> {
        let index = self.objects.iter().position(|item| !item.active)?;
        self.objects[index] = ObjectState {
            active: true,
            ..object
        };
        Some(index)
    }

    pub fn clear(&mut self) {
        for object in &mut self.objects {
            object.active = false;
        }
    }

    #[allow(dead_code)]
    pub fn active_count(&self) -> usize {
        self.objects.iter().filter(|item| item.active).count()
    }

    #[allow(dead_code)]
    pub fn first_active(&self) -> Option<&ObjectState> {
        self.objects.iter().find(|item| item.active)
    }

    pub fn for_each_active(&self, mut visitor: impl FnMut(&ObjectState)) {
        for object in &self.objects {
            if object.active {
                visitor(object);
            }
        }
    }

    pub fn for_each_active_mut(&mut self, mut update: impl FnMut(&mut ObjectState)) {
        for object in &mut self.objects {
            if object.active {
                update(object);
            }
        }
    }

    pub fn active_indices(&self) -> Vec<usize> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, object)| object.active.then_some(index))
            .collect()
    }

    pub fn get(&self, index: usize) -> Option<&ObjectState> {
        self.objects.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut ObjectState> {
        self.objects.get_mut(index)
    }
}
