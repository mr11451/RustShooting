#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub move_x: i8,
    pub move_y: i8,
    pub fire: bool,
    pub fire_trigger: bool,
    pub start: bool,
    pub quit: bool,
    pub(crate) left_pressed: bool,
    pub(crate) right_pressed: bool,
    pub(crate) up_pressed: bool,
    pub(crate) down_pressed: bool,
    pub(crate) controller_left_pressed: bool,
    pub(crate) controller_right_pressed: bool,
    pub(crate) controller_up_pressed: bool,
    pub(crate) controller_down_pressed: bool,
    pub(crate) keyboard_fire: bool,
    pub(crate) controller_fire: bool,
}

impl InputState {
    pub fn set_move_key(&mut self, axis: MoveAxis, pressed: bool) {
        match axis {
            MoveAxis::Left => self.left_pressed = pressed,
            MoveAxis::Right => self.right_pressed = pressed,
            MoveAxis::Up => self.up_pressed = pressed,
            MoveAxis::Down => self.down_pressed = pressed,
        }
        self.recompute_axes_and_fire();
    }

    pub fn set_controller_move_key(&mut self, axis: MoveAxis, pressed: bool) {
        match axis {
            MoveAxis::Left => self.controller_left_pressed = pressed,
            MoveAxis::Right => self.controller_right_pressed = pressed,
            MoveAxis::Up => self.controller_up_pressed = pressed,
            MoveAxis::Down => self.controller_down_pressed = pressed,
        }
        self.recompute_axes_and_fire();
    }

    pub fn set_controller_axis(&mut self, axis: MoveAxis, value: f32) {
        match axis {
            MoveAxis::Left => self.controller_left_pressed = value <= -0.5,
            MoveAxis::Right => self.controller_right_pressed = value >= 0.5,
            MoveAxis::Up => self.controller_up_pressed = value <= -0.5,
            MoveAxis::Down => self.controller_down_pressed = value >= 0.5,
        }
        self.recompute_axes_and_fire();
    }

    pub fn set_controller_fire(&mut self, pressed: bool, trigger: bool) {
        self.controller_fire = pressed;
        self.fire_trigger |= trigger;
        self.recompute_axes_and_fire();
    }

    pub fn set_keyboard_fire(&mut self, pressed: bool, trigger: bool) {
        self.keyboard_fire = pressed;
        self.fire_trigger |= trigger;
        self.recompute_axes_and_fire();
    }

    fn recompute_axes_and_fire(&mut self) {
        let left = self.left_pressed || self.controller_left_pressed;
        let right = self.right_pressed || self.controller_right_pressed;
        let up = self.up_pressed || self.controller_up_pressed;
        let down = self.down_pressed || self.controller_down_pressed;
        self.move_x = i8::from(right) - i8::from(left);
        self.move_y = i8::from(down) - i8::from(up);
        self.fire = self.keyboard_fire || self.controller_fire;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MoveAxis {
    Left,
    Right,
    Up,
    Down,
}

#[allow(dead_code)]
pub trait PlatformInput {
    fn poll(&mut self) -> InputState;
}

#[cfg(test)]
mod tests {
    use super::{InputState, MoveAxis};

    #[test]
    fn keeps_remaining_horizontal_direction_after_release() {
        let mut input = InputState::default();
        input.set_move_key(MoveAxis::Left, true);
        input.set_move_key(MoveAxis::Right, true);
        input.set_move_key(MoveAxis::Left, false);

        assert_eq!(input.move_x, 1);
    }

    #[test]
    fn keeps_remaining_vertical_direction_after_release() {
        let mut input = InputState::default();
        input.set_move_key(MoveAxis::Up, true);
        input.set_move_key(MoveAxis::Down, true);
        input.set_move_key(MoveAxis::Up, false);

        assert_eq!(input.move_y, 1);
    }
}
