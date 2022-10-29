//! This is a sample program that prints a bunch of elements to an SVG file so
//! we can visually see if the things that we render look right.

use layout::backends::svg::SVGWriter;
use layout::core::base::Orientation;
use layout::core::color::Color;
use layout::core::format::{RenderBackend, Renderable, Visible};
use layout::core::geometry::{segment_rect_intersection, Point};
use layout::core::style::{LineStyleKind, StyleAttr};
use layout::core::utils::save_to_file;
use layout::std_shapes::render;
use layout::std_shapes::render::get_shape_size;
use layout::std_shapes::shapes::*;

pub const LAYOUT_HELPER: bool = true;

fn generate_record() -> ShapeKind {
    let a = RecordDef::new_text_with_port("a", "a");
    let b = RecordDef::new_text_with_port("b", "b");
    let c = RecordDef::new_text_with_port("c", "c");
    let v0 = vec![a, b, c];
    let d = RecordDef::new_text_with_port("d", "d");
    let f = RecordDef::new_text("f");
    let v1 = vec![d, f];
    let t0 = RecordDef::Array(v0);
    let t1 = RecordDef::Array(v1);
    ShapeKind::Record(RecordDef::Array(vec![t0, t1]))
}

fn test0(offset_x: f64, offset_y: f64, svg: &mut SVGWriter, shape_idx: usize) {
    let sz = Point::new(100., 100.);
    let mut shapes = Vec::new();

    for i in 0..9 {
        let i = i as f64;
        let deg: f64 = 0.0174533 * 40.;
        let loc = Point::new(
            offset_x + 500. + 400. * (i * deg).cos(),
            offset_y + 500. + 400. * (i * deg).sin(),
        );

        let sp = match shape_idx {
            0 => ShapeKind::new_box(&i.to_string()),
            1 => ShapeKind::new_circle(&i.to_string()),
            2 => ShapeKind::new_double_circle(&i.to_string()),
            3 => generate_record(),
            _ => {
                panic!("Invalid test number");
            }
        };

        let look = StyleAttr::simple();
        let mut es = Element::create(sp, look, Orientation::LeftToRight, sz);
        es.position_mut().move_to(loc);
        shapes.push(es);
    }

    for s in &shapes {
        s.render(LAYOUT_HELPER, svg);
    }
    for s1 in &shapes {
        for s2 in &shapes {
            let stl = Arrow::simple("x");
            let vec: Vec<Element> = vec![s1.clone(), s2.clone()];
            render::render_arrow(svg, LAYOUT_HELPER, &vec[..], &stl);
        }
    }
}

fn test1(offset_x: f64, offset_y: f64, svg: &mut SVGWriter) {
    let sz = Point::new(100., 100.);

    let sp0 = ShapeKind::new_box("one");
    let sp1 = ShapeKind::new_box("two");
    let mut look0 = StyleAttr::simple();
    look0.fill_color = Some(Color::fast("pink"));
    look0.line_color = Color::fast("brown");

    let mut look1 = StyleAttr::simple();
    look1.fill_color = Some(Color::fast("olive"));
    look1.line_color = Color::fast("brown");

    let mut es0 = Element::create(sp0, look0, Orientation::LeftToRight, sz);
    let mut es1 = Element::create(sp1, look1, Orientation::LeftToRight, sz);

    let loc0 = Point::new(offset_x, offset_y);
    let loc1 = Point::new(offset_x, offset_y + 150.);

    es0.position_mut().move_to(loc0);
    es1.position_mut().move_to(loc1);

    es0.render(LAYOUT_HELPER, svg);
    es1.render(LAYOUT_HELPER, svg);

    let stl = Arrow::simple("x");
    let vec: Vec<Element> = vec![es0.clone(), es1.clone()];
    render::render_arrow(svg, LAYOUT_HELPER, &vec[..], &stl);
}

fn test3(
    offset_x: f64,
    offset_y: f64,
    offset_x_other: f64,
    svg: &mut SVGWriter,
) {
    let sz = Point::new(400., 50.);

    let sp0 = ShapeKind::new_box("one");
    let sp1 = ShapeKind::new_box("two");
    let mut look0 = StyleAttr::simple();
    look0.fill_color = Some(Color::fast("pink"));
    look0.line_color = Color::fast("brown");

    let mut look1 = StyleAttr::simple();
    look1.fill_color = Some(Color::fast("olive"));
    look1.line_color = Color::fast("brown");

    let mut es0 = Element::create(sp0, look0, Orientation::LeftToRight, sz);
    let mut es1 = Element::create(sp1, look1, Orientation::LeftToRight, sz);

    let loc0 = Point::new(offset_x, offset_y);
    let loc1 = Point::new(offset_x + offset_x_other, offset_y + 150.);

    es0.move_to(loc0);
    es1.move_to(loc1);

    es0.render(LAYOUT_HELPER, svg);
    es1.render(LAYOUT_HELPER, svg);

    let stl = Arrow::simple("down");
    let vec: Vec<Element> = vec![es0.clone(), es1.clone()];
    render::render_arrow(svg, LAYOUT_HELPER, &vec[..], &stl);
}

fn test4(
    offset_x: f64,
    offset_y: f64,
    offset_x_other: f64,
    svg: &mut SVGWriter,
) {
    let sz = Point::new(400., 50.);

    let sp0 = ShapeKind::new_circle("one");
    let sp1 = ShapeKind::new_circle("two");

    let mut look0 = StyleAttr::simple();
    look0.fill_color = Some(Color::fast("gold"));
    look0.line_color = Color::fast("black");

    let mut look1 = StyleAttr::simple();
    look1.fill_color = Some(Color::fast("lightblue"));
    look1.line_color = Color::fast("black");

    let mut es0 = Element::create(sp0, look0, Orientation::LeftToRight, sz);
    let mut es1 = Element::create(sp1, look1, Orientation::LeftToRight, sz);

    let loc0 = Point::new(offset_x, offset_y);
    let loc1 = Point::new(offset_x + offset_x_other, offset_y + 150.);

    es0.move_to(loc0);
    es1.move_to(loc1);

    es0.render(LAYOUT_HELPER, svg);
    es1.render(LAYOUT_HELPER, svg);

    let stl = Arrow::simple("down");
    let vec: Vec<Element> = vec![es0.clone(), es1.clone()];
    render::render_arrow(svg, LAYOUT_HELPER, &vec[..], &stl);
}

fn test5(
    offset_x: f64,
    offset_y: f64,
    offset_x_connector: f64,
    offset_x_last: f64,
    swap: bool,
    svg: &mut SVGWriter,
) {
    let sz = Point::new(50., 50.);

    let sp0 = ShapeKind::new_circle("one");
    let sp1 = ShapeKind::new_circle("two");

    let mut look0 = StyleAttr::simple();
    look0.fill_color = Some(Color::fast("dimgrey"));
    look0.line_color = Color::fast("black");

    let mut look1 = StyleAttr::simple();
    look1.fill_color = Some(Color::fast("salmon"));
    look1.line_color = Color::fast("black");

    let mut es0 = Element::create(sp0, look0, Orientation::LeftToRight, sz);
    let mut es1 = Element::create(sp1, look1, Orientation::LeftToRight, sz);
    let mut inv = Element::empty_connector(Orientation::LeftToRight);

    let mut loc0 = Point::new(offset_x, offset_y);
    let loc1 = Point::new(offset_x + offset_x_connector, offset_y + 100.);
    let mut loc2 = Point::new(offset_x + offset_x_last, offset_y + 200.);

    if swap {
        std::mem::swap(&mut loc0, &mut loc2);
    }

    es0.move_to(loc0);
    inv.move_to(loc1);
    es1.move_to(loc2);

    let stl = Arrow::simple("");
    let vec: Vec<Element> = vec![es0, inv, es1];
    render::render_arrow(svg, LAYOUT_HELPER, &vec[..], &stl);
}

fn test6(
    offset_x: f64,
    offset_y: f64,
    offset_x_other: f64,
    last_x_other: f64,
    svg: &mut SVGWriter,
) {
    let sz = Point::new(150., 150.);

    let mut look0 = StyleAttr::simple();
    look0.fill_color = Some(Color::fast("darkgoldenrod"));
    look0.line_color = Color::fast("black");

    let mut look1 = StyleAttr::simple();
    look1.fill_color = Some(Color::fast("darkkhaki"));
    look1.line_color = Color::fast("black");

    let rec0 = generate_record();
    let rec1 = generate_record();
    let mut es0 = Element::create(rec0, look0, Orientation::LeftToRight, sz);
    let mut es1 = Element::create(rec1, look1, Orientation::LeftToRight, sz);
    let mut inv = Element::empty_connector(Orientation::LeftToRight);

    let loc0 = Point::new(offset_x, offset_y);
    let loc1 = Point::new(offset_x + offset_x_other, offset_y + 150.);
    let loc2 = Point::new(offset_x + last_x_other, offset_y + 300.);

    es0.move_to(loc0);
    inv.move_to(loc1);
    es1.move_to(loc2);

    es0.render(LAYOUT_HELPER, svg);
    inv.render(LAYOUT_HELPER, svg);
    es1.render(LAYOUT_HELPER, svg);

    let look1 = StyleAttr::simple();

    let stl = Arrow::new(
        LineEndKind::None,
        LineEndKind::Arrow,
        LineStyleKind::Normal,
        "a to c",
        &look1,
        &Some("a".to_string()),
        &Some("c".to_string()),
    );
    let vec: Vec<Element> = vec![es0.clone(), inv.clone(), es1.clone()];
    render::render_arrow(svg, LAYOUT_HELPER, &vec[..], &stl);
}

fn test7(offset_x: f64, offset_y: f64, svg: &mut SVGWriter) {
    let sp = ShapeKind::new_box("intersect");
    let mut look1 = StyleAttr::simple();
    look1.fill_color = Some(Color::fast("steelblue"));
    look1.line_color = Color::fast("black");

    let mut red = StyleAttr::simple();
    red.line_color = Color::fast("red");

    let sz = Point::splat(100.);
    let mut es0 = Element::create(sp, look1, Orientation::LeftToRight, sz);
    let center = Point::new(offset_x, offset_y);
    es0.move_to(center);
    es0.render(false, svg);

    let mut state: u32 = 41;

    fn rand(state: &mut u32, range: u32) -> u32 {
        let k = state
            .saturating_mul(11)
            .saturating_add(101)
            .wrapping_rem(91241241);
        *state = k;
        *state % range
    }

    for _ in 1..34 {
        let n0 = rand(&mut state, 400) as f64;
        let n1 = rand(&mut state, 400) as f64;
        let n2 = rand(&mut state, 400) as f64;
        let n3 = rand(&mut state, 400) as f64;
        let from = Point::new(n0, n1);
        let to = Point::new(n2, n3);
        let from = from.add(center).sub(Point::splat(200.));
        let to = to.add(center).sub(Point::splat(200.));

        if segment_rect_intersection((from, to), es0.position().bbox(false)) {
            svg.draw_line(from, to, &red);
        } else {
            svg.draw_line(from, to, &StyleAttr::simple());
        }
    }
}

fn test8(offset_x: f64, offset_y: f64, svg: &mut SVGWriter) {
    let a = RecordDef::new_text_with_port("a", "a");
    let b = RecordDef::new_text_with_port("baaaa", "b");
    let c = RecordDef::new_text_with_port("c", "c");
    let v0 = vec![a, b, c];
    let d = RecordDef::new_text_with_port("dsssssssssssss", "d");
    let f = RecordDef::new_text("f");
    let v1 = vec![d, f];
    let t0 = RecordDef::Array(v0);
    let t1 = RecordDef::Array(v1);
    let rec0 = ShapeKind::Record(RecordDef::Array(vec![t0, t1]));
    let sz = get_shape_size(Orientation::LeftToRight, &rec0, 15, false);

    let mut look1 = StyleAttr::simple();
    look1.fill_color = Some(Color::fast("steelblue"));
    look1.line_color = Color::fast("white");

    let mut es0 = Element::create(rec0, look1, Orientation::LeftToRight, sz);

    let loc0 = Point::new(offset_x, offset_y);
    es0.move_to(loc0);

    es0.render(LAYOUT_HELPER, svg);
}

fn main() {
    let mut svg = SVGWriter::new();
    test0(0., 0., &mut svg, 0);
    test0(1000., 0., &mut svg, 1);
    test0(2000., 0., &mut svg, 2);
    test0(3000., 0., &mut svg, 3);
    test1(1000., 700., &mut svg);
    test3(100., 1050., 150., &mut svg);
    test3(100., 1350., 250., &mut svg);
    test3(100., 1650., 350., &mut svg);
    test3(100., 1950., 450., &mut svg);
    test4(600., 1050., 450., &mut svg);
    test4(600., 1350., 0., &mut svg);
    test5(900., 1350., 0., 0., false, &mut svg);
    test5(1200., 1350., -150., 0., false, &mut svg);
    test5(1200., 1350., 150., 0., false, &mut svg);
    test5(960., 1350., 0., 0., true, &mut svg);
    test5(1300., 1350., -150., 0., true, &mut svg);
    test5(1360., 1350., 150., 0., true, &mut svg);
    test5(860., 1650., 20., 50., true, &mut svg);
    test5(1050., 1650., -150., -50., true, &mut svg);
    test5(1060., 1650., 150., 170., true, &mut svg);
    test5(1500., 1350., 30., 180., true, &mut svg);
    test5(1630., 1350., 90., -110., true, &mut svg);
    test6(1900., 1550., 150., 150., &mut svg);
    test7(3600., 1450., &mut svg);
    test8(2600., 1450., &mut svg);

    let content = svg.finalize();
    let filename = "/tmp/shapes.svg";
    let res = save_to_file(filename, &content);
    if let Result::Err(err) = res {
        log::error!("Could not write the file {}", filename);
        log::error!("Error {}", err);
        return;
    }
    log::info!("Wrote {}", filename);
}
