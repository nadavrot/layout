//! SVG rendering backend that accepts draw calls and saves the output to a file.

use crate::core::color::Color;
use crate::core::format::{ClipHandle, RenderBackend};
use crate::core::geometry::Point;
use crate::core::style::StyleAttr;
use std::collections::HashMap;

static SVG_HEADER: &str =
    r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>"#;

static SVG_DEFS: &str = r#"<defs>
<marker id="startarrow" markerWidth="10" markerHeight="7"
refX="0" refY="3.5" orient="auto">
<polygon points="10 0, 10 7, 0 3.5" />
</marker>
<marker id="endarrow" markerWidth="10" markerHeight="7"
refX="10" refY="3.5" orient="auto">
<polygon points="0 0, 10 3.5, 0 7" />
</marker>

</defs>"#;

static SVG_FOOTER: &str = "</svg>";

fn escape_string(x: &str) -> String {
    let mut res = String::new();
    for c in x.chars() {
        match c {
            '&' => {
                res.push_str("&amp;");
            }
            '<' => {
                res.push_str("&lt;");
            }
            '>' => {
                res.push_str("&gt;");
            }
            '"' => {
                res.push_str("&quot;");
            }
            '\'' => {
                res.push_str("&apos;");
            }
            _ => {
                res.push(c);
            }
        }
    }
    res
}

pub struct SVGWriter {
    content: String,
    view_size: Point,
    counter: usize,
    // Maps font sizes to their class name and class impl.
    font_style_map: HashMap<usize, (String, String)>,
    // A list of clip regions to generate.
    clip_regions: Vec<String>,
}

impl SVGWriter {
    pub fn new() -> SVGWriter {
        SVGWriter {
            content: String::new(),
            view_size: Point::zero(),
            counter: 0,
            font_style_map: HashMap::new(),
            clip_regions: Vec::new(),
        }
    }
}

impl Default for SVGWriter {
    fn default() -> Self {
        Self::new()
    }
}

// This trivial implementation of `drop` adds a print to a file.
impl Drop for SVGWriter {
    fn drop(&mut self) {}
}

impl SVGWriter {
    // Grow the viewable svg window to include the point \p point plus some
    // offset \p size.
    fn grow_window(&mut self, point: Point, size: Point) {
        self.view_size.x = self.view_size.x.max(point.x + size.x + 5.);
        self.view_size.y = self.view_size.y.max(point.y + size.y + 5.);
    }

    // Gets or creates a font 'class' for the parameters. Returns the class
    // name.
    fn get_or_create_font_style(&mut self, font_size: usize) -> String {
        if let Option::Some(x) = self.font_style_map.get(&font_size) {
            return x.0.clone();
        }
        let class_name = format!("a{}", font_size);
        let class_impl = format!(
            ".a{} {{ font-size: {}px; font-family: Times, serif; }}",
            font_size, font_size
        );
        let impl_ = (class_name.clone(), class_impl);
        self.font_style_map.insert(font_size, impl_);
        class_name
    }

    fn emit_svg_font_styles(&self) -> String {
        let mut content = String::new();
        content.push_str("<style>\n");
        for p in self.font_style_map.iter() {
            content.push_str(&p.1 .1);
            content.push('\n');
        }
        content.push_str("</style>\n");
        for p in self.clip_regions.iter() {
            content.push_str(p);
            content.push('\n');
        }
        content
    }

    pub fn finalize(&self) -> String {
        let mut result = String::new();
        result.push_str(SVG_HEADER);

        let svg_line = format!(
            "<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\
            \" xmlns=\"http://www.w3.org/2000/svg\">\n",
            self.view_size.x,
            self.view_size.y,
            self.view_size.x,
            self.view_size.y
        );
        result.push_str(&svg_line);
        result.push_str(SVG_DEFS);
        result.push_str(&self.emit_svg_font_styles());
        result.push_str(&self.content);
        result.push_str(SVG_FOOTER);
        result
    }
}
impl RenderBackend for SVGWriter {
    fn draw_rect(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        properties: Option<String>,
        clip: Option<ClipHandle>,
    ) {
        self.grow_window(xy, size);

        let mut clip_option = String::new();
        if let Option::Some(clip_id) = clip {
            clip_option = format!("clip-path=\"url(#C{})\"", clip_id);
        }
        let props = properties.unwrap_or_default();
        let fill_color = look.fill_color.unwrap_or_else(Color::transparent);
        let stroke_width = look.line_width;
        let stroke_color = look.line_color;
        let rounded_px = look.rounded;
        let line1 = format!(
            "<g {props}>\n
            <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" 
            stroke-width=\"{}\" stroke=\"{}\" rx=\"{}\" {} />\n
            </g>\n",
            xy.x,
            xy.y,
            size.x,
            size.y,
            fill_color.to_web_color(),
            stroke_width,
            stroke_color.to_web_color(),
            rounded_px,
            clip_option
        );
        self.content.push_str(&line1);
    }

    fn draw_circle(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        properties: Option<String>,
    ) {
        self.grow_window(xy, size);
        let fill_color = look.fill_color.unwrap_or_else(Color::transparent);
        let stroke_width = look.line_width;
        let stroke_color = look.line_color;
        let props = properties.unwrap_or_default();
        let line1 = format!(
            "<g {props}>\n
            <ellipse cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" fill=\"{}\" 
            stroke-width=\"{}\" stroke=\"{}\"/>\n
            </g>\n",
            xy.x,
            xy.y,
            size.x / 2.,
            size.y / 2.,
            fill_color.to_web_color(),
            stroke_width,
            stroke_color.to_web_color()
        );
        self.content.push_str(&line1);
    }

    fn draw_text(
        &mut self,
        xy: Point,
        text: &str,
        look: &StyleAttr,
        
    ) {
        let len = text.len();

        let font_class = self.get_or_create_font_style(look.font_size);

        let mut content = String::new();
        let cnt = 1 + text.lines().count();
        let size_y = (cnt * look.font_size) as f64;
        for line in text.lines() {
            content.push_str(&format!("<tspan x = \"{}\" dy=\"1.0em\">", xy.x));
            content.push_str(&escape_string(line));
            content.push_str("</tspan>");
        }

        self.grow_window(xy, Point::new(10., len as f64 * 10.));
        let line = format!(
            "<text dominant-baseline=\"middle\" text-anchor=\"middle\" 
            x=\"{}\" y=\"{}\" class=\"{}\">{}</text>",
            xy.x,
            xy.y - size_y / 2.,
            font_class,
            &content
        );

        self.content.push_str(&line);
    }

    fn draw_arrow(
        &mut self,
        // This is a list of vectors. The first vector is the "exit" vector
        // from the first point, and the rest of the vectors are "entry" vectors
        // into the following points.
        path: &[(Point, Point)],
        dashed: bool,
        head: (bool, bool),
        look: &StyleAttr,
        properties: Option<String>,
        text: &str,
    ) {
        // Control points as defined in here:
        // https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/Paths#curve_commands
        // Structured as [(M,C) S S S ...]
        for point in path {
            self.grow_window(point.0, Point::zero());
            self.grow_window(point.1, Point::zero());
        }

        let dash = if dashed {
            &"stroke-dasharray=\"5,5\""
        } else {
            &""
        };
        let start = if head.0 {
            "marker-start=\"url(#startarrow)\""
        } else {
            ""
        };
        let end = if head.1 {
            "marker-end=\"url(#endarrow)\""
        } else {
            ""
        };

        let mut path_builder = String::new();

        // Handle the "exit vector" from the first point.
        path_builder.push_str(&format!(
            "M {} {} C {} {}, {} {}, {} {} ",
            path[0].0.x,
            path[0].0.y,
            path[0].1.x,
            path[0].1.y,
            path[1].0.x,
            path[1].0.y,
            path[1].1.x,
            path[1].1.y
        ));

        // Handle the "entry vector" from the rest of the points.
        for point in path.iter().skip(2) {
            path_builder.push_str(&format!(
                "S {} {}, {} {} ",
                point.0.x, point.0.y, point.1.x, point.1.y
            ));
        }

        let stroke_width = look.line_width;
        let stroke_color = look.line_color;
        let props = properties.unwrap_or_default();
        let line = format!(
            "<g {props}>\n
            <path id=\"arrow{}\" d=\"{}\" \
            stroke=\"{}\" stroke-width=\"{}\" {} {} {} 
            fill=\"transparent\" />\n
            </g>\n",
            self.counter,
            path_builder.as_str(),
            stroke_color.to_web_color(),
            stroke_width,
            dash,
            start,
            end
        );
        self.content.push_str(&line);

        let font_class = self.get_or_create_font_style(look.font_size);
        let line = format!(
            "<text><textPath href=\"#arrow{}\" startOffset=\"50%\" \
            text-anchor=\"middle\" class=\"{}\">{}</textPath></text>",
            self.counter,
            font_class,
            escape_string(text)
        );
        self.content.push_str(&line);
        self.counter += 1;
    }

    fn draw_line(&mut self, start: Point, stop: Point, look: &StyleAttr,properties: Option<String>) {
        let stroke_width = look.line_width;
        let stroke_color = look.line_color;
        let props = properties.unwrap_or_default();
        let line1 = format!(
            "<g {props}>\n
             <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke-width=\"{}\"
             stroke=\"{}\" />\n
             </g>\n",
            start.x,
            start.y,
            stop.x,
            stop.y,
            stroke_width,
            stroke_color.to_web_color()
        );
        self.content.push_str(&line1);
    }

    fn create_clip(
        &mut self,
        xy: Point,
        size: Point,
        rounded_px: usize,
    ) -> ClipHandle {
        let handle = self.clip_regions.len();

        let clip_code = format!(
            "<clipPath id=\"C{}\"><rect x=\"{}\" y=\"{}\" \
            width=\"{}\" height=\"{}\" rx=\"{}\" /> \
            </clipPath>",
            handle, xy.x, xy.y, size.x, size.y, rounded_px
        );

        self.clip_regions.push(clip_code);

        handle
    }
}
