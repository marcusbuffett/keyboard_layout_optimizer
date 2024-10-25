#[derive(Clone, Debug, PartialEq, Copy)]
pub enum SvalKeyDirection {
    North,
    South,
    East,
    West,
    Center,
}
impl SvalKeyDirection {
    pub fn from_key(key: &keyboard_layout::key::Key, closest_center: &(u8, u8)) -> Self {
        let (x, y) = (key.matrix_position.0, key.matrix_position.1);
        if x == closest_center.0 && y == closest_center.1 {
            return SvalKeyDirection::Center;
        }
        if x == closest_center.0 {
            if y < closest_center.1 {
                SvalKeyDirection::North
            } else {
                SvalKeyDirection::South
            }
        } else if y == closest_center.1 {
            if x < closest_center.0 {
                SvalKeyDirection::West
            } else {
                SvalKeyDirection::East
            }
        } else {
            panic!("Key is not on the closest center");
        }
    }
}
