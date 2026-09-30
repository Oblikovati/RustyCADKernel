//! ElCLib: values, derivatives, parameters and 3D embeddings of elementary curves.
//!
//! Every static function of the OCCT class is a free function here. Overloads that differ only
//! by dimension carry a `_2d` suffix, `To3d` overloads are `to3d_<kind>`, and the overloads
//! taking gp curves are the generic [`value`], [`d1`], [`d2`], [`d3`], [`dn`] and
//! [`parameter`] functions over [`ElementaryCurve`] and its sub-traits.
mod curves;
mod derivatives;
mod embedding;
mod parameters;
mod periodic;
mod plane;
mod space;
pub use curves::{
    ElementaryCurve, SecondDerivativeCurve, ThirdDerivativeCurve, d1, d2, d3, dn, parameter, value,
};
pub use derivatives::{
    circle_dn, circle_dn_2d, ellipse_dn, ellipse_dn_2d, hyperbola_dn, hyperbola_dn_2d, line_dn,
    line_dn_2d, parabola_dn, parabola_dn_2d,
};
pub use embedding::{
    to3d_axis, to3d_circle, to3d_direction, to3d_ellipse, to3d_frame, to3d_hyperbola, to3d_line,
    to3d_parabola, to3d_point, to3d_vector,
};
pub use parameters::{
    circle_parameter, circle_parameter_2d, ellipse_parameter, ellipse_parameter_2d,
    hyperbola_parameter, hyperbola_parameter_2d, line_parameter, line_parameter_2d,
    parabola_parameter, parabola_parameter_2d,
};
pub(crate) use periodic::normalize_angle;
pub use periodic::{adjust_periodic, in_period};
pub use plane::{
    PlaneD1, PlaneD2, PlaneD3, circle_d1_2d, circle_d2_2d, circle_d3_2d, circle_value_2d,
    ellipse_d1_2d, ellipse_d2_2d, ellipse_d3_2d, ellipse_value_2d, hyperbola_d1_2d,
    hyperbola_d2_2d, hyperbola_d3_2d, hyperbola_value_2d, line_d1_2d, line_value_2d,
    parabola_d1_2d, parabola_d2_2d, parabola_value_2d,
};
pub use space::{
    SpaceD1, SpaceD2, SpaceD3, circle_d1, circle_d2, circle_d3, circle_value, ellipse_d1,
    ellipse_d2, ellipse_d3, ellipse_value, hyperbola_d1, hyperbola_d2, hyperbola_d3,
    hyperbola_value, line_d1, line_value, parabola_d1, parabola_d2, parabola_value,
};
