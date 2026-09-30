// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp_Cinert.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepGProp_Cinert`: the global properties of an edge by Gauss integration.
use super::edge_tool::BRepGPropEdgeTool;
use super::gauss_table::gauss_points_and_weights;
use super::line_moments::LineMoments;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Point3;
use rustycad_tkmath::math;
use std::ops::Deref;

/// `BRepGProp_Cinert`: the length, centre and inertia of an edge (a `GProp_GProps`,
/// docs/topalgo-design.md section 9); the inherited members are reached through
/// `Deref`.
///
/// ```
/// use rustycad_tktopalgo::brep_gprop::BRepGPropCinert;
/// let props = BRepGPropCinert::new();
/// assert_eq!(props.mass(), 0.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BRepGPropCinert {
    base: GProps,
}

impl Deref for BRepGPropCinert {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

impl BRepGPropCinert {
    /// `BRepGProp_Cinert()`: an empty framework at the origin; `BRepGPropCinert::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `BRepGProp_Cinert(C, CLocation)`: the properties of `curve` accumulated at
    /// `location`; `BRepGPropCinert::with_curve(&curve, location)?`.
    pub fn with_curve(curve: &BRepAdaptorCurve, location: Point3) -> Result<Self, Failure> {
        let mut props = Self::new();
        props.set_location(location);
        props.perform(curve)?;
        Ok(props)
    }

    /// `SetLocation(CLocation)`; `props.set_location(p)`.
    pub fn set_location(&mut self, location: Point3) {
        self.base.loc = location;
    }

    /// `Perform(C)`: Gauss integration of order `min(IntegrationOrder, GaussPointsMax)`
    /// on each CN interval of the curve (clamped to its range); the centre is the last
    /// point evaluated when the length is below gp::Resolution(); `props.perform(&curve)?`.
    pub fn perform(&mut self, curve: &BRepAdaptorCurve) -> Result<(), Failure> {
        let lower = BRepGPropEdgeTool::first_parameter(curve)?;
        let upper = BRepGPropEdgeTool::last_parameter(curve)?;
        let order = std::cmp::min(
            BRepGPropEdgeTool::integration_order(curve)?,
            math::gauss_points_max(),
        );
        let (points, weights) = gauss_points_and_weights(order)?;
        let spans = cn_spans(curve, std_min(lower, upper), std_max(lower, upper))?;
        let mut totals = LineMoments::default();
        let mut last_point = Point3::default();
        for (span_lower, span_upper) in spans {
            let um = 0.5 * (span_upper + span_lower);
            let ur = 0.5 * (span_upper - span_lower);
            let mut local = LineMoments::default();
            for (point, weight) in points.iter().zip(&weights) {
                let (p, v1) = BRepGPropEdgeTool::d1(curve, um + ur * point)?;
                local.add_point(p.xyz() - self.base.loc.xyz(), v1.magnitude() * weight);
                last_point = p;
            }
            local.scale(ur);
            totals.accumulate(&local);
        }
        totals.store(&mut self.base, last_point);
        Ok(())
    }
}

/// The integration spans of `Perform`: each CN interval clamped to `[uu1, uu2]` when the
/// curve has more than one, else `[uu1, uu2]` itself.
fn cn_spans(curve: &BRepAdaptorCurve, uu1: f64, uu2: f64) -> Result<Vec<(f64, f64)>, Failure> {
    let nb_intervals = BRepGPropEdgeTool::nb_intervals(curve, Shape::CN)?;
    if nb_intervals <= 1 {
        return Ok(vec![(uu1, uu2)]);
    }
    let mut bounds: Array1<'static, f64> = Array1::with_bounds(1, nb_intervals + 1)
        .map_err(|error| error.into_failure("BRepGProp_Cinert::Perform"))?;
    BRepGPropEdgeTool::intervals(curve, &mut bounds, Shape::CN)?;
    Ok((1..=nb_intervals)
        .map(|i| (std_max(bounds[i], uu1), std_min(bounds[i + 1], uu2)))
        .collect())
}
