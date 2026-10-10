// Rust translation of the static KPartCircle and Compute
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The particular cases of an offset wire: a spine of one closed circle (or an open
//! result of one edge) offset directly, and a null offset copying the spine.
use super::GeneratedMap;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Compound, Edge, Face, Shape, ShapeIterator};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dOffsetCurve};
use rustycad_tkg2d::geom2d::{
    Geom2dCircle, Geom2dCurve, Geom2dLine, Geom2dOffsetCurve, Geom2dTrimmedCurve,
};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomPlane, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{self, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeEdge, BRepLibMakeWire};

/// The outputs of the particular case: the shape, the map of parallels and IsDone.
pub(super) type KPartOutputs<'a> = (&'a mut Shape, &'a mut GeneratedMap, &'a mut bool);

/// The single edge of the spine, `None` for none or several.
fn single_edge(spine: &Face) -> Result<Option<Edge>, Failure> {
    let mut found: Option<Edge> = None;
    let mut it = Explorer::new(spine, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while it.more() {
        if found.is_some() {
            return Ok(None);
        }
        found = Some(topods::edge(it.current())?);
        it.next()?;
    }
    Ok(found)
}

/// The basis of a trimmed 3d curve, else the curve.
fn untrimmed(c: Handle<dyn GeomCurve>) -> Handle<dyn GeomCurve> {
    let trimmed: Handle<GeomTrimmedCurve> = c.cast();
    match trimmed.get() {
        Some(t) => t.basis_curve(),
        None => c,
    }
}

/// `KPartCircle(mySpine, myOffset, IsOpenResult, Alt, myShape, myMap, theIsDone)`: a
/// spine of one closed circle edge (or of one edge for an open result) gives the offset
/// curve of its p-curve as the result wire (not done when a circle shrinks to nothing);
/// false when the case does not apply.
pub(super) fn k_part_circle(
    my_spine: &Face,
    my_offset: f64,
    is_open_result: bool,
    alt: f64,
    out: KPartOutputs<'_>,
) -> Result<bool, Failure> {
    let Some(e) = single_edge(my_spine)? else {
        return Ok(false);
    };
    let found = BRepTool::curve_with_location(&e)?;
    if found.curve.is_null() {
        return Ok(false);
    }
    let c: Handle<dyn GeomCurve> = untrimmed(found.curve.clone());
    let is_circle: bool = !c.cast::<GeomCircle>().is_null();
    if !((is_circle && BRepTool::is_closed(e.as_ref())?) || is_open_result) {
        return Ok(false);
    }
    let pc = BRepTool::curve_on_face(&e, my_spine)?;
    let Some(oc) = offset_pcurve(&e, &pc.pcurve, (pc.first, pc.last), my_offset)? else {
        *out.2 = false;
        return Ok(true);
    };
    let a_plane: Handle<dyn GeomSurface> = BRepTool::surface(my_spine)?.cast::<GeomPlane>().cast();
    let mut shape: Shape = BRepLibMakeEdge::from_pcurve_params(&oc, &a_plane, pc.first, pc.last)?
        .edge()?
        .into_shape();
    let (continuity, max_degree, max_segment) = brep_lib::BUILD_CURVE3D_DEFAULTS;
    brep_lib::build_curve3d(
        &topods::edge(&shape)?,
        1.0e-5,
        continuity,
        max_degree,
        max_segment,
    )?;
    shape.set_orientation(e.orientation());
    shape.set_location(&found.location, false)?;
    if alt.abs() > gp::resolution() {
        lift(&mut shape, my_spine, alt)?;
    }
    let mut ll: List<Shape> = List::default();
    ll.append(shape.clone());
    out.1.add(e.into_shape(), ll);
    *out.0 = BRepLibMakeWire::from_edge(&topods::edge(&shape)?)?
        .wire()?
        .into_shape();
    *out.2 = true;
    Ok(true)
}

/// The offset p-curve of the single edge: a line, a circle (`None` when it shrinks to
/// nothing) or a Geom2d_OffsetCurve of the trimmed p-curve.
fn offset_pcurve(
    e: &Edge,
    a_pcurve: &Handle<dyn Geom2dCurve>,
    (f, l): (f64, f64),
    my_offset: f64,
) -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
    let mut an_offset: f64 = my_offset;
    if e.orientation() == Orientation::Forward {
        an_offset *= -1.;
    }
    let ahc: Handle<dyn Adaptor2dCurve2d> =
        Handle::new(Geom2dAdaptorCurve::with_range(a_pcurve, f, l)?).cast();
    let adaptor = ahc.get().ok_or_else(null_pcurve)?;
    match adaptor.get_type()? {
        CurveType::Line => {
            let off = Adaptor2dOffsetCurve::with_offset(&ahc, an_offset)?;
            Ok(Some(
                Handle::new(Geom2dLine::from_lin2d(off.line()?)).cast(),
            ))
        }
        CurveType::Circle => {
            if !BRepTool::is_closed(e.as_ref())? {
                an_offset *= -1.;
            }
            let the_circ = adaptor.circle()?;
            if an_offset > 0. || an_offset.abs() < the_circ.radius() {
                let circle = Geom2dCircle::new(the_circ.position(), the_circ.radius() + an_offset)?;
                return Ok(Some(Handle::new(circle).cast()));
            }
            Ok(None)
        }
        _ => {
            let g2d_t: Handle<dyn Geom2dCurve> =
                Handle::new(Geom2dTrimmedCurve::new(a_pcurve, f, l, true, true)?).cast();
            Ok(Some(
                Handle::new(Geom2dOffsetCurve::new(&g2d_t, an_offset, false)?).cast(),
            ))
        }
    }
}

/// The shape moved by `alt` along the normal of the plane of the spine.
fn lift(shape: &mut Shape, my_spine: &Face, alt: f64) -> Result<(), Failure> {
    let s = BRepAdaptorSurface::with_face(my_spine, false)?;
    let nor = s.plane()?.axis();
    let trans: Vector3 = Vector3::from(nor.direction()) * alt;
    let mut t = Transform3::default();
    t.set_translation(trans);
    shape.move_by(&Location::new(t), false)
}

/// `Compute(Spine, aShape, Map, Alt)`: a null offset, the wires of the spine moved by
/// `Alt` along Z (opposite for a reversed spine), each edge mapped to its copy.
pub(super) fn compute(
    spine: &Face,
    a_shape: &mut Shape,
    map: &mut GeneratedMap,
    alt: f64,
) -> Result<(), Failure> {
    let b = BRepBuilder::default();
    let mut compound = Compound::default();
    b.make_compound(&mut compound);
    let alt: f64 = if spine.orientation() == Orientation::Reversed {
        -alt
    } else {
        alt
    };
    let mut t = Transform3::default();
    t.set_translation(Vector3::new(0., 0., alt));
    let l = Location::new(t);
    let mut exp = Explorer::new(spine, ShapeEnum::Wire, ShapeEnum::Shape)?;
    while exp.more() {
        let cur_w = exp.current().clone();
        let new_w: Shape = cur_w.moved(&l, false)?;
        b.add(&mut compound, &new_w)?;
        map_children(&cur_w, &new_w, map)?;
        exp.next()?;
    }
    *a_shape = compound.into_shape();
    Ok(())
}

/// Each child of the wire mapped to the child of its moved copy.
fn map_children(cur_w: &Shape, new_w: &Shape, map: &mut GeneratedMap) -> Result<(), Failure> {
    let mut it1 = ShapeIterator::new(cur_w, true, true);
    let mut it2 = ShapeIterator::new(new_w, true, true);
    while it1.more() {
        let mut list: List<Shape> = List::default();
        list.append(it2.value()?.clone());
        map.add(it1.value()?.clone(), list);
        it1.next();
        it2.next();
    }
    Ok(())
}

/// Standard_NullObject of a null p-curve.
fn null_pcurve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_OffsetWire: null p-curve of the spine edge; expected a curve on the spine",
        "BRepFill_OffsetWire",
    )
}
