use super::World;
use crate::input::InputState;

pub fn update_frame(world: &mut World, input: InputState) {
    super::update(world, input);
}
