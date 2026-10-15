// Rust translation of the static functions of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakeDPrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The drafted prism helpers: its history, the length bound (the largest extent of the
//! box), the side of a limiting shape and the test line along the normal of the profile.
use crate::brep_feat::common::sample_barycentre;
use crate::brep_feat::{DescendantMap, parametric_barycenter};
use crate::loc_ope::{CSIntersector, DPrism};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomRectangularTrimmedSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Axis1;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// Maps the first wire of an end shape of the drafted prism (when not null) to its faces.
fn map_end(end: &Shape, the_map: &mut DescendantMap, end_shape: &mut Shape) -> Result<(), Failure> {
    if end.is_null() {
        return Ok(());
    }
    let Some(wire) = Iterator::next(&mut Explorer::new(end, ShapeEnum::Wire, ShapeEnum::Shape)?)
    else {
        return Ok(());
    };
    *end_shape = wire.clone();
    let faces: List<Shape> = Explorer::new(end, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    the_map.bind(wire, faces);
    Ok(())
}

/// `MajMap(theB, theP, myMap, myFShape, myLShape)`: the end wires of the drafted prism
/// mapped to their faces, and each edge of the base not mapped yet to the faces it
/// sweeps.
pub(super) fn maj_map(
    the_b: &Shape,
    the_p: &DPrism,
    the_map: &mut DescendantMap,
    the_f_shape: &mut Shape,
    the_l_shape: &mut Shape,
) -> Result<(), Failure> {
    map_end(the_p.first_shape(), the_map, the_f_shape)?;
    map_end(the_p.last_shape(), the_map, the_l_shape)?;
    for e in Explorer::new(the_b, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !the_map.is_bound(&e) {
            let swept = the_p.shapes(&e)?;
            the_map.bind(e, swept);
        }
    }
    Ok(())
}

/// `HeightMax(theSbase, theSkface, theSFrom, theSUntil)`: the largest extent of the box
/// of the basis, the sketch face and the limiting shapes.
pub(super) fn height_max(
    the_sbase: &Shape,
    the_skface: &Face,
    the_s_from: &Shape,
    the_s_until: &Shape,
) -> Result<f64, Failure> {
    let mut b = BoundingBox3::default();
    BRepBndLib::add(the_sbase, &mut b, true)?;
    BRepBndLib::add(the_skface, &mut b, true)?;
    for s in [the_s_from, the_s_until] {
        if !s.is_null() {
            BRepBndLib::add(s, &mut b, true)?;
        }
    }
    let [xmin, ymin, zmin, xmax, ymax, zmax] = b.get()?;
    Ok((xmax - xmin)
        .abs()
        .max((ymax - ymin).abs())
        .max((zmax - zmin).abs()))
}

/// `SensOfPrism(C, Until)`: -1 when the shape lies strictly before the start of the
/// curve (both extreme intersections, or the parametric barycentre), else 1.
pub(super) fn sens_of_prism(c: &Handle<dyn GeomCurve>, until: &Shape) -> Result<i32, Failure> {
    let mut asi1 = CSIntersector::with_shape(until);
    let mut scur = Sequence::default();
    scur.append(c.clone());
    asi1.perform_curves(&scur)?;
    if asi1.is_done() && asi1.nb_points(1)? >= 1 {
        let nb = asi1.nb_points(1)?;
        let (prm1, prm2) = (
            asi1.point(1, 1)?.parameter(),
            asi1.point(1, nb)?.parameter(),
        );
        return Ok(if prm1 < 0. && prm2 < 0. { -1 } else { 1 });
    }
    if parametric_barycenter(until, c)? < 0. {
        return Ok(-1);
    }
    Ok(1)
}

/// `TestCurve(Base)`: the line through the barycentre of the sample points of the
/// profile along the normal of its plane (null for a profile not planar).
pub(super) fn test_curve(base: &Face) -> Result<Handle<dyn GeomCurve>, Failure> {
    let bar = sample_barycentre(base)?;
    let mut s = BRepTool::surface(base)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    if let Some(t) = trimmed.get() {
        s = t.basis_surface();
    }
    let plane: Handle<GeomPlane> = s.cast();
    let Some(p) = plane.get() else {
        return Ok(Handle::default());
    };
    let pln = p.pln();
    let pos = pln.position();
    let normale = pos.x_direction().crossed(pos.y_direction())?;
    Ok(Handle::new(GeomLine::new(Axis1::new(bar, normale))).cast())
}
