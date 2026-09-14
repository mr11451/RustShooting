use crate::fixed::Q12_4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hitbox {
    pub x: Q12_4,
    pub y: Q12_4,
    pub half_width: Q12_4,
    pub half_height: Q12_4,
}

pub fn overlaps(left: Hitbox, right: Hitbox) -> bool {
    let left_right = left.x + left.half_width;
    let right_left = right.x - right.half_width;
    let right_right = right.x + right.half_width;
    let left_left = left.x - left.half_width;
    let left_bottom = left.y + left.half_height;
    let right_top = right.y - right.half_height;
    let right_bottom = right.y + right.half_height;
    let left_top = left.y - left.half_height;

    left_right >= right_left
        && right_right >= left_left
        && left_bottom >= right_top
        && right_bottom >= left_top
}

#[cfg(test)]
mod tests {
    use super::*;

    fn box_at(x: i16, y: i16, width: i16, height: i16) -> Hitbox {
        Hitbox {
            x: Q12_4(x),
            y: Q12_4(y),
            half_width: Q12_4(width),
            half_height: Q12_4(height),
        }
    }

    #[test]
    fn touching_edges_are_hits() {
        assert!(overlaps(box_at(0, 0, 16, 16), box_at(32, 0, 16, 16)));
    }

    #[test]
    fn separated_boxes_do_not_hit() {
        assert!(!overlaps(box_at(0, 0, 8, 8), box_at(32, 0, 8, 8)));
    }
}
