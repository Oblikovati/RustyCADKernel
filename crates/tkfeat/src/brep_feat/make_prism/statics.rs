// Rust translation of the static functions of
// ModelingAlgorithms/TKFeat/BRepFeat/BRepFeat_MakePrism.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The prism helpers: the history of the prism, the length bound of the long prisms,
//! the side of a limiting shape along the prism and its test line.
use crate::brep_feat::common::sample_barycentre;
use crate::brep_feat::{DescendantMap, parametric_barycenter};
use crate::loc_ope::{CSIntersector, Prism};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Face, Shape};
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Axis1, Direction3};
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;

/// Maps the first or last shape of the prism: its first wire to its faces.
fn map_end(end: &Shape, the_map: &mut DescendantMap, end_shape: &mut Shape) -> Result<(), Failure> {
    let Some(wire) = Iterator::next(&mut Explorer::new(end, ShapeEnum::Wire, ShapeEnum::Shape)?)
    else {
        return Ok(());
    };
    *end_shape = wire.clone();
    let faces: List<Shape> = Explorer::new(end, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    the_map.bind(wire, faces);
    Ok(())
}

/// `MajMap(theB, theP, myMap, myFShape, myLShape)`: the first and last wires of the
/// prism mapped to their faces, and each edge of the base not mapped yet to the faces it
/// sweeps.
pub(super) fn maj_map(
    the_b: &Shape,
    the_p: &Prism,
    the_map: &mut DescendantMap,
    the_f_shape: &mut Shape,
    the_l_shape: &mut Shape,
) -> Result<(), Failure> {
    map_end(the_p.first_shape(), the_map, the_f_shape)?;
    map_end(the_p.last_shape(), the_map, the_l_shape)?;
    for e in Explorer::new(the_b, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !the_map.is_bound(&e) {
            let swept = the_p.shapes(&e)?.clone();
            the_map.bind(e, swept);
        }
    }
    Ok(())
}

/// Whether a limiting shape has an edge without vertex (an infinite face of
/// revolution), whose box is then left out.
fn has_infinite_edge(s: &Shape) -> Result<bool, Failure> {
    for e in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !Explorer::new(&e, ShapeEnum::Vertex, ShapeEnum::Shape)?.more() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `HeightMax(theSbase, theSkface, theSFrom, theSUntil)`: twice the spread of the box
/// coordinates of the basis, the sketch face and the finite limiting shapes.
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
        if !s.is_null() && !has_infinite_edge(s)? {
            BRepBndLib::add(s, &mut b, true)?;
        }
    }
    let [xmin, ymin, zmin, xmax, ymax, zmax] = b.get()?;
    let c = [xmin, xmax, ymin, ymax, zmin, zmax];
    let (mut parmin, mut parmax) = (c[0], c[0]);
    for v in c {
        if v > parmax {
            parmax = v;
        }
        if v < parmin {
            parmin = v;
        }
    }
    Ok((2. * (parmax - parmin)).abs())
}

/// `SensOfPrism(C, Until)`: -1 when the shape lies before the start of the curve
/// (both extreme intersections, or the parametric barycentre without intersection),
/// else 1.
pub(super) fn sens_of_prism(c: &Handle<dyn GeomCurve>, until: &Shape) -> Result<i32, Failure> {
    let mut asi1 = CSIntersector::with_shape(until);
    let mut scur = Sequence::default();
    scur.append(c.clone());
    asi1.perform_curves(&scur)?;
    if asi1.is_done() && asi1.nb_points(1)? >= 1 {
        let n = asi1.nb_points(1)?;
        let tol = precision::confusion();
        if asi1.point(1, 1)?.parameter() + tol < 0. && asi1.point(1, n)?.parameter() + tol < 0. {
            return Ok(-1);
        }
        return Ok(1);
    }
    if parametric_barycenter(until, c)? < 0. {
        return Ok(-1);
    }
    Ok(1)
}

/// `TestCurve(Base, V)`: the line along `v` through the barycentre of the sample points
/// of the base.
pub(super) fn test_curve(base: &Shape, v: Direction3) -> Result<Handle<dyn GeomCurve>, Failure> {
    let bar = sample_barycentre(base)?;
    Ok(Handle::new(GeomLine::new(Axis1::new(bar, v))).cast())
}
