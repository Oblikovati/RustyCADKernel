use super::BRepOffsetAnalyse;
use crate::brep_offset::BRepOffsetInterval;
use rustycad_tkbrep::{
    brep::BRepTool,
    brep_adaptor::BRepAdaptorSurface,
    brep_tools,
    topexp::{self, Explorer},
    topods::{self, Edge, Face},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::{chfi_ds::TypeOfConcavity, chfi3d};
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dSurface,
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkmath::geom_abs::{Shape as Continuity, SurfaceType};
impl BRepOffsetAnalyse {
    pub(super) fn build_ancestors(&mut self) -> Result<(), Failure> {
        let mut native = IndexedDataMapOfShapeListOfShape::default();
        for (child, parent) in [
            (ShapeEnum::Vertex, ShapeEnum::Edge),
            (ShapeEnum::Edge, ShapeEnum::Face),
        ] {
            topexp::map_shapes_and_unique_ancestors(
                &self.shape,
                child,
                parent,
                &mut native,
                false,
            )?;
        }
        self.ancestors.clear(false);
        for index in 1..=native.extent() {
            self.ancestors.bind(
                native.find_key(index)?.clone(),
                native.find_from_index(index)?.iter().cloned().collect(),
            );
        }
        Ok(())
    }
    pub(super) fn classify_edges(&mut self) -> Result<(), Failure> {
        let shape = self.shape.oriented(Orientation::Forward);
        for shape in Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if self.intervals.is_bound(&shape) {
                continue;
            }
            // Perform intentionally retains previous bindings; only Clear resets the type map.
            self.intervals.bind(shape.clone(), Vec::new());
            let edge = topods::edge(&shape)?;
            let parents = self.ancestors(&shape)?;
            let interval = classify_incidence(&edge, parents, self.angle.sin().abs())?;
            if let Some(interval) = interval {
                self.intervals.change_find(&shape)?.push(interval);
            }
        }
        Ok(())
    }
}
fn classify_incidence(
    edge: &Edge,
    parents: &[topods::Shape],
    sin_tolerance: f64,
) -> Result<Option<BRepOffsetInterval>, Failure> {
    if parents.is_empty() || parents.len() > 2 {
        return Ok(None);
    }
    let first = topods::face(&parents[0])?;
    let (start, end) = BRepTool::range_on_face(edge, &first)?;
    let kind = if parents.len() == 2 {
        classify_pair(edge, &first, &topods::face(&parents[1])?, sin_tolerance)?
    } else if brep_tools::is_really_closed(edge, &first)? {
        TypeOfConcavity::Other
    } else {
        TypeOfConcavity::FreeBound
    };
    Ok(Some(BRepOffsetInterval::new(start, end, kind)))
}
fn classify_pair(
    edge: &Edge,
    first: &Face,
    second: &Face,
    sin_tolerance: f64,
) -> Result<TypeOfConcavity, Failure> {
    let first_type = BRepAdaptorSurface::with_face(first, false)?.get_type()?;
    let second_type = BRepAdaptorSurface::with_face(second, false)?.get_type()?;
    if first_type == SurfaceType::Plane && second_type == SurfaceType::Plane {
        if BRepTool::continuity_on_faces(edge, first, second)? != Continuity::C0 {
            return Ok(TypeOfConcavity::Tangential);
        }
        return chfi3d::define_connect_type(edge, first, second, sin_tolerance, false);
    }
    if spline_surface(first_type)
        && spline_surface(second_type)
        && super::mixed::check_mixed_continuity(edge, first, second, 0.1)?
    {
        return Ok(TypeOfConcavity::Mixed);
    }
    if chfi3d::is_tangent_faces(edge, first, second, Continuity::G1)? {
        return Ok(TypeOfConcavity::Tangential);
    }
    chfi3d::define_connect_type(edge, first, second, sin_tolerance, false)
}
fn spline_surface(kind: SurfaceType) -> bool {
    matches!(
        kind,
        SurfaceType::BezierSurface | SurfaceType::BSplineSurface
    )
}
