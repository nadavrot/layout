//! Contains utilities, enums, constants and simple data structures that are
//! used across the program.

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Both,
    None,
}

impl Direction {
    pub fn is_down(&self) -> bool {
        match self {
            Direction::None | Direction::Up => false,
            Direction::Both | Direction::Down => true,
        }
    }
    pub fn is_up(&self) -> bool {
        match self {
            Direction::Both | Direction::Up => true,
            Direction::None | Direction::Down => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    TopToBottom,
    LeftToRight,
}

impl Orientation {
    pub fn is_top_to_bottom(&self) -> bool {
        if let Orientation::TopToBottom = self {
            return true;
        }
        false
    }
    pub fn is_left_right(&self) -> bool {
        if let Orientation::TopToBottom = self {
            return false;
        }
        true
    }
    pub fn flip(&self) -> Orientation {
        if let Orientation::TopToBottom = self {
            return Orientation::LeftToRight;
        }
        Orientation::TopToBottom
    }
}
