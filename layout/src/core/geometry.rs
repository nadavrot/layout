//! Contains functions that are related to the geometry of shapes and their
//! interaction. This includes things like intersection of shapes and length
//! of vectors.

// Stores a 2D coordinate, or a vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn zero() -> Point {
        Self { x: 0., y: 0. }
    }

    pub fn new(x: f64, y: f64) -> Point {
        Self { x, y }
    }

    pub fn splat(s: f64) -> Point {
        Point::new(s, s)
    }

    pub fn neg(&self) -> Point {
        Point::new(-self.x, -self.y)
    }

    pub fn add(&self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }

    pub fn sub(&self, other: Point) -> Point {
        self.add(other.neg())
    }

    pub fn distance_to(&self, other: Point) -> f64 {
        let d = self.sub(other);
        (d.x * d.x + d.y * d.y).sqrt()
    }

    pub fn length(&self) -> f64 {
        Point::zero().distance_to(*self)
    }

    pub fn scale(&self, s: f64) -> Point {
        Point::new(self.x * s, self.y * s)
    }

    pub fn transpose(&self) -> Point {
        Point::new(self.y, self.x)
    }

    pub fn rotate_around(&self, center: Point, angle: f64) -> Point {
        let normalized = self.sub(center);
        let rotated = normalized.rotate(angle);
        rotated.add(center)
    }
    pub fn rotate(&self, angle: f64) -> Point {
        let x = self.x;
        let y = self.y;
        Point::new(
            x * angle.cos() - y * angle.sin(),
            x * angle.sin() + y * angle.cos(),
        )
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(x: {:.3}, y: {:.3})", self.x, self.y)
    }
}

/// \returns the intersection point for a line with slope \p m with an ellipse
/// with the formula. 1 = (x^2 / a^2) + (y^2 / b^2).
/// Replace Y with the line equation and isolate x and solve to get the
/// intersection point with the ellipse.
/// Notice that a line has two intersection points with a circle, so users need
/// to figure out which of the two values (+X, +Y) or (-X, -Y) is relevant.
pub fn ellipse_line_intersection(a: f64, b: f64, m: f64) -> Point {
    let x: f64 = ((a * a * b * b) / (b * b + a * a * m * m)).sqrt();
    let y: f64 = m * x;
    Point::new(x, y)
}

/// This is the implementation of get_connector_location for circle-like shapes.
/// 'See get_connector_location' for details.
pub fn get_connection_point_for_circle(
    loc: Point,
    size: Point,
    from: Point,
    force: f64,
) -> (Point, Point) {
    let loc = loc;
    let dx = from.x - loc.x;
    let dy = from.y - loc.y;

    let a = size.x / 2.;
    let b = size.y / 2.;
    let m = dy / dx;

    if dx == 0. {
        let b = b * dy.signum();
        let loc1 = Point::new(loc.x, loc.y + b);
        return create_vector_of_length(loc1, from, force);
    }

    let mut v = ellipse_line_intersection(a, b, m);

    // The intersection formula gives two solutions (for the sqrt). Figure out
    // which solution is needed depending on the direction of the arrow (dx).
    if dx < 0. {
        v = v.neg();
    }

    let loc1 = loc.add(v);
    create_vector_of_length(loc1, from, force)
}

/// Perform linear interpolation of the vectors v0 and v1, using the
/// ratio w which is assumed to be between 0..1.
pub fn interpolate(v0: Point, v1: Point, w: f64) -> Point {
    v0.scale(w).add(v1.scale(1. - w))
}

/// Return the normalized vector \p v multiplied by the scalar \p s.
pub fn normalize_scale_vector(v: Point, s: f64) -> Point {
    let len = Point::zero().distance_to(v);
    assert!(len > 0., "Can't normalize the unit vector");
    v.scale(s / len)
}
// Returns a vector in a direction of \to target, of length \p s.
pub fn create_vector_of_length(
    from: Point,
    to: Point,
    s: f64,
) -> (Point, Point) {
    // We can't normalize the zero vectors, so just create a unit vector.
    if from == to {
        return (from, Point::new(from.x + s, from.y));
    }
    let t = to.sub(from);
    let t = normalize_scale_vector(t, s);
    (from, t.add(from))
}

/// This is the implementation of get_connector_location for box-like shapes.
/// 'See get_connector_location' for details.
pub fn get_connection_point_for_box(
    loc: Point,
    size: Point,
    from: Point,
    force: f64,
) -> (Point, Point) {
    let mut loc = loc;
    let mut size = size;

    // Try to cut the left or right part of the box, to make arrows connect to
    // a region of the box that's closer.
    //                                     Example:    |
    //                                                 v
    //                                              [------|------]
    if from.x > loc.x + size.x / 2. {
        // Cut the left half.
        size.x /= 2.;
        loc.x += size.x / 2.;
    } else if from.x < loc.x - size.x / 2. {
        // Cut the right half.
        size.x /= 2.;
        loc.x -= size.x / 2.;
    }

    let dx = loc.x - from.x;
    let dy = loc.y - from.y;

    let mut box_x = size.x / 2.;
    let mut box_y = size.y / 2.;

    // This is a vertical edge. Don't divide by zero.
    if dx == 0. {
        // Edge coming from the top. Connect on top.
        if dy > 0. {
            let loc = Point::new(loc.x, loc.y - box_y);
            return create_vector_of_length(loc, from, force);
        } else {
            // Connect on the bottom.
            let loc = Point::new(loc.x, loc.y + box_y);
            return create_vector_of_length(loc, from, force);
        }
    }

    let slope_from = dy / dx;
    // How much y goes up or down as we progress along x, up to the edge.
    let mut gain_y = box_x * slope_from;

    // Need to connect from the side.
    if gain_y.abs() < box_y {
        if dx > 0. {
            box_x = -box_x;
            gain_y = -gain_y;
        }

        let con = Point::new(loc.x + box_x, loc.y + gain_y);
        return create_vector_of_length(con, from, force);
    }

    // How much x gain as we move along y and hit the top or bottom.
    // dx * s = y  => dx = y/s
    let mut gain_x = box_y / slope_from;

    if dy > 0. {
        box_y = -box_y;
        gain_x = -gain_x;
    }

    let con = Point::new(loc.x + gain_x, loc.y + box_y);
    create_vector_of_length(con, from, force)
}

pub fn get_passthrough_path_invisible(
    _size: Point,
    center: Point,
    from: Point,
    to: Point,
    force: f64,
) -> (Point, Point) {
    //  We are trying to figure out the vector that represents the bezier
    //  control point from R to A. If R is close to A then we should to honor
    //  the preference of A, and not B, to make sure that we don't overshoot
    // and create an edge that wraps around. We interpolate the direction
    // vectors in reverse proportion to make this happen.
    //
    //      (from)A-------->R (center)
    //                       \
    //                        \
    //                         v
    //                          B (to)

    let ar = center.sub(from);
    let rb = to.sub(center);

    let a_outgoing_edge = normalize_scale_vector(ar.neg(), force);
    let b_outgoing_edge = normalize_scale_vector(rb.neg(), force);

    // If this is a self-edge then handle it in a special way. First check if
    // the source and destination are identical. If they are then prevent the
    // sharp-edge problem and give the middle part a bow by changing the angle
    // by 90'.
    let sum = a_outgoing_edge.add(b_outgoing_edge);
    if sum.length() < 1. {
        let edge = a_outgoing_edge.rotate(90_f64.to_radians());
        return (center, edge.add(center));
    }

    let total = ar.length() + rb.length();
    let mut a_ratio = ar.length() / total;

    // If the edges are vertical or horizontal then make sure that they are
    // perfectly aligned, because lines that are almost straight don't look
    // good.
    if center.x == to.x || center.y == to.y {
        a_ratio = 1.;
    } else if center.x == from.x || center.y == from.y {
        a_ratio = 0.;
    }

    let res = interpolate(a_outgoing_edge, b_outgoing_edge, 1. - a_ratio);
    (center, res.add(center))
}

/// Make the shape have the same X and Y values.
pub fn make_size_square(sz: Point) -> Point {
    let l = sz.x.max(sz.y);
    Point::new(l, l)
}

/// Increase the size of X and Y by \p s.
pub fn pad_shape_scalar(size: Point, s: f64) -> Point {
    Point::new(size.x + s, size.y + s)
}

#[inline]
fn get_width_of_line(label: &str) -> usize {
    label.chars().count()
}

/// Estimate the bounding box of some rendered text.
pub fn get_size_for_str(label: &str, font_size: usize) -> Point {
    // Find the longest line.
    let max_line_len = if !label.is_empty() {
        label.lines().map(|x| get_width_of_line(x)).max().unwrap()
    } else {
        0
    };
    let ts = (max_line_len.max(1), label.lines().count().max(1));
    Point::new(ts.0 as f64, ts.1 as f64).scale(font_size as f64)
}

/// \return true if \p x is in the inclusive range P.x .. P.y.
pub fn in_range(range: (f64, f64), x: f64) -> bool {
    x >= range.0 && x <= range.1
}

/// trivial function for checking aproximate equality of f64, within epsion of f64
fn approx_eq_f64(x: f64, y: f64) -> bool {
    if x == 0. {
        y.abs() < f64::EPSILON
    } else if y == 0. {
        x.abs() < f64::EPSILON
    } else {
        let abs_diff = (x - y).abs();
        if abs_diff < f64::EPSILON {
            true
        } else {
            abs_diff / x.abs().max(y.abs()) < f64::EPSILON
        }
    }
}

/// Similar to usual smaller than or equal to op, except for equal is withint f64 epsilon
fn smaller_than_or_equal_to_f64(x: f64, y: f64) -> bool {
    if x > y {
        false
    } else {
        // stricter than usual approx_eq for f64, but works ok and stricter
        approx_eq_f64(x, y)
    }
}

/// \return True if the boxes (defined by the bounding box) intersect.
pub fn do_boxes_intersect(p1: (Point, Point), p2: (Point, Point)) -> bool {
    let overlap_x = smaller_than_or_equal_to_f64(p2.0.x, p1.1.x)
        && smaller_than_or_equal_to_f64(p1.0.x, p2.1.x);
    let overlap_y = smaller_than_or_equal_to_f64(p2.0.y, p1.1.y)
        && smaller_than_or_equal_to_f64(p1.0.y, p2.1.y);
    overlap_x && overlap_y
}

/// Return the weighted median for \p vec.
/// This is the method that's described in
/// "DAG - A Program that Draws Directed Graphs"
/// Gansner, North, Vo 1989. Pg 10.
pub fn weighted_median(vec: &[f64]) -> f64 {
    assert!(!vec.is_empty(), "array can't be empty");

    let mut vec = vec.to_vec();
    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if vec.len() == 1 {
        return vec[0];
    }

    if vec.len() == 2 {
        return (vec[0] + vec[1]) / 2.;
    }
    let mid = vec.len() / 2;

    if vec.len() % 2 == 1 {
        return vec[mid];
    }

    (vec[mid] + vec[mid - 1]) / 2.
}

/// Represents the size, location and centerpoint of a shape. We align shapes
/// along their center points, and have edges directed at the center. Shapes
/// like Box and Circle have their center point in the middle, but labels have
/// their center point in one of the sides to make sure that edges don't
/// obscure the text. The halo is the gap around the shape where nothing can be
/// placed and it is applied symmetrically to the sides.
///
/// This struct has fields that represent the following points:
///   ____________________
///  |    _____________   |
///  |  |             |   |
///  |  |             |   |
///  |  |      M <----|---|--the middle of the shape, in absolute coordinates.
///  |  |         C <-|---|--the center point, saved as delta, relative to M.
///  |  |_____________|   |
///  |                ^---|--- the size of the shape.
///  |____________________| <----- size + halo.
///
#[derive(Debug, Clone, Copy)]
pub struct Position {
    middle: Point, // The middle of the shape, in absolute coordinates.
    size: Point,   // Height and width of the shape.
    center: Point, // Delta from the middle point.
    halo: Point,   // The boundary around the shape, applied symmetrically.
}

impl Position {
    pub fn new(middle: Point, size: Point, center: Point, halo: Point) -> Self {
        Self {
            middle,
            size,
            center,
            halo,
        }
    }

    pub fn distance_to_left(&self, with_halo: bool) -> f64 {
        self.center().x - self.bbox(with_halo).0.x
    }
    pub fn distance_to_right(&self, with_halo: bool) -> f64 {
        self.bbox(with_halo).1.x - self.center().x
    }
    pub fn left(&self, with_halo: bool) -> f64 {
        self.bbox(with_halo).0.x
    }
    pub fn right(&self, with_halo: bool) -> f64 {
        self.bbox(with_halo).1.x
    }
    pub fn top(&self, with_halo: bool) -> f64 {
        self.bbox(with_halo).0.y
    }
    pub fn bottom(&self, with_halo: bool) -> f64 {
        self.bbox(with_halo).1.y
    }
    // Returns the bounding box of the shape.
    // Include the size of the halo, if \p with_halo is set.
    pub fn bbox(&self, with_halo: bool) -> (Point, Point) {
        let size = self.size(with_halo);
        let top_left = self.middle.sub(size.scale(0.5));
        let bottom_right = top_left.add(size);
        (top_left, bottom_right)
    }

    /// Returns the center of the shape in absolute coordinates.
    pub fn center(&self) -> Point {
        self.middle.add(self.center)
    }

    /// Returns the middle of the shape. (not center!)
    pub fn middle(&self) -> Point {
        self.middle
    }

    pub fn size(&self, with_halo: bool) -> Point {
        if with_halo {
            self.size.add(self.halo)
        } else {
            self.size
        }
    }

    /// \return True if the box fits within the x ranges of \p range.
    pub fn in_x_range(&self, range: (f64, f64), with_halo: bool) -> bool {
        self.left(with_halo) >= range.0 && self.right(with_halo) <= range.1
    }

    pub fn set_size(&mut self, size: Point) {
        self.size = size;
    }

    /// Update the center point for the shape. This is expressed as the delta
    /// from the center of mass (middle-point).
    pub fn set_new_center_point(&mut self, center: Point) {
        self.center = center;
        assert!(self.center.x.abs() < self.size.x);
        assert!(self.center.y.abs() < self.size.y);
    }

    // Move the shape to a new location. The coordinate \p p is the absolute
    // coordinates for new center of the shape.
    pub fn move_to(&mut self, p: Point) {
        let delta = p.sub(self.center());
        self.middle = self.middle.add(delta);
    }

    pub fn align_to_top(&mut self, y: f64) {
        self.middle.y = y + self.size.y / 2. + self.halo.y / 2.
    }
    pub fn align_to_left(&mut self, x: f64) {
        self.middle.x = x + self.size.x / 2. + self.halo.x / 2.
    }
    pub fn align_to_right(&mut self, x: f64) {
        self.middle.x = x - self.size.x / 2. - self.halo.x / 2.;
    }
    // Move the shape in the direction of \p d.
    pub fn translate(&mut self, d: Point) {
        self.middle = self.middle.add(d);
    }

    /// Align the shape to the line \p x, to the right or the left, depending on
    ///  \p to_left.
    pub fn align_x(&mut self, x: f64, to_left: bool) {
        let half_box = self.size.x / 2. + self.halo.x / 2.;
        if to_left {
            self.middle.x = x + half_box;
        } else {
            self.middle.x = x - half_box;
        }
    }

    // Align the center of the shape to \p x.
    pub fn set_x(&mut self, x: f64) {
        self.middle.x = x - self.center.x;
    }

    // Align the center of the shape to \p y.
    pub fn set_y(&mut self, y: f64) {
        self.middle.y = y - self.center.y;
    }

    pub fn transpose(&mut self) {
        self.middle = self.middle.transpose();
        self.size = self.size.transpose();
        self.center = self.center.transpose();
        self.halo = self.halo.transpose();
    }
}

/// \return True if the segment intersects the rect.
pub fn segment_rect_intersection(
    seg: (Point, Point),
    rect: (Point, Point),
) -> bool {
    // Check that the rect is normalized.
    assert!(rect.0.x <= rect.1.x);
    assert!(rect.0.y <= rect.1.y);

    // Check the case of vertical segment:
    if seg.0.x == seg.1.x {
        return seg.1.x >= rect.0.x && seg.1.x <= rect.1.x;
    }

    // Check if the lives are outside of the x range.
    let above = seg.0.x < rect.0.x && seg.1.x < rect.0.x;
    let below = seg.0.x > rect.1.x && seg.1.x > rect.1.x;
    if above || below {
        return false;
    }

    // Check if the lives are outside of the y range.
    let above = seg.0.y < rect.0.y && seg.1.y < rect.0.y;
    let below = seg.0.y > rect.1.y && seg.1.y > rect.1.y;
    if above || below {
        return false;
    }

    // Find the intersection point with the edge of the box.
    //    | o
    //    |/
    //    o  <----- y
    //   /|
    //  / |
    // o  x
    let dx = seg.1.x - seg.0.x; // Can't be zero.
    let dy = seg.1.y - seg.0.y;
    let a = dy / dx;
    // y = a x + b
    // b = y - a * x;
    let b = seg.0.y - a * seg.0.x;

    // Intersect the segment with the two vertical lines of the box.
    let y0 = a * rect.0.x + b;
    let y1 = a * rect.1.x + b;

    // There is no intersection if both hits are on the same side of the box.
    let above = y0 < rect.0.y && y1 < rect.0.y;
    let below = y0 > rect.1.y && y1 > rect.1.y;
    !(above || below)
}

#[test]
fn segment_rect_intersection_test() {
    // Check intersection:
    let v0 = (
        Point::new(-48., -27.),
        Point::new(-196., -55.),
        Point::new(-50., -50.),
        Point::new(50., 50.),
    );
    let v1 = (
        Point::new(-70., -156.),
        Point::new(57., 41.),
        Point::new(-50., -50.),
        Point::new(50., 50.),
    );
    let v2 = (
        Point::new(70., -11.),
        Point::new(-20., -119.),
        Point::new(-50., -50.),
        Point::new(50., 50.),
    );
    assert!(segment_rect_intersection((v0.0, v0.1), (v0.2, v0.3)));
    assert!(segment_rect_intersection((v1.0, v1.1), (v1.2, v1.3)));
    assert!(segment_rect_intersection((v2.0, v2.1), (v2.2, v2.3)));

    // Check no intersection:
    let v0 = (
        Point::new(190., -55.),
        Point::new(173., 199.),
        Point::new(-50., -50.),
        Point::new(50., 50.),
    );
    let v1 = (
        Point::new(142., -19.),
        Point::new(-108., -133.),
        Point::new(-50., -50.),
        Point::new(50., 50.),
    );
    let v2 = (
        Point::new(151., 80.),
        Point::new(17., 124.),
        Point::new(-50., -50.),
        Point::new(50., 50.),
    );
    assert!(!segment_rect_intersection((v0.0, v0.1), (v0.2, v0.3)));
    assert!(!segment_rect_intersection((v1.0, v1.1), (v1.2, v1.3)));
    assert!(!segment_rect_intersection((v2.0, v2.1), (v2.2, v2.3)));
}
