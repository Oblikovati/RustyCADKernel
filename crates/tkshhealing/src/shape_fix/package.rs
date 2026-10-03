//! Package utilities from OCCT ShapeFix.cxx at 3d097a0328e71b8.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Edge, Shape},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;

/// Shape healing package utilities; `ShapeFix::least_edge_size(&shape)?`.
pub struct ShapeFix;
impl ShapeFix {
    /// Measure the least three-sample bounding diagonal; `ShapeFix::least_edge_size(&shape)?`.
    /// Empty shapes return sqrt(RealLast), following native behavior.
    pub fn least_edge_size(shape: &Shape) -> Result<f64, Failure> {
        let mut minimum = f64::MAX;
        let mut edges = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while edges.more() {
            let size = sampled_edge_extent(&topods::edge(edges.current())?)?;
            if size < minimum {
                minimum = size;
            }
            edges.next()?;
        }
        Ok(minimum.sqrt())
    }
    /// Store edge regularity using BRepLib; `ShapeFix::encode_regularity(&shape, 1e-10)?`.
    pub fn encode_regularity(shape: &Shape, angular_tolerance: f64) -> Result<(), Failure> {
        rustycad_tktopalgo::brep_lib::encode_regularity(shape, angular_tolerance)
    }
}
fn sampled_edge_extent(edge: &Edge) -> Result<f64, Failure> {
    let (curve, first, last) = BRepTool::curve(edge)?;
    if curve.is_null() {
        return Ok(f64::MAX);
    }
    let mut bounds = BoundingBox3::default();
    for parameter in [first, last, (last + first) / 2.] {
        bounds.add_point(curve.value(parameter)?);
    }
    let [x1, y1, z1, x2, y2, z2] = bounds.get()?;
    Ok((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1) + (z2 - z1) * (z2 - z1))
}

mod plane_tolerances;
mod same_parameter;

#[cfg(test)]
mod plane_tolerances_tests;
