//! Defines the interfaces for accessing and querying shapes.

use super::{
    geometry::{Point, Position},
    style::StyleAttr,
};

/// This is the trait that all elements that can be arranged need to implement.
pub trait Visible {
    /// \return the Position of the shape.
    fn position(&self) -> Position;
    /// \return the mutable reference to the Position of the shape.
    fn position_mut(&mut self) -> &mut Position;
    /// Return true if the element is a connector.
    fn is_connector(&self) -> bool;
    /// Swap the coordinates of the location and size.
    fn transpose(&mut self);
    /// Update the size of the shape.
    fn resize(&mut self);
}
/// This is the trait that all elements that can be rendered on a canvas need to
/// implement.
pub trait Renderable {
    /// Render the shape into a canvas.
    /// If \p debug is set then extra markers will be rendered.
    fn render(&self, debug: bool, canvas: &mut dyn RenderBackend);

    /// \Return the coordinate for the connection point of an arrow that's
    /// coming from the direction of \p from.
    /// The format of the path is (x, y, cx, cy), where cx and cy, are the
    /// control points of the bezier curve.
    /// \p force is the magnitude of the edge direction.
    /// \p port is the optional port name (for named records).
    fn get_connector_location(
        &self,
        from: Point,
        force: f64,
        port: &Option<String>,
    ) -> (Point, Point);

    /// Computes the coordinate for the connection point of an arrow that's
    /// passing through this edge.
    /// coming from the direction of \p from.
    /// \returns the bezier path in the format (x, y, cx, cy), where cx and cy,
    /// are the control points for the entry path of the bezier curve. The exit
    /// path is assumed to be the mirror point for the center (first point).
    /// \p force is the magnitude of the edge direction.
    /// This works with the get_connector_location method for drawing edges.
    fn get_passthrough_path(
        &self,
        from: Point,
        to: Point,
        force: f64,
    ) -> (Point, Point);
}

pub type ClipHandle = usize;

/// This is the trait that all rendering backends need to implement.
pub trait RenderBackend {
    /// Draw a rectangle. The top-left point of the rectangle is \p xy. The shape
    /// style (color, edge-width) are passed in \p look. The parameter \p clip
    /// is an optional clip region (see: create_clip).
    fn draw_rect(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        url: Option<&str>,
        clip: Option<ClipHandle>,
    );

    /// Draw a line between \p start and \p stop.
    fn draw_line(&mut self, start: Point, stop: Point, look: &StyleAttr);

    /// Draw an ellipse with the center \p xy, and size \p size.
    fn draw_circle(
        &mut self,
        xy: Point,
        size: Point,
        look: &StyleAttr,
        url: Option<&str>,
    );

    /// Draw a labe.
    fn draw_text(&mut self, xy: Point, text: &str, look: &StyleAttr);

    /// Draw an arrow, with a label, with the style parameters in \p look.
    fn draw_arrow(
        &mut self,
        path: &[(Point, Point)],
        dashed: bool,
        head: (bool, bool),
        look: &StyleAttr,
        text: &str,
    );

    /// Generate a clip region that shapes can use to create complex shapes.
    fn create_clip(
        &mut self,
        xy: Point,
        size: Point,
        rounded_px: usize,
    ) -> ClipHandle;
}
