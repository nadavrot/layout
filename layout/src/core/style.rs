//! This module represents general shape style information.

use crate::core::color::Color;

#[derive(Debug, Clone)]
pub enum Align {
    Center,
    Left,
    Right,
}

impl Align {
    pub fn from_tag_attr_list(list: Vec<(String, String)>) -> Self {
        for (key, value) in list.iter() {
            if key == "align" {
                return match value.as_str() {
                    "left" => Align::Left,
                    "right" => Align::Right,
                    _ => Align::Center,
                };
            }
        }
        Align::Center
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "left" => Align::Left,
            "right" => Align::Right,
            _ => Align::Center,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BAlign {
    Center,
    Left,
    Right,
}
#[derive(Debug, Clone)]
pub enum VAlign {
    Middle,
    Top,
    Bottom,
}

#[derive(Debug, Copy, Clone)]
pub enum LineStyleKind {
    Normal,
    Dashed,
    Dotted,
    None,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum FontStyle {
    None,
    Italic,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum FontWeight {
    None,
    Bold,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct TextDecoration {
    pub(crate) underline: bool,
    pub(crate) overline: bool,
    pub(crate) line_through: bool,
}

impl TextDecoration {
    pub fn new() -> Self {
        Self {
            underline: false,
            overline: false,
            line_through: false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum BaselineShift {
    Sub,
    Super,
    Normal,
}

#[derive(Clone, Debug)]
pub struct StyleAttr {
    pub line_color: Color,
    pub line_width: usize,
    pub fill_color: Option<Color>,
    pub rounded: usize,
    pub font_size: usize,
    pub(crate) fontname: String,
    pub(crate) font_color: Color,
    pub(crate) font_style: FontStyle,
    pub(crate) font_weight: FontWeight,
    pub(crate) text_decoration: TextDecoration,
    pub(crate) baseline_shift: BaselineShift,
    pub(crate) align: Align,
    pub(crate) valign: VAlign,
    pub(crate) balign: BAlign,
}

impl StyleAttr {
    pub fn new(
        line_color: Color,
        line_width: usize,
        fill_color: Option<Color>,
        rounded: usize,
        font_size: usize,
    ) -> Self {
        let font_color = Color::fast("black");
        let fontname = String::from("Times,serif");
        Self {
            line_color,
            line_width,
            fill_color,
            font_color,
            rounded,
            font_size,
            fontname,
            font_style: FontStyle::None,
            font_weight: FontWeight::None,
            text_decoration: TextDecoration::new(),
            baseline_shift: BaselineShift::Normal,
            align: Align::Center,
            valign: VAlign::Middle,
            balign: BAlign::Center,
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
