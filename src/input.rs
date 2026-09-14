#[derive(Clone, Copy, Debug, Default)]
pub struct InputState {
    pub move_x: i8,
    pub move_y: i8,
    pub fire: bool,
    pub start: bool,
    pub quit: bool,
}

impl InputState {
    pub fn set_move_key(&mut self, axis: MoveAxis, pressed: bool) {
        match axis {
            MoveAxis::Left => self.move_x = if pressed { -1 } else { 0 },
            MoveAxis::Right => self.move_x = if pressed { 1 } else { 0 },
            MoveAxis::Up => self.move_y = if pressed { -1 } else { 0 },
            MoveAxis::Down => self.move_y = if pressed { 1 } else { 0 },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MoveAxis {
    Left,
    Right,
    Up,
    Down,
}

pub trait PlatformInput {
    fn poll(&mut self) -> InputState;
}
