use crate::core::geometry::{get_size_for_str, pad_shape_scalar, Point};
use std::collections::HashMap;

use crate::std_shapes::render::BOX_SHAPE_PADDING;

use crate::core::style::{Align, BAlign, FontStyle, StyleAttr, VAlign};

use crate::core::color::Color;

/// Creates an error from the string \p str.
fn to_error<T>(str: &str) -> Result<T, String> {
    Result::Err(str.to_string())
}

#[derive(Debug, Clone)]
pub enum Token {
    Colon,
    EOF,
    Identifier(String),
    OpeningTag(TagType),
    ClosingTag(TagType),
    TagEnd,
    TagEndWithSlash,
    TagAttr(String, String),
    Error(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagType {
    Table,
    Tr,
    Td,
    Font,
    Br,
    I,
    Img,
    B,
    U,
    O,
    Sub,
    Sup,
    S,
    Hr,
    Vr,
    Unrecognized,
}

impl TagType {
    pub fn from_str(tag: &str) -> Self {
        // use capital letter for all letters for patter matching
        match tag {
            "table" => TagType::Table,
            "tr" => TagType::Tr,
            "td" => TagType::Td,
            "font" => TagType::Font,
            "br" => TagType::Br,
            "i" => TagType::I,
            "img" => TagType::Img,
            "b" => TagType::B,
            "u" => TagType::U,
            "o" => TagType::O,
            "sub" => TagType::Sub,
            "sup" => TagType::Sup,
            "s" => TagType::S,
            "hr" => TagType::Hr,
            "vr" => TagType::Vr,
            _ => TagType::Unrecognized,
        }
    }
    pub fn is_single_tag(&self) -> bool {
        match self {
            TagType::Br | TagType::Hr | TagType::Vr | TagType::Img => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Scale {
    False,
    True,
    Width,
    Height,
    Both,
}

#[derive(Debug, Clone)]
pub struct Font {
    pub(crate) color: Option<Color>,
    pub(crate) face: Option<String>,
    pub(crate) point_size: Option<f64>,
}

impl Font {
    pub fn new() -> Self {
        Self {
            color: None,
            face: None,
            point_size: None,
        }
    }

    pub fn set_attr(&mut self, attr: &str, value: &str) {
        match attr {
            "color" => {
                self.color = {
                    if let Some(color) = Color::from_name(value) {
                        Some(color)
                    } else {
                        None
                    }
                }
            }
            "face" => self.face = Some(value.to_string()),
            "point-size" => self.point_size = value.parse().ok(),
            _ => {}
        }
    }

    pub fn from_tag_attr_list(list: Vec<(String, String)>) -> Self {
        let mut font = Self::new();
        for (key, value) in list.iter() {
            font.set_attr(key, value);
        }
        font
    }
}

#[derive(Debug, Clone)]
pub enum TextTag {
    Font(Font),
    I,
    B,
    U,
    O,
    Sub,
    Sup,
    S,
}

impl TextTag {
    pub fn new(tag: &TagType, tag_attr_list: Vec<(String, String)>) -> Self {
        match tag {
            TagType::Font => {
                let font = Font::from_tag_attr_list(tag_attr_list);
                TextTag::Font(font)
            }
            TagType::I => TextTag::I,
            TagType::B => TextTag::B,
            TagType::U => TextTag::U,
            TagType::O => TextTag::O,
            TagType::Sub => TextTag::Sub,
            TagType::Sup => TextTag::Sup,
            TagType::S => TextTag::S,
            _ => panic!("Invalid tag for text: {:?}", tag),
        }
    }
}

pub type Text = Vec<TextItem>;

#[derive(Debug, Clone)]
pub struct TaggedText {
    pub text_items: Text,
    pub tag: TextTag,
}

#[derive(Debug, Clone)]
pub enum TableTag {
    None,
    Font(Font),
    I,
    B,
    U,
    O,
}
impl TableTag {
    pub fn from_tag(
        tag_pair: Option<(TagType, Vec<(String, String)>)>,
    ) -> Self {
        if let Some(tag_inner) = tag_pair {
            match tag_inner.0 {
                TagType::Table => TableTag::None,
                TagType::Font => TableTag::Font(Font::from_tag_attr_list(
                    tag_inner.1.clone(),
                )),
                TagType::I => TableTag::I,
                TagType::B => TableTag::B,
                TagType::U => TableTag::U,
                TagType::O => TableTag::O,
                _ => panic!("Invalid tag for table: {:?}", tag_inner.0),
            }
        } else {
            TableTag::None
        }
    }
}
#[derive(Debug, Clone)]
pub enum LabelOrImg {
    Html(Html),
    Img(Scale, String),
}
#[derive(Debug, Clone)]
pub struct DotCell {
    pub label: LabelOrImg,
    pub td_attr: TdAttr,
}

#[derive(Debug, Clone)]
pub struct DotCellGrid {
    pub(crate) i: usize,
    pub(crate) j: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub label_grid: LabelOrImgGrid,
    pub td_attr: TdAttr,
}

impl DotCellGrid {
    pub fn from_dot_cell(
        i: usize,
        j: usize,
        width: usize,
        height: usize,
        dot_cell: &DotCell,
    ) -> Self {
        let label_grid = match &dot_cell.label {
            LabelOrImg::Html(html) => {
                LabelOrImgGrid::Html(HtmlGrid::from_html(html))
            }
            LabelOrImg::Img(scale, img) => {
                LabelOrImgGrid::Img(scale.clone(), img.clone())
            }
        };
        Self {
            i,
            j,
            width,
            height,
            label_grid,
            td_attr: dot_cell.td_attr.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum LabelOrImgGrid {
    Html(HtmlGrid),
    Img(Scale, String),
}

#[derive(Debug, Clone)]
pub struct TdAttr {
    // No inheritance on align, use the most recent value
    align: Align, // CENTER|LEFT|RIGHT
    balign: BAlign,
    valign: VAlign, // MIDDLE|BOTTOM|TOP
    colspan: u16,
    rowspan: u16,
    height: Option<u16>, // value
    width: Option<u16>,  // value
    fixedsize: bool,     // FALSE|TRUE
    sides: Sides,

    // Full inheritance on bgcolor, use only if set
    bgcolor: Option<String>, // color
    color: Option<String>,   // color

    // inheritance only for the first children cell
    pub(crate) border: Option<u8>,      // value
    pub(crate) cellpadding: Option<u8>, // value

    // this seems not to be used by the official graphviz tools
    // nor does it make sense to have it in the cell attributes
    // probably wrong documentation
    // TODO: report it back to graphviz
    cellspacing: Option<u8>, // value

    // TODO: to be implemented
    gradientangle: Option<String>,   // value
    href: Option<String>,            // value
    id: Option<String>,              // value
    pub(crate) port: Option<String>, // portName
    style: Option<String>,           // value
    target: Option<String>,          // value
    title: Option<String>,           // value
    tooltip: Option<String>,         // value
}

impl TdAttr {
    pub fn new() -> Self {
        Self {
            align: Align::Center,
            balign: BAlign::Center,
            bgcolor: None,
            border: None,
            cellpadding: None,
            cellspacing: None,
            color: None,
            colspan: 1,
            fixedsize: false,
            gradientangle: None,
            height: None,
            href: None,
            id: None,
            port: None,
            rowspan: 1,
            sides: Sides::from_str(""),
            style: None,
            target: None,
            title: None,
            tooltip: None,
            valign: VAlign::Middle,
            width: None,
        }
    }

    pub fn from_tag_attr_list(list: Vec<(String, String)>) -> Self {
        let mut attr = Self::new();
        for (key, value) in list.iter() {
            attr.set_attr(key, value);
        }
        attr
    }

    pub fn set_attr(&mut self, attr: &str, value: &str) {
        match attr {
            "align" => {
                self.align = match value {
                    "left" => Align::Left,
                    "right" => Align::Right,
                    _ => Align::Center,
                }
            }
            "balign" => {
                self.balign = match value {
                    "left" => BAlign::Left,
                    "right" => BAlign::Right,
                    _ => BAlign::Center,
                }
            }
            "bgcolor" => self.bgcolor = Some(value.to_string()),
            "border" => self.border = value.parse().ok(),
            "cellpadding" => self.cellpadding = value.parse().ok(),
            "cellspacing" => self.cellspacing = value.parse().ok(),
            "color" => self.color = Some(value.to_string()),
            "colspan" => self.colspan = value.parse().unwrap_or(1),
            "fixedsize" => self.fixedsize = value == "true",
            "gradientangle" => self.gradientangle = Some(value.to_string()),
            "height" => self.height = value.parse().ok(),
            "href" => self.href = Some(value.to_string()),
            "id" => self.id = Some(value.to_string()),
            "port" => self.port = Some(value.to_string()),
            "rowspan" => self.rowspan = value.parse().unwrap_or(1),
            "sides" => self.sides = Sides::from_str(value),
            "style" => self.style = Some(value.to_string()),
            "target" => self.target = Some(value.to_string()),
            "title" => self.title = Some(value.to_string()),
            "tooltip" => self.tooltip = Some(value.to_string()),
            "valign" => {
                self.valign = match value {
                    "top" => VAlign::Top,
                    "bottom" => VAlign::Bottom,
                    _ => VAlign::Middle,
                }
            }
            "width" => self.width = value.parse().ok(),
            _ => {}
        }
    }

    pub fn update_style_attr(&self, style_attr: &mut StyleAttr) {
        if let Some(ref color) = self.bgcolor {
            style_attr.fill_color = Color::from_name(color);
        }
        style_attr.valign = self.valign.clone();
        style_attr.align = self.align.clone();
        style_attr.balign = self.balign.clone();
    }
}

#[derive(Debug, Clone)]
pub enum ColumnFormat {
    Star,
    None,
}

impl ColumnFormat {
    pub fn from_str(s: &str) -> Self {
        if s.starts_with('*') {
            Self::Star
        } else {
            Self::None
        }
    }
}

#[derive(Debug, Clone)]
pub enum RowFormat {
    Star,
    None,
}

impl RowFormat {
    pub fn from_str(s: &str) -> Self {
        if s.starts_with('*') {
            Self::Star
        } else {
            Self::None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sides {
    left: bool,
    right: bool,
    top: bool,
    bottom: bool,
}

impl Sides {
    pub fn from_str(s: &str) -> Self {
        let mut sides = Sides {
            left: false,
            right: false,
            top: false,
            bottom: false,
        };
        for c in s.chars() {
            match c {
                'L' => sides.left = true,
                'R' => sides.right = true,
                'T' => sides.top = true,
                'B' => sides.bottom = true,
                _ => {}
            }
        }
        sides
    }
}

#[derive(Debug, Clone)]
pub struct TableAttr {
    // No inheritance on align, use the most recent value
    align: Align,                  // CENTER|LEFT|RIGHT
    valign: VAlign,                // MIDDLE|BOTTOM|TOP
    sides: Sides,                  // value
    height: Option<u16>,           // value
    width: Option<u16>,            // value
    columns: Option<ColumnFormat>, // value
    rows: Option<RowFormat>,       // value
    fixedsize: bool,               // FALSE|TRUE

    // Full inheritance on bgcolor, use only if set
    color: Option<Color>,   // color
    bgcolor: Option<Color>, // color

    // inheritance only for the first children cell
    pub(crate) border: u8,             // value
    pub(crate) cellborder: Option<u8>, // value
    pub(crate) cellpadding: u8,        // value
    pub(crate) cellspacing: u8,        // value

    gradientangle: Option<String>,   // value
    href: Option<String>,            // value
    id: Option<String>,              // value
    pub(crate) port: Option<String>, // portName
    style: Option<String>,           // value
    target: Option<String>,          // value
    title: Option<String>,           // value
    tooltip: Option<String>,         // value
}

impl TableAttr {
    pub fn new() -> Self {
        Self {
            align: Align::Center,
            bgcolor: None,
            border: 1,
            cellborder: None,
            cellpadding: 2,
            cellspacing: 2,
            color: None,
            columns: None,
            fixedsize: false,
            gradientangle: None,
            height: None,
            href: None,
            id: None,
            port: None,
            rows: None,
            sides: Sides::from_str(""),
            style: None,
            target: None,
            title: None,
            tooltip: None,
            valign: VAlign::Middle,
            width: None,
        }
    }
    pub fn from_attr_list(list: Vec<(String, String)>) -> Self {
        let mut attr = Self::new();
        for (key, value) in list.iter() {
            attr.set_attr(key, value);
        }
        attr
    }

    pub fn set_attr(&mut self, attr: &str, value: &str) {
        let attr = attr.to_lowercase();
        match attr.as_str() {
            "align" => {
                self.align = match value {
                    "left" => Align::Left,
                    "right" => Align::Right,
                    _ => Align::Center,
                }
            }
            "bgcolor" => {
                self.bgcolor = {
                    if let Some(color) = Color::from_name(value) {
                        Some(color)
                    } else {
                        None
                    }
                }
            }
            "border" => self.border = value.parse().unwrap_or(0),
            "cellborder" => self.cellborder = value.parse().ok(),
            "cellpadding" => self.cellpadding = value.parse().unwrap_or(0),
            "cellspacing" => self.cellspacing = value.parse().unwrap_or(0),
            "color" => {
                self.color = {
                    if let Some(color) = Color::from_name(value) {
                        Some(color)
                    } else {
                        None
                    }
                }
            }
            "fixedsize" => self.fixedsize = value == "true",
            "gradientangle" => self.gradientangle = Some(value.to_string()),
            "height" => self.height = value.parse().ok(),
            "width" => self.width = value.parse().ok(),
            "href" => self.href = Some(value.to_string()),
            "id" => self.id = Some(value.to_string()),
            "port" => self.port = Some(value.to_string()),
            "rows" => self.rows = Some(RowFormat::from_str(value)),
            "sides" => self.sides = Sides::from_str(value),
            "style" => self.style = Some(value.to_string()),
            "target" => self.target = Some(value.to_string()),
            "title" => self.title = Some(value.to_string()),
            "tooltip" => self.tooltip = Some(value.to_string()),
            "valign" => {
                self.valign = match value {
                    "top" => VAlign::Top,
                    "bottom" => VAlign::Bottom,
                    _ => VAlign::Middle,
                }
            }
            "columns" => self.columns = Some(ColumnFormat::from_str(value)),
            _ => {}
        }
    }
    pub fn update_style_attr(&self, style_attr: &mut StyleAttr) {
        if let Some(ref color) = self.bgcolor {
            style_attr.fill_color = Some(color.clone());
        }
        style_attr.valign = self.valign.clone();
        style_attr.align = self.align.clone();
    }
}
#[derive(Debug, Clone)]
pub struct FontTable {
    pub rows: Vec<(Row, Option<Hr>)>,
    pub tag: TableTag,
    pub table_attr: TableAttr,
}
#[derive(Debug, Clone)]
pub struct Vr {}

#[derive(Debug, Clone)]
pub struct Hr {}

#[derive(Debug, Clone)]
pub struct Row {
    pub cells: Vec<(DotCell, Option<Vr>)>,
}

#[derive(Debug, Clone)]
pub enum TextItem {
    TaggedText(TaggedText),
    Br(Align),
    PlainText(String),
}

#[derive(Debug, Clone)]
pub enum Html {
    Text(Text),
    FontTable(FontTable),
}

impl Html {
    // fn new_text() -> Self {

    // }
}

#[derive(Debug, Clone)]
pub enum HtmlGrid {
    Text(Text),
    FontTable(TableGrid),
}

impl HtmlGrid {
    pub fn size(&self, font_size: usize) -> Point {
        match self {
            HtmlGrid::Text(text) => {
                let mut size = Point::new(0.0, 0.0);
                for item in text.iter() {
                    match item {
                        TextItem::TaggedText(tagged_text) => {
                            let text_size =
                                get_tagged_text_size(tagged_text, font_size);
                            size.x += text_size.x;
                            size.y = size.y.max(text_size.y);
                        }
                        TextItem::Br(_) => {
                            size.x = 0.0;
                            size.y += font_size as f64;
                        }
                        TextItem::PlainText(text) => {
                            let text_size = get_size_for_str(text, font_size);
                            size.x += text_size.x;
                            size.y = size.y.max(text_size.y);
                        }
                    }
                }
                pad_shape_scalar(size, BOX_SHAPE_PADDING)
            }
            HtmlGrid::FontTable(table_grid) => table_grid.size(font_size),
        }
    }
    pub fn from_html(html: &Html) -> Self {
        match html {
            Html::Text(text) => HtmlGrid::Text(text.clone()),
            Html::FontTable(table) => {
                HtmlGrid::FontTable(TableGrid::from_table(table))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HtmlMode {
    Html,
    // HtmlText,
    HtmlTag,
}

// fn escape_new

#[derive(Debug, Clone)]
struct HtmlParser {
    input: Vec<char>,
    pos: usize,
    tok: Token,
    mode: HtmlMode,
    pub ch: char,
}

impl HtmlParser {
    pub fn next_token(&mut self) -> Token {
        match self.mode {
            HtmlMode::Html => self.read_html(),
            // HtmlMode::HtmlText => self.read_html_text(),
            HtmlMode::HtmlTag => self.read_tag_inside(),
        }
    }
    pub fn lex(&mut self) {
        match self.tok {
            Token::Error(_) => {
                panic!("can't parse after error");
            }
            Token::EOF => {
                panic!("can't parse after EOF");
            }
            _ => {
                // Lex the next token.
                self.tok = self.next_token();
            }
        }
    }
    pub fn skip_whitespace(&mut self) -> bool {
        let mut changed = false;
        while self.ch.is_ascii_whitespace() {
            self.read_char();
            changed = true;
        }
        changed
    }
    pub fn has_next(&self) -> bool {
        self.pos < self.input.len()
    }
    pub fn read_char(&mut self) {
        if !self.has_next() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.pos];
            self.pos += 1;
        }
    }

    pub fn read_tag_inside(&mut self) -> Token {
        let tok: Token;
        while self.skip_whitespace() {}
        if self.ch == '>' {
            self.read_char();
            self.mode = HtmlMode::Html;
            return Token::TagEnd;
        }
        if self.ch == '/' {
            self.read_char();
            if self.ch == '>' {
                self.read_char();
                self.mode = HtmlMode::Html;
                return Token::TagEndWithSlash;
            } else {
                return Token::Error(self.pos);
            }
        }
        tok = self.read_tag_attr();
        self.read_char();
        tok
    }
    pub fn read_html_text(&mut self) -> Token {
        let mut result = String::new();
        while self.ch != '<' && self.ch != '\0' && self.ch != '>' {
            result.push(self.ch);
            self.read_char();
            // escape new line
        }
        Token::Identifier(result)
    }
    pub fn read_html(&mut self) -> Token {
        let mut tag_name = String::new();
        if self.ch == '\0' {
            return Token::EOF;
        }

        if self.ch != '<' {
            return self.read_html_text();
        }
        self.read_char();

        if self.ch == '/' {
            self.read_char();
            while self.ch != '>' && self.ch != '\0' {
                tag_name.push(self.ch);
                self.read_char();
            }
            if self.ch == '\0' {
                return Token::Error(self.pos);
            }
            if tag_name.is_empty() {
                return Token::Error(self.pos);
            }
            self.mode = HtmlMode::Html;
            self.read_char();
            let tag_name = tag_name.to_lowercase();
            Token::ClosingTag(TagType::from_str(&tag_name))
        } else {
            while self.ch.is_alphabetic() {
                tag_name.push(self.ch);
                self.read_char();
            }
            if tag_name.is_empty() {
                return Token::Error(self.pos);
            }
            let tag_name = tag_name.to_lowercase();
            self.mode = HtmlMode::HtmlTag;
            Token::OpeningTag(TagType::from_str(&tag_name))
        }
    }
    pub fn read_string(&mut self) -> Token {
        let mut result = String::new();
        self.read_char();
        while self.ch != '"' {
            // Handle escaping
            if self.ch == '\\' {
                // Consume the escape character.
                self.read_char();
                self.ch = match self.ch {
                    'n' => '\n',
                    'l' => '\n',
                    _ => self.ch,
                }
            } else if self.ch == '\0' {
                // Reached EOF without completing the string
                return Token::Error(self.pos);
            }
            result.push(self.ch);
            self.read_char();
        }
        Token::Identifier(result)
    }
    pub fn read_tag_attr(&mut self) -> Token {
        let mut attr_name = String::new();
        while self.skip_whitespace() {}
        while self.ch != '=' && self.ch != '>' && self.ch != '\0' {
            // skip over whitespace
            if !self.ch.is_ascii_whitespace() {
                attr_name.push(self.ch);
            }
            self.read_char();
        }
        if self.ch != '=' {
            return Token::Error(self.pos);
        }
        self.read_char();
        while self.skip_whitespace() {}
        if self.ch != '"' {
            return Token::Error(self.pos);
        }
        // self.read_char();
        let x = self.read_string();
        if let Token::Identifier(s) = x {
            Token::TagAttr(attr_name.to_lowercase(), s.to_lowercase())
        } else {
            Token::Error(self.pos)
        }
    }
    // Parse HTML-like label content between < and >
    pub fn parse_html_label(&mut self) -> Result<Html, String> {
        let is_table = self.is_table()?;

        if is_table {
            Ok(Html::FontTable(self.parse_table()?))
        } else {
            Ok(Html::Text(self.parse_text()?))
        }
    }

    pub fn parse_text(&mut self) -> Result<Text, String> {
        let mut text_items = vec![];
        loop {
            match self.tok {
                Token::ClosingTag(_) | Token::EOF => {
                    break;
                }
                _ => {}
            }
            text_items.push(self.parse_text_item()?);
        }
        Ok(text_items)
    }

    pub fn is_table(&self) -> Result<bool, String> {
        // check if the current token is a table tag with a look ahead of distance 2
        // check if cloing is necessary
        let mut parser = self.clone();
        if let Token::Identifier(_) = parser.tok.clone() {
            // Consume the text.
            parser.lex();
        }

        if let Token::OpeningTag(TagType::Table) = parser.tok.clone() {
            // Consume the opening tag.
            return Ok(true);
        }

        match parser.tok.clone() {
            Token::OpeningTag(_) => {
                parser.mode = HtmlMode::HtmlTag;
                parser.parse_tag_start(true)?;
            }
            _ => {
                return Ok(false);
            }
        }

        if let Token::OpeningTag(TagType::Table) = parser.tok.clone() {
            // Consume the opening tag.
            return Ok(true);
        }

        Ok(false)
    }

    pub fn parse_text_item(&mut self) -> Result<TextItem, String> {
        Ok(match self.tok.clone() {
            Token::Identifier(x) => {
                self.lex();
                TextItem::PlainText(x)
            }
            Token::OpeningTag(x) => {
                self.mode = HtmlMode::HtmlTag;
                let (tag, tag_attr) = self.parse_tag_start(false)?;

                if tag.is_single_tag() {
                    match x {
                        TagType::Br => {
                            return Ok(TextItem::Br(
                                Align::from_tag_attr_list(tag_attr),
                            ));
                        }
                        _ => {}
                    }
                } else {
                    let text_items = self.parse_text()?;
                    self.parse_tag_end(&tag, false)?;
                    match x {
                        _ => {
                            return Ok(TextItem::TaggedText(TaggedText {
                                tag: TextTag::new(&tag, tag_attr),
                                text_items,
                            }))
                        }
                    }
                }

                return to_error(
                    format!(
                        "Expected closing tag for {:?}, found {:?}",
                        x, self.tok
                    )
                    .as_str(),
                );
            }
            _ => {
                return to_error(
                    format!(
                        "Expected identifier or tag opener, found {:?}",
                        self.tok
                    )
                    .as_str(),
                )
            }
        })
    }

    pub fn parse_table(&mut self) -> Result<FontTable, String> {
        let mut string_before = false;
        if let Token::Identifier(_) = self.tok.clone() {
            // Consume the text.
            self.lex();
            string_before = true;
        }
        let (tag1, table_attr1) = self.parse_tag_start(true)?;
        let (table_tag1, table_attr2) = match tag1 {
            TagType::Font
            | TagType::I
            | TagType::B
            | TagType::U
            | TagType::O => {
                let (tag, tag_attr) = self.parse_tag_start(true)?;
                if tag != TagType::Table {
                    return to_error(
                        format!("Expected <table>, found {:?}", tag).as_str(),
                    );
                }
                if string_before {
                    return to_error(
                        "Cannot inlcuding string before table tag",
                    );
                }
                (Some((tag1, table_attr1)), tag_attr)
            }
            TagType::Table => (None, table_attr1),
            _ => {
                return to_error(
                    format!("Expected <tr>, found {:?}", tag1).as_str(),
                );
            }
        };
        let mut rows = Vec::new();

        loop {
            if let Token::ClosingTag(_) = self.tok.clone() {
                break;
            }
            let row = self.parse_row()?;
            let row_split = if let Token::OpeningTag(TagType::Hr) =
                self.tok.clone()
            {
                let (tag_type, _) = self.parse_tag_start(true)?;
                if tag_type != TagType::Hr {
                    return to_error(
                        format!("Expected <vr>, found {:?}", tag_type).as_str(),
                    );
                }
                Some(Hr {})
            } else {
                None
            };
            rows.push((row, row_split));
        }

        self.parse_tag_end(&TagType::Table, true)?;
        if let Some(ref tag) = table_tag1 {
            self.parse_tag_end(&tag.0, true)?;
        }
        let table_attr = TableAttr::from_attr_list(table_attr2);

        Ok(FontTable {
            rows,
            tag: TableTag::from_tag(table_tag1),
            table_attr,
        })
    }

    pub fn parse_tag_attr_list(
        &mut self,
        tag_type: TagType,
    ) -> Result<Vec<(String, String)>, String> {
        let mut lst = Vec::new();
        loop {
            match self.tok {
                Token::TagEnd => {
                    if tag_type.is_single_tag() {
                        return to_error(format!("Tag {:?} is a pair attribe and should be closed ending tag", tag_type).as_str());
                    }
                    self.lex();
                    break;
                }
                Token::TagEndWithSlash => {
                    if !tag_type.is_single_tag() {
                        return to_error(format!("Tag {:?} is a single tag and should be closed with single tag", tag_type).as_str());
                    }
                    self.lex();
                    break;
                }
                _ => {}
            }
            let tag_attr = if let Token::TagAttr(attr, value) = self.tok.clone()
            {
                self.lex();
                (attr, value)
            } else {
                return to_error("wrong identifi inside able tag");
            };
            lst.push(tag_attr);
        }

        Ok(lst)
    }
    pub fn parse_row(&mut self) -> Result<Row, String> {
        let (tag_type, _attr_list) = self.parse_tag_start(true)?;
        if tag_type != TagType::Tr {
            return to_error(
                format!("Expected <tr>, found {:?}", tag_type).as_str(),
            );
        }
        // TODO: consume the 1st cell so that it gurannets the grammar property that splitter can appear in better
        // cells
        // The same for splitting of row
        let mut cells = Vec::new();
        loop {
            if let Token::ClosingTag(_) = self.tok.clone() {
                break;
            }
            let cell = self.parse_cell()?;
            let cell_split = if let Token::OpeningTag(TagType::Vr) =
                self.tok.clone()
            {
                let (tag_type, _) = self.parse_tag_start(true)?;
                if tag_type != TagType::Vr {
                    return to_error(
                        format!("Expected <vr>, found {:?}", tag_type).as_str(),
                    );
                }
                Some(Vr {})
            } else {
                None
            };
            cells.push((cell, cell_split));
        }
        self.parse_tag_end(&TagType::Tr, true)?;
        Ok(Row { cells })
    }

    pub fn parse_cell(&mut self) -> Result<DotCell, String> {
        let (tag_type, attr_list) = self.parse_tag_start(false)?;
        if tag_type != TagType::Td {
            return to_error(
                format!("Expected <td>, found {:?}", tag_type).as_str(),
            );
        }
        let label = LabelOrImg::Html(self.parse_html_label()?);
        self.parse_tag_end(&TagType::Td, true)?;
        Ok(DotCell {
            label,
            td_attr: TdAttr::from_tag_attr_list(attr_list),
        })
    }

    pub fn parse_tag_start(
        &mut self,
        pass_identifier: bool,
    ) -> Result<(TagType, Vec<(String, String)>), String> {
        let tag_type = if let Token::OpeningTag(x) = self.tok.clone() {
            self.mode = HtmlMode::HtmlTag;
            self.lex();
            x
        } else {
            return to_error(
                format!(
                    "Expected opening tag to start HTML label tag, found {:?}",
                    self.tok
                )
                .as_str(),
            );
        };
        let tag_attr_list = self.parse_tag_attr_list(tag_type.clone())?;
        match tag_type {
            TagType::Br | TagType::Sub | TagType::Sup | TagType::S => {
                // self.lexer.mode = super::lexer::HtmlMode::Html;
            }
            TagType::Hr
            | TagType::Tr
            | TagType::Td
            | TagType::Table
            | TagType::Img
            | TagType::Vr
            | TagType::Font
            | TagType::I
            | TagType::B
            | TagType::U
            | TagType::O => {
                if pass_identifier {
                    if let Token::Identifier(_) = self.tok.clone() {
                        self.lex();
                    }
                }
            }
            TagType::Unrecognized => {
                return to_error(
                    format!("Unrecognized tag type {:?}", tag_type).as_str(),
                );
            }
        }
        Ok((tag_type, tag_attr_list))
    }

    pub fn parse_tag_end(
        &mut self,
        tag: &TagType,
        pass_identifier: bool,
    ) -> Result<(), String> {
        if let Token::ClosingTag(x) = self.tok.clone() {
            // self.lexer.mode = super::lexer::HtmlMode::HtmlText;
            if x == *tag {
                self.lex();
            } else {
                return to_error(
                    format!(
                        "Expected {:?} to end HTML label tag, found {:?}",
                        tag, x
                    )
                    .as_str(),
                );
            }
        } else {
            return to_error(format!("Expected 'closing tag {:?}' to end HTML label tag, found {:?}", tag, self.tok).as_str());
        }
        if pass_identifier {
            if let Token::Identifier(_) = self.tok.clone() {
                self.lex();
            }
        }

        Ok(())
    }
}

pub fn parse_html_string(input: &str) -> Result<HtmlGrid, String> {
    let mut parser = HtmlParser {
        input: input.chars().collect(),
        pos: 0,
        tok: Token::Colon,
        mode: HtmlMode::Html,
        ch: '\0',
    };
    parser.read_char();
    parser.lex();
    let x = parser.parse_html_label()?;
    Ok(HtmlGrid::from_html(&x))
}

#[derive(Debug, Clone)]
struct TableGridInner {
    pub(crate) cells: Vec<(TdAttr, DotCellGrid)>,
    pub(crate) occupation: HashMap<(usize, usize), usize>, // x, y, cell index
}

impl TableGridInner {
    pub fn width(&self) -> usize {
        self.occupation.keys().map(|(x, _)| *x).max().unwrap_or(0) + 1
    }
    pub fn height(&self) -> usize {
        self.occupation.keys().map(|(_, y)| *y).max().unwrap_or(0) + 1
    }
    pub fn pretty_print(&self) {
        // print in a table format with + indicating occupied and - indicating free
        let width = self.width();
        let height = self.height();
        let mut table = vec![vec!['-'; width]; height];
        for (x, y) in self.occupation.keys() {
            table[*y][*x] = '+';
        }
        for y in 0..height {
            for x in 0..width {
                print!("{}", table[y][x]);
            }
            println!();
        }
    }
    pub fn add_cell(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        td_attr: TdAttr,
        dot_cell: DotCell,
    ) {
        self.cells.push((
            td_attr,
            DotCellGrid::from_dot_cell(x, y, width, height, &dot_cell),
        ));
        // boundaries are desinged with respect to html specs for forming table algo
        for i in x..(x + width) {
            for j in y..(y + height) {
                let x = self.occupation.insert((i, j), self.cells.len() - 1);
                if x.is_some() {
                    panic!("Cell already occupied at ({}, {})", i, j);
                }
            }
        }
    }
    fn is_occupied(&self, x: usize, y: usize) -> bool {
        self.occupation.contains_key(&(x, y))
    }

    pub fn from_table(font_table: &FontTable) -> Self {
        let mut width = 0;
        let mut height = 0;
        let mut y_current = 0;
        let mut table_grid = Self {
            cells: Vec::new(),
            occupation: HashMap::new(),
        };
        for row in &font_table.rows {
            table_grid.process_row(
                &row.0,
                &mut width,
                &mut height,
                &mut y_current,
            );
        }

        // ending part
        table_grid.pretty_print();
        table_grid
    }

    fn process_row(
        &mut self,
        row: &Row,
        width: &mut usize,
        height: &mut usize,
        y_current: &mut usize,
    ) {
        if height == y_current {
            *height += 1;
        }
        let mut x_current = 0;
        for c in &row.cells {
            let cell = &c.0;
            let colspan = cell.td_attr.colspan as usize;
            let rowspan = cell.td_attr.rowspan as usize;
            while x_current < *width && self.is_occupied(x_current, *y_current)
            {
                x_current += 1;
            }
            if x_current == *width {
                *width += 1;
            }
            if *width < x_current + colspan {
                *width = x_current + colspan;
            }
            if *height < *y_current + rowspan {
                *height = *y_current + rowspan;
            }

            self.add_cell(
                x_current,
                *y_current,
                colspan,
                rowspan,
                cell.td_attr.clone(),
                cell.clone(),
            );
            x_current += colspan;
        }
        *y_current += 1;
    }
}

#[derive(Debug, Clone)]
pub struct TableGrid {
    pub(crate) cells: Vec<(TdAttr, DotCellGrid)>,
    pub(crate) grid: Vec<Option<usize>>,
    pub(crate) width_arr: Vec<f64>, // width in svg units
    pub(crate) height_arr: Vec<f64>, // height in svg units
    width_in_cell: usize,           // width of the table in cells
    height_in_cell: usize,          // height of the table in cells
    font_size: usize,
    pub(crate) table_attr: TableAttr,
}

impl TableGrid {
    pub fn width(&self) -> f64 {
        self.width_arr.iter().sum::<f64>()
            + (self.table_attr.cellspacing as usize * (self.width_in_cell + 1))
                as f64
            + self.table_attr.border as f64 * 2.
    }
    pub fn height(&self) -> f64 {
        self.height_arr.iter().sum::<f64>()
            + (self.table_attr.cellspacing as usize * (self.height_in_cell + 1))
                as f64
            + self.table_attr.border as f64 * 2.
    }
    pub fn size(&self, font_size: usize) -> Point {
        if font_size != self.font_size {
            let mut table_grid = self.clone();
            table_grid.resize(font_size);
            Point::new(table_grid.width(), table_grid.height())
        } else {
            Point::new(self.width(), self.height())
        }
    }
    pub fn cell_pos(&self, d: &DotCellGrid) -> Point {
        let idx = d.i;
        let x = self.width_arr.iter().take(idx).sum::<f64>()
            + (self.table_attr.cellspacing as usize * (idx + 1)) as f64
            + self.table_attr.border as f64 / 2.0;

        let idx = d.j;

        let y = self.height_arr.iter().take(idx).sum::<f64>()
            + (self.table_attr.cellspacing as usize * (idx + 1)) as f64
            + self.table_attr.border as f64 / 2.0;

        Point::new(x, y)
    }
    pub fn cell_size(&self, dot_cell_grid: &DotCellGrid) -> Point {
        let mut height = 0f64;
        for i in dot_cell_grid.j..(dot_cell_grid.j + dot_cell_grid.height) {
            height += self.height_arr[i];
        }
        height += self.table_attr.cellspacing as f64
            * (dot_cell_grid.height as f64 - 1.);

        let mut width = 0f64;
        for i in dot_cell_grid.i..(dot_cell_grid.i + dot_cell_grid.width) {
            width += self.width_arr[i];
        }
        width += self.table_attr.cellspacing as f64
            * (dot_cell_grid.width as f64 - 1.);

        Point::new(width, height)
    }

    pub fn from_table(font_table: &FontTable) -> Self {
        let table_grid_inner = TableGridInner::from_table(font_table);
        let width_in_cell = table_grid_inner.width();
        let height_in_cell = table_grid_inner.height();
        let mut grid = vec![None; width_in_cell * height_in_cell];
        for (idx, (_td_attr, dot_cell)) in
            table_grid_inner.cells.iter().enumerate()
        {
            for i in 0..dot_cell.width {
                for j in 0..dot_cell.height {
                    let x_cur = dot_cell.i + i;
                    let y_cur = dot_cell.j + j;
                    grid[(y_cur * width_in_cell) + x_cur] = Some(idx);
                }
            }
        }

        Self {
            cells: table_grid_inner.cells,
            grid,
            width_arr: vec![1.0; width_in_cell],
            height_arr: vec![1.0; height_in_cell],
            width_in_cell,
            height_in_cell,
            font_size: 0,
            table_attr: font_table.table_attr.clone(),
        }
    }

    pub fn get_cell(&self, i: usize, j: usize) -> Option<&DotCellGrid> {
        if i < self.width_in_cell && j < self.height_in_cell {
            let index = self.grid[(j * (self.width_in_cell)) + i];
            if let Some(i) = index {
                return Some(&self.cells[i].1);
            }
        }
        None
    }

    pub fn get_cell_mut(
        &mut self,
        i: usize,
        j: usize,
    ) -> Option<&mut DotCellGrid> {
        if i < self.width_in_cell && j < self.height_in_cell {
            let index = self.grid[(j * (self.width_in_cell)) + i];
            if let Some(i) = index {
                return Some(&mut self.cells[i].1);
            }
        }
        None
    }

    pub(crate) fn cellpadding(&self, d: &DotCellGrid) -> f64 {
        let cellpadding = if let Some(td_cellpadding) = d.td_attr.cellpadding {
            td_cellpadding
        } else {
            self.table_attr.cellpadding
        } as f64;

        cellpadding
    }

    pub(crate) fn cellborder(&self, d: &DotCellGrid) -> f64 {
        let cellborder = if let Some(td_cellborder) = d.td_attr.border {
            td_cellborder
        } else if let Some(td_cellborder) = self.table_attr.cellborder {
            td_cellborder
        } else {
            self.table_attr.border
        } as f64;

        cellborder
    }

    pub fn resize(&mut self, font_size: usize) {
        // TODO: can check if font size is updated
        for x in 0..self.width_in_cell {
            let mut max_width = 0f64;
            for y in 0..self.height_in_cell {
                if let Some(cell) = self.get_cell_mut(x, y) {
                    match &mut cell.label_grid {
                        LabelOrImgGrid::Html(HtmlGrid::FontTable(x)) => {
                            x.resize(font_size);
                        }
                        _ => {}
                    }
                }
            }
            for y in 0..self.height_in_cell {
                if let Some(cell) = self.get_cell(x, y) {
                    let w = match &cell.label_grid {
                        LabelOrImgGrid::Html(html) => match html {
                            HtmlGrid::Text(text) => {
                                let mut size = Point::zero();
                                for text_item in text {
                                    let item_size = get_text_item_size(
                                        text_item, font_size,
                                    );
                                    size = size.add(item_size);
                                }
                                size.x
                            }
                            HtmlGrid::FontTable(x) => x.width(),
                        },
                        _ => 0.0,
                    };
                    let cellpadding = self.cellpadding(cell);
                    let cellborder = self.cellborder(cell);

                    let w = w + cellborder * 2.0 + cellpadding * 2.0;

                    max_width = max_width.max(w / cell.width as f64);
                }
            }

            self.width_arr[x] = max_width;
        }

        for y in 0..self.height_in_cell {
            let mut max_height = 0f64;
            for x in 0..self.width_in_cell {
                if let Some(cell) = self.get_cell(x, y) {
                    let h = match &cell.label_grid {
                        LabelOrImgGrid::Html(html) => match html {
                            HtmlGrid::Text(text) => {
                                let mut size = Point::zero();
                                for text_item in text {
                                    let item_size = get_text_item_size(
                                        text_item, font_size,
                                    );
                                    size = size.add(item_size);
                                }
                                size.y as f64
                            }
                            HtmlGrid::FontTable(x) => x.height(),
                        },
                        _ => 0.0,
                    };
                    let cellpadding = self.cellpadding(cell);
                    let cellborder = self.cellborder(cell);

                    let h = h + cellborder * 2.0 + cellpadding * 2.0;

                    max_height = max_height.max(h / cell.height as f64);
                }
            }
            self.height_arr[y] = max_height;
        }

        // update the font size
        self.font_size = font_size;
    }
}

pub(crate) fn get_text_item_size(item: &TextItem, font_size: usize) -> Point {
    match item {
        TextItem::Br(_) => Point::new(1.0, 1.0),
        TextItem::PlainText(text) => {
            let size = get_size_for_str(text, font_size);
            pad_shape_scalar(size, BOX_SHAPE_PADDING)
        }
        TextItem::TaggedText(tagged_text) => {
            get_tagged_text_size(tagged_text, font_size)
        }
    }
}

fn get_tagged_text_size(tagged_text: &TaggedText, font_size: usize) -> Point {
    let mut size = Point::zero();
    for text_item in &tagged_text.text_items {
        let item_size = get_text_item_size(text_item, font_size);
        size = size.add(item_size);
    }
    size
}
