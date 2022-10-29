//! The shapes that are defined in this module are used to describe and render the
//! shapes that appear on the screen. Things like 'circle' and 'edge' go in here.
//! Shapes need to contain all of the information that they need to be rendered.
//! This includes things like font size, and color.

use crate::core::base::Orientation;
use crate::core::format::Visible;
use crate::core::geometry::{Point, Position};
use crate::core::style::{LineStyleKind, StyleAttr};
use crate::std_shapes::render::get_shape_size;

const PADDING: f64 = 60.;
const CONN_PADDING: f64 = 10.;

#[derive(Copy, Clone)]
pub enum LineEndKind {
    None,
    Arrow,
}

#[derive(Clone)]
pub enum RecordDef {
    // Label, port:
    Text(String, Option<String>),
    Array(Vec<RecordDef>),
}

impl RecordDef {
    pub fn new_text(s: &str) -> Self {
        RecordDef::Text(s.to_string(), None)
    }

    pub fn new_text_with_port(s: &str, p: &str) -> Self {
        RecordDef::Text(s.to_string(), Some(p.to_string()))
    }
}

#[derive(Clone)]
pub enum ShapeKind {
    None,
    Box(String),
    Circle(String),
    DoubleCircle(String),
    Record(RecordDef),
    Connector(Option<String>),
}

impl ShapeKind {
    pub fn new_box(s: &str) -> Self {
        ShapeKind::Box(s.to_string())
    }
    pub fn new_circle(s: &str) -> Self {
        ShapeKind::Circle(s.to_string())
    }
    pub fn new_double_circle(s: &str) -> Self {
        ShapeKind::DoubleCircle(s.to_string())
    }
    pub fn new_record(r: &RecordDef) -> Self {
        ShapeKind::Record(r.clone())
    }
    pub fn new_connector(s: &str) -> Self {
        if s.is_empty() {
            return ShapeKind::Connector(None);
        }
        ShapeKind::Connector(Some(s.to_string()))
    }
}

#[derive(Clone)]
pub struct Element {
    pub shape: ShapeKind,
    pub pos: Position,
    pub look: StyleAttr,
    pub orientation: Orientation,
}

impl Element {
    pub fn create(
        shape: ShapeKind,
        look: StyleAttr,
        orientation: Orientation,
        size: Point,
    ) -> Element {
        Element {
            shape,
            look,
            orientation,
            pos: Position::new(
                Point::zero(),
                size,
                Point::zero(),
                Point::splat(PADDING),
            ),
        }
    }
    pub fn create_connector(
        label: &str,
        look: &StyleAttr,
        dir: Orientation,
    ) -> Element {
        Element {
            shape: ShapeKind::new_connector(label),
            look: look.clone(),
            orientation: dir,
            pos: Position::new(
                Point::zero(),
                Point::zero(),
                Point::zero(),
                Point::splat(CONN_PADDING),
            ),
        }
    }

    pub fn empty_connector(dir: Orientation) -> Element {
        Self::create_connector("", &StyleAttr::simple(), dir)
    }

    // Make the center of the shape point to \p to.
    pub fn move_to(&mut self, to: Point) {
        self.pos.move_to(to)
    }
}

#[derive(Clone)]
pub struct Arrow {
    pub start: LineEndKind,
    pub end: LineEndKind,
    pub line_style: LineStyleKind,
    pub text: String,
    pub look: StyleAttr,
    pub src_port: Option<String>,
    pub dst_port: Option<String>,
}

impl Default for Arrow {
    fn default() -> Arrow {
        Arrow {
            start: LineEndKind::None,
            end: LineEndKind::Arrow,
            line_style: LineStyleKind::Normal,
            text: String::new(),
            look: StyleAttr::simple(),
            src_port: Option::None,
            dst_port: Option::None,
        }
    }
}

impl Arrow {
    pub fn reverse(&self) -> Arrow {
        Arrow {
            start: self.end,
            end: self.start,
            line_style: self.line_style,
            text: self.text.clone(),
            look: self.look.clone(),
            src_port: self.dst_port.clone(),
            dst_port: self.src_port.clone(),
        }
    }

    pub fn new(
        start: LineEndKind,
        end: LineEndKind,
        line_style: LineStyleKind,
        text: &str,
        look: &StyleAttr,
        src_port: &Option<String>,
        dst_port: &Option<String>,
    ) -> Arrow {
        Arrow {
            start,
            end,
            line_style,
            text: String::from(text),
            look: look.clone(),
            src_port: src_port.clone(),
            dst_port: dst_port.clone(),
        }
    }

    pub fn simple(text: &str) -> Arrow {
        Arrow::new(
            LineEndKind::None,
            LineEndKind::Arrow,
            LineStyleKind::Normal,
            text,
            &StyleAttr::simple(),
            &None,
            &None,
        )
    }

    pub fn invisible() -> Arrow {
        Arrow::new(
            LineEndKind::None,
            LineEndKind::None,
            LineStyleKind::None,
            "",
            &StyleAttr::simple(),
            &None,
            &None,
        )
    }
}

impl Visible for Element {
    fn position(&self) -> Position {
        self.pos
    }
    fn position_mut(&mut self) -> &mut Position {
        &mut self.pos
    }

    fn is_connector(&self) -> bool {
        matches!(self.shape, ShapeKind::Connector(_))
    }

    fn transpose(&mut self) {
        self.orientation = self.orientation.flip();
        self.pos.transpose();
    }

    fn resize(&mut self) {
        if let ShapeKind::Connector(_) = self.shape.clone() {
            let size = get_shape_size(
                self.orientation,
                &self.shape,
                self.look.font_size,
                false,
            );
            self.pos.set_size(size);
            match self.orientation {
                Orientation::TopToBottom => {
                    self.pos.set_new_center_point(Point::new(0., size.y / 2.));
                }
                Orientation::LeftToRight => {
                    self.pos.set_new_center_point(Point::new(size.x / 2., 0.));
                }
            }
        }
    }
}
