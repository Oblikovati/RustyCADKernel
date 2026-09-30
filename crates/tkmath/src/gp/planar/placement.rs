//! Positional members that gp_Circ2d, gp_Elips2d, gp_Hypr2d and gp_Parab2d forward verbatim
//! to their `gp_Ax22d` placement.

/// Members of a planar conic placed by an `Axis22d` stored in `position`.
macro_rules! axis22d_curve_placement {
    ($curve:ty) => {
        impl $curve {
            /// Returns the centre or apex; `curve.location()`.
            pub const fn location(&self) -> $crate::gp::Point2 {
                self.position.location()
            }
            /// Returns the placement frame; `curve.axis()`.
            pub const fn axis(&self) -> $crate::gp::Axis22d {
                self.position
            }
            /// Moves the placement origin; `curve.set_location(point)`.
            pub fn set_location(&mut self, location: $crate::gp::Point2) {
                self.position.set_location(location);
            }
            /// Replaces the placement frame; `curve.set_axis(frame)`.
            pub fn set_axis(&mut self, position: $crate::gp::Axis22d) {
                self.position.set_axis(position);
            }
            /// Tests a counterclockwise frame (`X ^ Y >= 0`); `curve.is_direct()`.
            pub fn is_direct(&self) -> bool {
                self.position
                    .x_direction()
                    .crossed(self.position.y_direction())
                    >= 0.0
            }
            /// Flips the orientation by rebuilding the frame with a reversed Y; `curve.reverse()`.
            pub fn reverse(&mut self) {
                // Native Reverse() is noexcept: it rebuilds gp_Ax22d(P, X, -Y) from the stored
                // unit X, so only the sense of the reversed Y matters and nothing can raise.
                let (x, y) = (self.position.x_direction(), self.position.y_direction());
                let sense = x.crossed(y.reversed()) >= 0.0;
                let frame = $crate::gp::Axis22d::from_unit(self.position.location(), x, sense);
                self.position.set_axis(frame);
            }
            /// Returns the reversed curve; `curve.reversed()`.
            pub fn reversed(mut self) -> Self {
                self.reverse();
                self
            }
            /// Mirrors the frame through a point; `curve.mirrored_point(center)`.
            pub fn mirrored_point(mut self, center: $crate::gp::Point2) -> Self {
                self.position.mirror_point(center);
                self
            }
            /// Mirrors through a point in place; `curve.mirror_point(center)`.
            pub fn mirror_point(&mut self, center: $crate::gp::Point2) {
                self.position.mirror_point(center);
            }
            /// Mirrors the frame about an axis; `curve.mirrored_axis(axis)`.
            pub fn mirrored_axis(mut self, axis: $crate::gp::Axis2d) -> Self {
                self.position.mirror_axis(axis);
                self
            }
            /// Mirrors about an axis in place; `curve.mirror_axis(axis)`.
            pub fn mirror_axis(&mut self, axis: $crate::gp::Axis2d) {
                self.position.mirror_axis(axis);
            }
            /// Rotates the frame about a centre; `curve.rotated(center,angle)`.
            pub fn rotated(mut self, center: $crate::gp::Point2, angle: f64) -> Self {
                self.position.rotate(center, angle);
                self
            }
            /// Rotates in place; `curve.rotate(center,angle)`.
            pub fn rotate(&mut self, center: $crate::gp::Point2, angle: f64) {
                self.position.rotate(center, angle);
            }
            /// Translates the frame origin; `curve.translated(vector)`.
            pub fn translated(mut self, vector: $crate::gp::Vector2) -> Self {
                self.position.translate(vector);
                self
            }
            /// Translates in place; `curve.translate(vector)`.
            pub fn translate(&mut self, vector: $crate::gp::Vector2) {
                self.position.translate(vector);
            }
            /// Translates with gp_Pnt2d's add-then-subtract order; `curve.translated_between(from,to)`.
            pub fn translated_between(
                mut self,
                from: $crate::gp::Point2,
                to: $crate::gp::Point2,
            ) -> Self {
                self.position.translate_between(from, to);
                self
            }
            /// Translates between points in place; `curve.translate_between(from,to)`.
            pub fn translate_between(&mut self, from: $crate::gp::Point2, to: $crate::gp::Point2) {
                self.position.translate_between(from, to);
            }
        }
    };
}
pub(in crate::gp) use axis22d_curve_placement;

/// X/Y axis accessors of the circle, ellipse and hyperbola (the parabola names them differently).
macro_rules! axis22d_curve_axes {
    ($curve:ty) => {
        impl $curve {
            /// Replaces origin and X, keeping the sense; `curve.set_x_axis(axis)?`.
            pub fn set_x_axis(
                &mut self,
                axis: $crate::gp::Axis2d,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.position.set_x_axis(axis)
            }
            /// Replaces origin and Y, keeping the sense; `curve.set_y_axis(axis)?`.
            pub fn set_y_axis(
                &mut self,
                axis: $crate::gp::Axis2d,
            ) -> Result<(), rustycad_tkernel::standard::Failure> {
                self.position.set_y_axis(axis)
            }
            /// Returns the X (major) axis; `curve.x_axis()`.
            pub fn x_axis(&self) -> $crate::gp::Axis2d {
                self.position.x_axis()
            }
            /// Returns the Y (minor) axis; `curve.y_axis()`.
            pub fn y_axis(&self) -> $crate::gp::Axis2d {
                self.position.y_axis()
            }
        }
    };
}
pub(in crate::gp) use axis22d_curve_axes;

/// Implicit-equation coefficients shared by gp_Elips2d and gp_Hypr2d; `sign` is +1 for the
/// ellipse and -1 for the hyperbola, and `a + (-1·b)` equals the native `a - b` exactly.
pub(super) fn central_conic_coefficients(
    major_axis: super::Axis2d,
    major_radius: f64,
    minor_radius: f64,
    sign: f64,
) -> [f64; 6] {
    let minor = minor_radius * minor_radius;
    let major = major_radius * major_radius;
    if minor <= f64::MIN_POSITIVE && major <= f64::MIN_POSITIVE {
        return [0.0; 6];
    }
    let transform = super::Transform2::transformation_to(major_axis);
    let first = [1, 2, 3].map(|column| transform.coefficient(1, column));
    if minor <= f64::MIN_POSITIVE {
        return degenerate_coefficients(first, major);
    }
    let second = [1, 2, 3].map(|column| transform.coefficient(2, column));
    let term =
        |i: usize, j: usize| first[i] * first[j] / major + sign * (second[i] * second[j] / minor);
    [
        term(0, 0),
        term(1, 1),
        term(0, 1),
        term(0, 2),
        term(1, 2),
        term(2, 2) - 1.0,
    ]
}
fn degenerate_coefficients(first: [f64; 3], major: f64) -> [f64; 6] {
    [
        first[0] * first[0],
        first[1] * first[1],
        first[0] * first[1],
        first[0] * first[2],
        first[1] * first[2],
        first[2] * first[2] - major,
    ]
}
