//! Implements the drawing of elements and arrows on the backing canvas.

use crate::core::base::Orientation;
use crate::core::format::{ClipHandle, RenderBackend, Renderable, Visible};
use crate::core::geometry::*;
use crate::core::style::{Align, LineStyleKind, StyleAttr, VAlign};
use crate::gv::html::{
    get_line_height, DotCellGrid, HtmlGrid, LabelOrImgGrid, Scale, TableGrid,
    TextGrid,
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

const BOX_SHAPE_PADDING: f64 = 10.;
const CIRCLE_SHAPE_PADDING: f64 = 20.;

fn get_size_for_content(content: &ShapeContent, font: usize) -> Point {
    match content {
        ShapeContent::String(s) => get_size_for_str(s, font),
        ShapeContent::Html(html) => html.size(font),
    }
}

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
        ShapeKind::Box(text) => pad_shape_scalar(
            get_size_for_content(text, font),
            BOX_SHAPE_PADDING,
        ),
        ShapeKind::Circle(text) => pad_shape_scalar(
            get_size_for_content(text, font),
            CIRCLE_SHAPE_PADDING,
        ),
        ShapeKind::DoubleCircle(text) => pad_shape_scalar(
            get_size_for_content(text, font),
            CIRCLE_SHAPE_PADDING,
        ),
        ShapeKind::Record(sr) => {
            pad_shape_scalar(get_record_size(sr, dir, font), BOX_SHAPE_PADDING)
        }
        ShapeKind::Connector(text) => {
            if let Option::Some(text) = text {
                pad_shape_scalar(
                    get_size_for_content(text, font),
                    BOX_SHAPE_PADDING,
                )
            } else {
                Point::new(1., 1.)
            }
        }
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
    loc: Point,
    size: Point,
    look: &StyleAttr,
    canvas: &mut dyn RenderBackend,
) {
    match rec {
        HtmlGrid::Text(text) => {
            render_text(text, loc, size, look, canvas);
        }
        HtmlGrid::FontTable(table) => {
            render_font_table(table, loc, look, canvas);
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
    rec: &TextGrid,
    loc: Point,
    size: Point,
    look: &StyleAttr,
    canvas: &mut dyn RenderBackend,
) {
    let loc_0_x = loc.x;
    let mut loc = loc;

    loc.y -= rec.height(look.font_size) / 2.;

    for line in &rec.text_items {
        let mut line_width = 0.;
        for t in line {
            line_width += t.width(look.font_size);
        }
        loc.x = loc_0_x - line_width / 2.;
        for t in line {
            let look_text = t.build_style_attr(look);
            let text_size = get_size_for_str(&t.text, look_text.font_size);
            loc.x += text_size.x / 2.;
            let loc_text = update_location(loc, size, &t.text, &look_text);
            canvas.draw_text(loc_text, t.text.as_str(), &look_text);
            loc.x += text_size.x / 2.;
        }
        loc.y += get_line_height(line, look.font_size);
    }
}

fn render_font_table(
    rec: &TableGrid,
    loc: Point,
    look: &StyleAttr,
    canvas: &mut dyn RenderBackend,
) {
    let look = rec.build_style_attr(look);
    let table_grid_width = rec.width();
    let table_grid_height = rec.height();

    // top left origin location of the table
    let loc_0 = Point::new(
        loc.x - table_grid_width / 2.,
        loc.y - table_grid_height / 2.,
    );
    canvas.draw_rect(
        loc_0,
        Point::new(
            table_grid_width - rec.table_attr.border as f64,
            table_grid_height - rec.table_attr.border as f64,
        ),
        &look,
        Option::None,
        Option::None,
    );

    for (td_attr, c) in rec.cells.iter() {
        let cellpadding = rec.cellpadding(c);
        let cellborder = rec.cellborder(c);
        let cell_size = rec.cell_size(c);
        let cell_origin = rec.cell_pos(c);

        // center of the cell
        let cell_loc = Point::new(
            loc_0.x + cell_origin.x + cell_size.x * 0.5,
            loc_0.y + cell_origin.y + cell_size.y * 0.5,
        );
        let look_cell = td_attr.build_style_attr(&look);

        let mut look_cell_border = look.clone();
        look_cell_border.line_width = cellborder as usize;

        canvas.draw_rect(
            Point::new(
                loc_0.x + cell_origin.x + cellborder * 0.5,
                loc_0.y + cell_origin.y + cellborder * 0.5,
            ),
            cell_size.sub(Point::splat(cellborder)),
            &look_cell_border,
            Option::None,
            Option::None,
        );

        // cell inside
        let size = Point::new(
            cell_size.x - cellborder * 2. - cellpadding * 2.,
            cell_size.y - cellborder * 2. - cellpadding * 2.,
        );
        render_cell(&c, cell_loc, size, &look_cell, canvas);
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
            render_html(html, loc, size, look, canvas);
        }
        LabelOrImgGrid::Img(img) => {
            // TODO: Need to introduce setting to control file access as specificed by ofifical graphviz source
            let image_size = img.size();
            let image_size = match &img.scale {
                Scale::False => Point::new(image_size.x, image_size.y),
                Scale::True => {
                    let x_scale = size.x / image_size.x;
                    let y_scale = size.y / image_size.y;
                    let scale =
                        if x_scale < y_scale { x_scale } else { y_scale };
                    Point::new(image_size.x * scale, image_size.y * scale)
                }
                Scale::Width => {
                    let scale = size.x / image_size.x;
                    Point::new(image_size.x * scale, image_size.y)
                }
                Scale::Height => {
                    let scale = size.y / image_size.y;
                    Point::new(image_size.x, image_size.y * scale)
                }
                Scale::Both => size.clone(),
            };
            canvas.draw_image(
                Point::new(
                    loc.x - image_size.x / 2.,
                    loc.y - image_size.y / 2.,
                ),
                image_size,
                &img.source,
                None,
            );
        }
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

fn draw_shape_content(
    content: &ShapeContent,
    loc: Point,
    size: Point,
    look: &StyleAttr,
    canvas: &mut dyn RenderBackend,
) {
    match content {
        ShapeContent::String(text) => {
            canvas.draw_text(loc, text.as_str(), look);
        }
        ShapeContent::Html(html) => {
            render_html(html, loc, size, look, canvas);
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
            ShapeKind::Box(text) => {
                canvas.draw_rect(
                    self.pos.bbox(false).0,
                    self.pos.size(false),
                    &self.look,
                    self.properties.clone(),
                    Option::None,
                );
                draw_shape_content(
                    text,
                    self.pos.center(),
                    self.pos.size(false),
                    &self.look,
                    canvas,
                );
            }
            ShapeKind::Circle(text) => {
                canvas.draw_circle(
                    self.pos.center(),
                    self.pos.size(false),
                    &self.look,
                    self.properties.clone(),
                );
                // canvas.draw_text(self.pos.center(), text.as_str(), &self.look);
                draw_shape_content(
                    text,
                    self.pos.center(),
                    self.pos.size(false),
                    &self.look,
                    canvas,
                );
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
                draw_shape_content(
                    text,
                    self.pos.center(),
                    self.pos.size(false),
                    &self.look,
                    canvas,
                );
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
                    // canvas.draw_text(self.pos.middle(), label, &self.look);
                    draw_shape_content(
                        label,
                        self.pos.middle(),
                        self.pos.size(false),
                        &self.look,
                        canvas,
                    );
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
            ShapeKind::Box(x) => {
                let loc = self.pos.center();
                let size = self.pos.size(false);
                // get_connection_point_for_box(loc, size, from, force)
                match x {
                    ShapeContent::String(_) => {
                        get_connection_point_for_box(loc, size, from, force)
                    }
                    ShapeContent::Html(html) => {
                        let mut loc = self.pos.center();
                        let mut size = self.pos.size(false);
                        if let Option::Some(port_name) = port {
                            let r = get_html_port_location(
                                html,
                                loc,
                                size,
                                &mut Locator {
                                    port_name: port_name.to_string(),
                                    loc,
                                    size,
                                },
                            );
                            loc = r.0;
                            size = r.1;
                        }
                        get_connection_point_for_box(loc, size, from, force)
                    }
                }
            }
            ShapeKind::Circle(x) => {
                let loc = self.pos.center();
                let size = self.pos.size(false);
                // get_connection_point_for_circle(loc, size, from, force)
                match x {
                    ShapeContent::String(_) => {
                        get_connection_point_for_circle(loc, size, from, force)
                    }
                    ShapeContent::Html(html) => {
                        let mut loc = self.pos.center();
                        let mut size = self.pos.size(false);
                        if let Option::Some(port_name) = port {
                            let r = get_html_port_location(
                                html,
                                loc,
                                size,
                                &mut Locator {
                                    port_name: port_name.to_string(),
                                    loc,
                                    size,
                                },
                            );
                            loc = r.0;
                            size = r.1;
                        }
                        get_connection_point_for_circle(loc, size, from, force)
                    }
                }
            }
            ShapeKind::DoubleCircle(x) => {
                let loc = self.pos.center();
                let size = self.pos.size(false);
                // get_connection_point_for_circle(loc, size, from, force)
                match x {
                    ShapeContent::String(_) => {
                        get_connection_point_for_circle(loc, size, from, force)
                    }
                    ShapeContent::Html(html) => {
                        let mut loc = self.pos.center();
                        let mut size = self.pos.size(false);
                        if let Option::Some(port_name) = port {
                            let r = get_html_port_location(
                                html,
                                loc,
                                size,
                                &mut Locator {
                                    port_name: port_name.to_string(),
                                    loc,
                                    size,
                                },
                            );
                            loc = r.0;
                            size = r.1;
                        }
                        get_connection_point_for_circle(loc, size, from, force)
                    }
                }
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
