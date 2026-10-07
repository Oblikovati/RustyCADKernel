// Rust translation of the end of BRepFill_OffsetWire::PerformWithBiLo (the merged
// vertices substituted, the vertices moved onto the reference plane and to the ends of
// the 3d curves) (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices of the offset wires: merged, moved onto the reference plane and set at
//! the ends of the 3d curves.
use super::BRepFillOffsetWire;
use super::trim::MergedVertices;
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::Substitution;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Vertex};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tktopalgo::brep_lib;

impl BRepFillOffsetWire {
    /// Each merged vertex replaced by its new vertex (tolerance raised to their
    /// distance) in every parallel, the orientation of the parallel kept.
    pub(super) fn substitute_merged_vertices(
        &mut self,
        map_vv: &MergedVertices,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        for ind in 1..=map_vv.extent() {
            let old_v: Vertex = topods::vertex(map_vv.find_key(ind)?)?;
            let new_v: Vertex = topods::vertex(map_vv.find_from_index(ind)?)?;
            let (p1, p2) = (BRepTool::pnt(&old_v)?, BRepTool::pnt(&new_v)?);
            builder.update_vertex_tolerance(&new_v, p1.distance(&p2))?;
            let mut a_subst = Substitution::new();
            a_subst.substitute(old_v.as_ref(), &[new_v.oriented(Orientation::Forward)])?;
            for j in 1..=self.my_map.extent() {
                for item in self.my_map.change_from_index(j)?.iter_mut() {
                    a_subst.build(item)?;
                    if a_subst.is_copied(item) {
                        let save_or = item.orientation();
                        *item = a_subst
                            .copy(item)?
                            .into_iter()
                            .next()
                            .ok_or_else(not_plane)?;
                        item.set_orientation(save_or);
                    }
                }
            }
        }
        Ok(())
    }

    /// The vertices of the offset moved onto the reference plane, the 3d curves built,
    /// then each vertex set at the end of its first edge curve (the next edges raising
    /// its tolerance to 1.5 times their gap).
    pub(super) fn update_offset_vertices(
        &mut self,
        ref_plane: &Handle<dyn GeomSurface>,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let plane = ref_plane.get().ok_or_else(not_plane)?;
        let mut map_vertex = MapOfShape::default();
        let mut exp = Explorer::new(&self.my_shape, ShapeEnum::Vertex, ShapeEnum::Shape)?;
        while exp.more() {
            let v: Vertex = topods::vertex(exp.current())?;
            if map_vertex.add(v.as_ref().clone()) {
                let p = BRepTool::pnt(&v)?;
                builder.update_vertex_point(
                    &v,
                    plane.value(p.x(), p.y())?,
                    precision::confusion(),
                )?;
            }
            exp.next()?;
        }
        brep_lib::build_curves3d_default(&self.my_shape)?;
        map_vertex.clear(true);
        let mut explo = Explorer::new(&self.my_shape, ShapeEnum::Edge, ShapeEnum::Shape)?;
        while explo.more() {
            snap_edge_vertices(&topods::edge(explo.current())?, &mut map_vertex)?;
            explo.next()?;
        }
        Ok(())
    }
}

/// The vertices of an edge set at the ends of its 3d curve the first time they are met,
/// else their tolerance raised to 1.5 times the gap.
fn snap_edge_vertices(an_edge: &Edge, map_vertex: &mut MapOfShape) -> Result<(), Failure> {
    let (v1, v2) = topexp::vertices(an_edge, false);
    let found = BRepTool::curve_with_location(an_edge)?;
    let copy: Handle<dyn GeomCurve> = found.curve.get().ok_or_else(not_plane)?.copy()?.cast();
    let the_curve = copy.get().ok_or_else(not_plane)?;
    the_curve.transform(found.location.transformation())?;
    let (f3d, l3d) = (the_curve.value(found.first)?, the_curve.value(found.last)?);
    let tv1 = vertex_tshape(v1.as_ref(), "BRepFill_OffsetWire::PerformWithBiLo")?;
    let tv2 = vertex_tshape(v2.as_ref(), "BRepFill_OffsetWire::PerformWithBiLo")?;
    let (dist1, dist2) = (f3d.distance(&tv1.pnt()), l3d.distance(&tv2.pnt()));
    for (v, tv, p, dist) in [(&v1, tv1, f3d, dist1), (&v2, tv2, l3d, dist2)] {
        if !map_vertex.contains(v.as_ref()) {
            tv.set_pnt(p);
            map_vertex.add(v.as_ref().clone());
        } else {
            tv.update_tolerance(1.5 * dist);
        }
    }
    Ok(())
}

/// Standard_NullObject of a missing plane, curve or item (natively a crash).
fn not_plane() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_OffsetWire: null plane, curve or item; expected a non-null value",
        "BRepFill_OffsetWire",
    )
}
