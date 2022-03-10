//! This module represents general shape style information.

use crate::core::color::Color;

#[derive(Copy, Clone)]
pub enum LineStyleKind {
    Normal,
    Dashed,
    Dotted,
    None,
}

#[derive(Clone)]

pub struct StyleAttr {
    pub line_color: Color,
    pub line_width: usize,
    pub fill_color: Option<Color>,
    pub rounded: usize,
    pub font_size: usize,
}

impl StyleAttr {
    pub fn new(
        line_color: Color,
        line_width: usize,
        fill_color: Option<Color>,
        rounded: usize,
        font_size: usize,
    ) -> Self {
        Self {
            line_color,
            line_width,
            fill_color,
            rounded,
            font_size,
        }
    }

    pub fn simple() -> Self {
        StyleAttr::new(
            Color::fast("black"),
            2,
            Option::Some(Color::fast("white")),
            0,
            15,
        )
    }

    pub fn debug0() -> Self {
        StyleAttr::new(
            Color::fast("black"),
            1,
            Option::Some(Color::fast("pink")),
            0,
            15,
        )
    }
    pub fn debug1() -> Self {
        StyleAttr::new(
            Color::fast("black"),
            1,
            Option::Some(Color::fast("aliceblue")),
            0,
            15,
        )
    }
    pub fn debug2() -> Self {
        StyleAttr::new(
            Color::fast("black"),
            1,
            Option::Some(Color::fast("white")),
            0,
            15,
        )
    }
}
