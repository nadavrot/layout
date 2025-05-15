//! Implements the drawing of elements and arrows on the backing canvas.

use crate::core::base::Orientation;
use crate::core::format::{ClipHandle, RenderBackend, Renderable, Visible};
use crate::core::geometry::*;
use crate::core::style::{
    Align, BaselineShift, FontStyle, FontWeight, LineStyleKind, StyleAttr,
    VAlign,
};
use crate::gv::html::{
    DotCellGrid, HtmlGrid, LabelOrImgGrid, TableGrid, Text, TextItem, TextTag,
};
use crate::std_shapes::shapes::*;

/// Return the height and width of the record, depending on the geometry and
/// internal text.
fn get_record_size(
    rec: &RecordDef,
    dir: Orientation,
    font_size: usize,
) -> Point {
    match rec {
        RecordDef::Text(label, _) => pad_shape_scalar(
            get_size_for_str(label, font_size),
            BOX_SHAPE_PADDING,
        ),
        RecordDef::Array(arr) => {
            let mut x: f64 = 0.;
            let mut y: f64 = 0.;
            for elem in arr {
                let ret = get_record_size(elem, dir.flip(), font_size);
                if dir.is_left_right() {
                    x += ret.x;
                    y = y.max(ret.y);
                } else {
                    x = x.max(ret.x);
                    y += ret.y;
                }
            }
            Point::new(x, y)
        }
    }
}

pub(crate) const BOX_SHAPE_PADDING: f64 = 10.;
const CIRCLE_SHAPE_PADDING: f64 = 20.;

/// Return the size of the shape. If \p make_xy_same is set then make the
/// X and the Y of the shape the same. This will turn ellipses into circles and
/// rectangles into boxes. The parameter \p dir specifies the direction of the
/// graph. This tells us if we need to draw records left to right or top down.
pub fn get_shape_size(
    dir: Orientation,
    s: &ShapeKind,
    font: usize,
    make_xy_same: bool,
) -> Point {
    let mut res = match s {
        ShapeKind::Box(text) => {
            pad_shape_scalar(get_size_for_str(text, font), BOX_SHAPE_PADDING)
        }
        ShapeKind::Circle(text) => {
            pad_shape_scalar(get_size_for_str(text, font), CIRCLE_SHAPE_PADDING)
        }
        ShapeKind::DoubleCircle(text) => {
            pad_shape_scalar(get_size_for_str(text, font), CIRCLE_SHAPE_PADDING)
        }
        ShapeKind::Record(sr) => {
            pad_shape_scalar(get_record_size(sr, dir, font), BOX_SHAPE_PADDING)
        }
        ShapeKind::Connector(text) => {
            if let Option::Some(text) = text {
                pad_shape_scalar(
                    get_size_for_str(text, font),
                    BOX_SHAPE_PADDING,
                )
            } else {
                Point::new(1., 1.)
            }
        }
        ShapeKind::Html(html_grid) => html_grid.size(font),
        ShapeKind::None => Point::new(1., 1.),
    };
    if make_xy_same {
        res = make_size_square(res);
    }
    res
}

// Returns the innermost shape that the record describes, or the location and
// size of the outer shape.
fn get_record_port_location(
    rec: &RecordDef,
    dir: Orientation,
    loc: Point,
    size: Point,
    look: &StyleAttr,
    port_name: &str,
) -> (Point, Point) {
    struct Locator {
        port_name: String,
        loc: Point,
        size: Point,
    }

    impl RecordVisitor for Locator {
        fn handle_box(&mut self, _loc: Point, _size: Point) {}
        fn handle_text(
            &mut self,
            loc: Point,
            size: Point,
            _label: &str,
            port: &Option<String>,
        ) {
            if let Option::Some(port_name) = port {
                if *port_name == self.port_name {
                    self.loc = loc;
                    self.size = size;
                }
            }
        }
    }

    let mut visitor = Locator {
        port_name: port_name.to_string(),
        loc,
        size,
    };
    visit_record(rec, dir, loc, size, look, &mut visitor);
    (visitor.loc, visitor.size)
}
struct Locator {
    port_name: String,
    loc: Point,
    size: Point,
}
fn get_html_port_location(
    html: &HtmlGrid,
    loc: Point,
    size: Point,
    // look: &StyleAttr,
    visitor: &mut Locator,
) -> (Point, Point) {
    match html {
        HtmlGrid::Text(_text) => {}
        HtmlGrid::FontTable(table) => {
            get_table_port_location(table, loc, size, visitor)
        }
    }
    (visitor.loc, visitor.size)
}

fn get_table_port_location(
    table: &TableGrid,
    loc: Point,
    _size: Point,
    visitor: &mut Locator,
) {
    if let Some(ref port_name) = table.table_attr.port {
        if port_name == &visitor.port_name {
            visitor.loc = loc;
            visitor.size = Point::new(table.width(), table.height());
        }
    }
    let table_width = table.width();
    let table_height = table.height();
    for (td_attr, c) in table.cells.iter() {
        let cell_size = table.cell_size(c);
        let cell_origin = table.cell_pos(c);
        let cell_loc = Point::new(
            visitor.loc.x + cell_origin.x + cell_size.x * 0.5
                - table_width * 0.5,
            visitor.loc.y + cell_origin.y + cell_size.y * 0.5
                - table_height * 0.5,
        );
        if let Option::Some(ref port_name) = td_attr.port {
            if port_name == &visitor.port_name {
                visitor.loc = cell_loc;
                visitor.size = cell_size;
            }
        }

        get_cell_port_location(&c, cell_loc, cell_size, visitor);
    }
}

fn get_cell_port_location(
    rec: &DotCellGrid,
    loc: Point,
    size: Point,
    visitor: &mut Locator,
) {
    if let LabelOrImgGrid::Html(html) = &rec.label_grid {
        get_html_port_location(html, loc, size, visitor);
    }
}

fn render_record(
    rec: &RecordDef,
    dir: Orientation,
    loc: Point,
    size: Point,
    look: &StyleAttr,
    canvas: &mut dyn RenderBackend,
) {
    struct Renderer<'a> {
        look: StyleAttr,
        clip_handle: Option<ClipHandle>,
        canvas: &'a mut dyn RenderBackend,
    }

    // A reference to the clip region.
    let mut clip_handle: Option<ClipHandle> = Option::None;

    if look.rounded > 0 {
        let xy = Point::new(loc.x - size.x / 2., loc.y - size.y / 2.);
        let ch = canvas.create_clip(xy, size, 15);
        clip_handle = Option::Some(ch);
    }

    impl<'a> RecordVisitor for Renderer<'a> {
        fn handle_box(&mut self, loc: Point, size: Point) {
            self.canvas.draw_rect(
                Point::new(loc.x - size.x / 2., loc.y - size.y / 2.),
                Point::new(size.x, size.y),
                &self.look,
                Option::None,
                self.clip_handle,
            );
        }
        fn handle_text(
            &mut self,
            loc: Point,
            _size: Point,
            label: &str,
            _port: &Option<String>,
        ) {
            self.canvas.draw_text(loc, label, &self.look);
        }
    }

    let mut visitor = Renderer {
        look: look.clone(),
        clip_handle,
        canvas,
    };
    // Make the internal record boxes square and not round.
    visitor.look.rounded = 0;
    visit_record(rec, dir, loc, size, look, &mut visitor);

    let mut look = look.clone();
    look.fill_color = Option::None;
    canvas.draw_rect(
        Point::new(loc.x - size.x / 2., loc.y - size.y / 2.),
        Point::new(size.x, size.y),
        &look,
        Option::None,
        Option::None,
    );
}

fn render_html(
    rec: &HtmlGrid,
    _dir: Orientation,
    loc: Point,
    size: Point,
    look: &StyleAttr,
    canvas: &mut dyn RenderBackend,
) {
    match rec {
        HtmlGrid::Text(text) => {
            render_text(
                text,
                loc,
                size,
                look,
                canvas,
                Option::None,
                Option::None,
            );
        }
        HtmlGrid::FontTable(table) => {
            render_font_table(
                table,
                loc,
                look,
                canvas,
                Option::None,
                Option::None,
            );
        }
    }
}

fn update_location(
    loc: Point,
    size: Point,
    text: &str,
    look: &StyleAttr,
) -> Point {
    let mut loc = loc;
    let text_size = get_size_for_str(text, look.font_size);
    let displacement = size.sub(text_size);
    match look.align {
        Align::Left => {
            loc.x -= displacement.x / 2.;
        }
        Align::Right => {
            loc.x += displacement.x / 2.;
        }
        Align::Center => {}
    }
    match look.valign {
        VAlign::Top => {
            loc.y -= displacement.y / 2.;
        }
        VAlign::Bottom => {
            loc.y += displacement.y / 2.;
        }
        VAlign::Middle => {}
    }
    loc
}

fn render_text(
    rec: &Text,
    loc: Point,
    size: Point,
    look: &StyleAttr,
    canvas: &mut dyn RenderBackend,
    clip_handle: Option<ClipHandle>,
    _clip: Option<ClipHandle>,
) {
    let mut loc = loc;
    loc.x -= size.x / 2.;
    for item in rec {
        match item {
            TextItem::Br(_) => {
                loc.y += look.font_size as f64;
            }
            TextItem::PlainText(text) => {
                let size_str = get_size_for_str(text, look.font_size);
                let look = look.clone();
                loc.x += size_str.x / 2.;
                let loc2 = update_location(loc, size, text, &look);
                canvas.draw_text(loc2, text.as_str(), &look);
                loc.x += size_str.x / 2.;
            }
            TextItem::TaggedText(tagged_text) => {
                let mut look = look.clone();
                match &tagged_text.tag {
                    TextTag::B => {
                        look.font_weight = FontWeight::Bold;
                    }
                    TextTag::I => {
                        look.font_style = FontStyle::Italic;
                    }
                    TextTag::U => {
                        look.text_decoration.underline = true;
                    }
                    TextTag::S => {
                        look.text_decoration.line_through = true;
                    }
                    TextTag::Sub => {
                        look.baseline_shift = BaselineShift::Sub;
                    }
                    TextTag::Font(font) => {
                        // look.font_size = font.point_size;
                        if let Some(point_size) = font.point_size {
                            look.font_size = point_size as usize;
                        }
                        if let Some(font_color) = font.color {
                            look.font_color = font_color;
                        }
                        if let Some(ref font_name) = font.face {
                            look.fontname = font_name.clone();
                        }
                    }
                    TextTag::O => {
                        look.text_decoration.overline = true;
                    }
                    TextTag::Sup => {
                        look.baseline_shift = BaselineShift::Super;
                    }
                }
                let mut loc3 = loc;
                loc3.x += size.x / 2.;
                render_text(
                    &tagged_text.text_items,
                    loc3,
                    size,
                    &look,
                    canvas,
                    clip_handle,
                    Option::None,
                );
            }
        }
    }
}

fn render_font_table(
    rec: &TableGrid,
    loc: Point,
    look: &StyleAttr,
    canvas: &mut dyn RenderBackend,
    clip_handle: Option<ClipHandle>,
    _clip: Option<ClipHandle>,
) {
    let mut look = look.clone();
    look.line_width = rec.table_attr.border as usize;

    rec.table_attr.update_style_attr(&mut look);
    let table_grid_width = rec.width();
    let table_grid_height = rec.height();
    let loc0 = Point::new(
        loc.x - table_grid_width / 2.,
        loc.y - table_grid_height / 2.,
    );
    canvas.draw_rect(
        loc0,
        Point::new(
            table_grid_width - rec.table_attr.border as f64,
            table_grid_height - rec.table_attr.border as f64,
        ),
        &look,
        Option::None,
        clip_handle,
    );

    for (td_attr, c) in rec.cells.iter() {
        // let cellpadding = rec.cellpadding(c);
        let cellborder = rec.cellborder(c);
        let mut look = look.clone();

        td_attr.update_style_attr(&mut look);

        let cell_size = rec.cell_size(c);
        let cell_origin = rec.cell_pos(c);
        let cell_loc = Point::new(
            loc0.x + cell_origin.x + cell_size.x * 0.5,
            loc0.y + cell_origin.y + cell_size.y * 0.5,
        );
        let mut look_border = look.clone();
        look_border.line_width = cellborder as usize;

        canvas.draw_rect(
            Point::new(
                loc0.x + cell_origin.x + look_border.line_width as f64 * 0.5,
                loc0.y + cell_origin.y + look_border.line_width as f64 * 0.5,
            ),
            cell_size.sub(Point::splat(cellborder)),
            &look_border,
            Option::None,
            clip_handle,
        );
        render_cell(&c, cell_loc, c.size(look.font_size), &look, canvas);
    }
}

fn render_cell(
    rec: &DotCellGrid,
    loc: Point,
    size: Point,
    look: &StyleAttr,
    canvas: &mut dyn RenderBackend,
) {
    match &rec.label_grid {
        LabelOrImgGrid::Html(html) => {
            render_html(
                html,
                Orientation::LeftToRight,
                loc,
                size,
                look,
                canvas,
            );
        }
        LabelOrImgGrid::Img(_, _) => {}
    }
}

pub trait RecordVisitor {
    fn handle_box(&mut self, loc: Point, size: Point);
    fn handle_text(
        &mut self,
        loc: Point,
        size: Point,
        label: &str,
        port: &Option<String>,
    );
}

fn visit_record(
    rec: &RecordDef,
    dir: Orientation,
    loc: Point,
    size: Point,
    look: &StyleAttr,
    visitor: &mut dyn RecordVisitor,
) {
    visitor.handle_box(loc, size);
    match rec {
        RecordDef::Text(text, port) => {
            visitor.handle_text(loc, size, text, port);
        }
        RecordDef::Array(arr) => {
            let mut sizes: Vec<Point> = Vec::new();
            let mut sum = Point::zero();
            let mut mx = Point::zero();
            // Figure out the recursive size of each element, and the largest
            // element.
            for elem in arr {
                let sz = get_record_size(elem, dir, look.font_size);
                sizes.push(sz);
                sum = Point::new(sum.x + sz.x, sum.y + sz.y);
                mx = Point::new(mx.x.max(sz.x), mx.y.max(sz.y));
            }
            // Normalize the size of each element on the x axis, and the maximum
            // width of the y axis to render something like: [...][..][.][...]
            for sz in &mut sizes {
                if dir.is_left_right() {
                    *sz = Point::new(size.x * sz.x / sum.x, size.y);
                } else {
                    *sz = Point::new(size.x, size.y * sz.y / sum.y);
                }
            }

            if dir.is_left_right() {
                // Start placing blocks from the left edge of the box.
                // Use the edge as reference point.
                let mut startx = loc.x - size.x / 2.;
                for i in 0..sizes.len() {
                    let element = &arr[i];
                    let loc2 = Point::new(startx + sizes[i].x / 2., loc.y);
                    visit_record(
                        element,
                        dir.flip(),
                        loc2,
                        sizes[i],
                        look,
                        visitor,
                    );
                    startx += sizes[i].x;
                }
            } else {
                // Start placing blocks from the top edge of the box.
                // Use the edge as reference point.
                let mut starty = loc.y - size.y / 2.;
                for i in 0..sizes.len() {
                    let element = &arr[i];
                    let loc2 = Point::new(loc.x, starty + sizes[i].y / 2.);
                    visit_record(
                        element,
                        dir.flip(),
                        loc2,
                        sizes[i],
                        look,
                        visitor,
                    );
                    starty += sizes[i].y;
                }
            }
        }
    }
}

impl Renderable for Element {
    fn render(&self, debug: bool, canvas: &mut dyn RenderBackend) {
        if debug {
            // Draw the pink bounding box.
            let debug_look = StyleAttr::debug0();
            let bb = self.pos.bbox(true);
            canvas.draw_rect(
                bb.0,
                self.pos.size(true),
                &debug_look,
                self.properties.clone(),
                Option::None,
            );
        }

        match &self.shape {
            ShapeKind::None => {}
            ShapeKind::Record(rec) => {
                render_record(
                    rec,
                    self.orientation,
                    self.pos.center(),
                    self.pos.size(false),
                    &self.look,
                    canvas,
                );
            }
            ShapeKind::Html(rec) => render_html(
                rec,
                self.orientation,
                self.pos.center(),
                self.pos.size(false),
                &self.look,
                canvas,
            ),
            ShapeKind::Box(text) => {
                canvas.draw_rect(
                    self.pos.bbox(false).0,
                    self.pos.size(false),
                    &self.look,
                    self.properties.clone(),
                    Option::None,
                );
                canvas.draw_text(self.pos.center(), text.as_str(), &self.look);
            }
            ShapeKind::Circle(text) => {
                canvas.draw_circle(
                    self.pos.center(),
                    self.pos.size(false),
                    &self.look,
                    self.properties.clone(),
                );
                canvas.draw_text(self.pos.center(), text.as_str(), &self.look);
            }
            ShapeKind::DoubleCircle(text) => {
                canvas.draw_circle(
                    self.pos.center(),
                    self.pos.size(false),
                    &self.look,
                    self.properties.clone(),
                );
                let outer_circle_style = {
                    let mut x = self.look.clone();
                    x.fill_color = None;
                    x
                };
                canvas.draw_circle(
                    self.pos.center(),
                    self.pos.size(false).add(Point::splat(8.)),
                    &outer_circle_style,
                    None,
                );
                canvas.draw_text(self.pos.center(), text.as_str(), &self.look);
            }
            ShapeKind::Connector(label) => {
                if debug {
                    canvas.draw_rect(
                        self.pos.bbox(true).0,
                        self.pos.size(true),
                        &StyleAttr::debug0(),
                        Option::None,
                        Option::None,
                    );

                    canvas.draw_rect(
                        self.pos.bbox(false).0,
                        self.pos.size(false),
                        &StyleAttr::debug1(),
                        Option::None,
                        Option::None,
                    );
                }
                if let Option::Some(label) = label {
                    canvas.draw_text(self.pos.middle(), label, &self.look);
                }
            }
        }
        if debug {
            canvas.draw_circle(
                self.pos.center(),
                Point::new(6., 6.),
                &StyleAttr::debug2(),
                Option::None,
            );
        }
    }

    fn get_connector_location(
        &self,
        from: Point,
        force: f64,
        port: &Option<String>,
    ) -> (Point, Point) {
        match &self.shape {
            ShapeKind::None => (Point::zero(), Point::zero()),
            ShapeKind::Record(rec) => {
                let mut loc = self.pos.center();
                let mut size = self.pos.size(false);
                // Find the region that represents the inner box in the record.
                if let Option::Some(port_name) = port {
                    let r = get_record_port_location(
                        rec,
                        self.orientation,
                        loc,
                        size,
                        &self.look,
                        port_name,
                    );
                    loc = r.0;
                    size = r.1;
                }

                get_connection_point_for_box(loc, size, from, force)
            }
            ShapeKind::Box(_) => {
                let loc = self.pos.center();
                let size = self.pos.size(false);
                get_connection_point_for_box(loc, size, from, force)
            }
            ShapeKind::Circle(_) => {
                let loc = self.pos.center();
                let size = self.pos.size(false);
                get_connection_point_for_circle(loc, size, from, force)
            }
            ShapeKind::DoubleCircle(_) => {
                let loc = self.pos.center();
                let size = self.pos.size(false);
                get_connection_point_for_circle(loc, size, from, force)
            }
            ShapeKind::Html(html) => {
                let mut loc = self.pos.center();
                let mut size = self.pos.size(false);
                if let Some(port_name) = port {
                    let mut visitor = Locator {
                        port_name: port_name.to_string(),
                        loc,
                        size,
                    };
                    let r =
                        get_html_port_location(html, loc, size, &mut visitor);
                    loc = r.0;
                    size = r.1;
                }
                get_connection_point_for_box(loc, size, from, force)
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn get_passthrough_path(
        &self,
        _from: Point,
        _to: Point,
        _force: f64,
    ) -> (Point, Point) {
        let loc = self.pos.center();
        let size = self.pos.size(false);
        if let ShapeKind::Connector(_) = self.shape {
            get_passthrough_path_invisible(size, loc, _from, _to, _force)
        } else {
            panic!("We don't pass edges through this kind of shape");
        }
    }
}

pub fn generate_curve_for_elements(
    elements: &[Element],
    arrow: &Arrow,
    force: f64,
) -> Vec<(Point, Point)> {
    let mut path: Vec<(Point, Point)> = Vec::new();
    let to_loc = elements[1].position().center();
    let from_con =
        elements[0].get_connector_location(to_loc, force, &arrow.src_port);

    let mut prev_exit_loc = from_con.0;

    path.push((from_con.0, from_con.1));

    for i in 1..elements.len() {
        let to_con;
        let is_last: bool = i == elements.len() - 1;

        if is_last {
            let to = &elements[i];
            to_con = to.get_connector_location(
                prev_exit_loc,
                force,
                &arrow.dst_port,
            );
            prev_exit_loc = to_con.0;
        } else {
            let center = &elements[i];
            let to = &elements[i + 1];
            let to_loc = to.position().center();
            to_con = center.get_passthrough_path(prev_exit_loc, to_loc, force);
            prev_exit_loc = to_con.0;
        }

        path.push((to_con.1, to_con.0));
    }

    path
}

pub fn render_arrow(
    canvas: &mut dyn RenderBackend,
    debug: bool,
    elements: &[Element],
    arrow: &Arrow,
) {
    let path = generate_curve_for_elements(elements, arrow, 30.);

    if debug {
        for seg in &path {
            canvas.draw_line(seg.0, seg.1, &StyleAttr::debug2(), Option::None);
            canvas.draw_circle(
                seg.0,
                Point::new(6., 6.),
                &StyleAttr::debug1(),
                Option::None,
            );
            canvas.draw_circle(
                seg.1,
                Point::new(6., 6.),
                &StyleAttr::debug1(),
                Option::None,
            );
        }
    }

    let dash = match arrow.line_style {
        LineStyleKind::None => {
            return;
        }
        LineStyleKind::Normal => false,
        LineStyleKind::Dashed => true,
        LineStyleKind::Dotted => true,
    };

    let start = matches!(arrow.start, LineEndKind::Arrow);
    let end = matches!(arrow.end, LineEndKind::Arrow);

    canvas.draw_arrow(
        &path,
        dash,
        (start, end),
        &arrow.look,
        arrow.properties.clone(),
        &arrow.text,
    );
}
