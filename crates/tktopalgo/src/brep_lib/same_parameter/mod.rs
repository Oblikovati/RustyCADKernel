// Rust translation of BRepLib::SameParameter, InternalSameParameter, GetEdgeTol and
// UpdateVTol (BRepLib.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::SameParameter`: the curves on surface of each edge reparametrized (or
//! approximated) to follow the 3d curve, the edge tolerance set to the deviation
//! reached, and the vertex and edge tolerances updated after it.
mod c0_pcurve;
mod edge;
mod measure;

use crate::brep_lib::update_tolerances::{
    ShapeTolerances, internal_update_tolerances, upd_sh_tol, upd_tol_map,
};
use rustycad_tkbrep::brep::access::edge_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::{BRepToolsReShape, ReShape};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d_adaptor::make_curve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomGeometry, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::proj_lib::ProjectedCurve;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::SurfaceType;

pub use edge::same_parameter_edge_with_tolerance;

/// `BRepLib::SameParameter(S, Tolerance, forced)`: every edge of the shape made
/// SameParameter in place (`forced` first clears the flags of the edges that have
/// them), then the vertex tolerances enlarged to the new edge tolerances, the edges of
/// planar faces to the deviation of their projected curves, and the tolerances
/// harmonized; `brep_lib::same_parameter_shape(&shape, 1e-5, false)?`.
pub fn same_parameter_shape(shape: &Shape, tolerance: f64, forced: bool) -> Result<(), Failure> {
    let reshaper = BRepToolsReShape::new();
    internal_same_parameter(shape, &reshaper, tolerance, forced, true)
}

/// `BRepLib::SameParameter(S, theReshaper, Tolerance, forced)`: the same without
/// changing the shape: the modified edges, vertices and faces are copies recorded in
/// the re-shaper; `brep_lib::same_parameter_shape_with_reshape(&shape, &reshaper, 1e-5, false)?`.
pub fn same_parameter_shape_with_reshape(
    shape: &Shape,
    reshaper: &dyn ReShape,
    tolerance: f64,
    forced: bool,
) -> Result<(), Failure> {
    internal_same_parameter(shape, reshaper, tolerance, forced, false)
}

/// `BRepLib::SameParameter(E, Tolerance)`: the edge made SameParameter in place, its
/// vertices enlarged to the new edge tolerance; `brep_lib::same_parameter_edge(&edge, 1e-5)?`.
pub fn same_parameter_edge(edge: &Edge, tolerance: f64) -> Result<(), Failure> {
    let mut new_tol: f64 = -1.;
    same_parameter_edge_with_tolerance(edge, tolerance, &mut new_tol, true)?;
    if new_tol > 0. {
        let (v1, v2) = topexp::vertices(edge, false);
        update_v_tol(&[v1, v2], new_tol)?;
    }
    Ok(())
}

/// `UpdateVTol`: the non-null vertices enlarged to `tol`.
fn update_v_tol(vertices: &[rustycad_tkbrep::topods::Vertex], tol: f64) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    for vertex in vertices.iter().filter(|v| !v.is_null()) {
        builder.update_vertex_tolerance(vertex, tol)?;
    }
    Ok(())
}

/// The static `InternalSameParameter`.
fn internal_same_parameter(
    shape: &Shape,
    reshaper: &dyn ReShape,
    tolerance: f64,
    forced: bool,
    is_mutable_input: bool,
) -> Result<(), Failure> {
    let mut sh_to_tol = ShapeTolerances::default();
    let mut done = MapOfShape::default();
    for shape_edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if done.add(shape_edge.clone()) {
            same_parameter_in_reshape(
                &Edge::unchecked(shape_edge),
                reshaper,
                (tolerance, forced, is_mutable_input),
                &mut sh_to_tol,
            )?;
        }
    }
    planar_edge_tolerances(shape, reshaper, &mut sh_to_tol)?;
    upd_sh_tol(&sh_to_tol, is_mutable_input, reshaper, false)?;
    internal_update_tolerances(shape, false, is_mutable_input, reshaper)
}

/// One edge of `InternalSameParameter`: the edge (or its copy in the re-shaper, a new
/// copy when `forced` clears the flags of an edge that has them) made SameParameter;
/// its vertices take the new edge tolerance in the map.
fn same_parameter_in_reshape(
    old_edge: &Edge,
    reshaper: &dyn ReShape,
    (tolerance, forced, is_mutable_input): (f64, bool, bool),
    sh_to_tol: &mut ShapeTolerances,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut new_edge = Edge::unchecked(reshaper.value(old_edge)?);
    let mut use_old_edge: bool =
        is_mutable_input || reshaper.is_new_shape(old_edge) || !new_edge.is_same(old_edge);
    if forced && (BRepTool::same_range(old_edge)? || BRepTool::same_parameter(old_edge)?) {
        if !use_old_edge {
            new_edge = copied_edge(old_edge)?;
            reshaper.replace(old_edge, &new_edge)?;
            use_old_edge = true;
        }
        builder.same_range(&new_edge, false)?;
        builder.same_parameter(&new_edge, false)?;
    }
    let mut new_tol: f64 = -1.;
    let result =
        same_parameter_edge_with_tolerance(&new_edge, tolerance, &mut new_tol, use_old_edge)?;
    if !use_old_edge && !result.is_null() {
        // NE have been empty-copied
        reshaper.replace(&new_edge, &result)?;
    }
    if new_tol > 0. {
        let (v1, v2) = topexp::vertices(old_edge, false);
        for vertex in [v1, v2].into_iter().filter(|v| !v.is_null()) {
            upd_tol_map(&vertex.into_shape(), new_tol, sh_to_tol)?;
        }
    }
    Ok(())
}

/// An empty copy of the edge with its vertices.
pub(super) fn copied_edge(edge: &Edge) -> Result<Edge, Failure> {
    let builder = BRepBuilder::default();
    let mut copy: Shape = edge.empty_copied()?;
    let mut children = ShapeIterator::new(edge, true, true);
    while let Ok(child) = children.value() {
        builder.add(&mut copy, child)?;
        children.next();
    }
    Ok(Edge::unchecked(copy))
}

/// The edges of the planar faces (each face once): the deviation of the projection of
/// each non-degenerated edge without a stored curve on the plane.
fn planar_edge_tolerances(
    shape: &Shape,
    reshaper: &dyn ReShape,
    sh_to_tol: &mut ShapeTolerances,
) -> Result<(), Failure> {
    let mut done = MapOfShape::default();
    for face_shape in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        if !done.add(face_shape.clone()) {
            continue;
        }
        let face = Face::unchecked(face_shape);
        if BRepAdaptorSurface::with_face(&face, true)?.get_type()? != SurfaceType::Plane {
            continue;
        }
        for edge_shape in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = Edge::unchecked(edge_shape.clone());
            if BRepTool::degenerated(&edge)? {
                continue;
            }
            let new_edge = Edge::unchecked(reshaper.value(&edge)?);
            if let Some(new_etol) = get_edge_tol(&new_edge, &face)? {
                upd_tol_map(&edge_shape, new_etol, sh_to_tol)?;
            }
        }
    }
    Ok(())
}

/// The static `GetEdgeTol`: None when the edge has a curve on the face's plane, else
/// 1.05 times the largest distance between its 3d curve and the image on the plane of
/// its projection, at 24 parameters.
fn get_edge_tol(edge: &Edge, face: &Face) -> Result<Option<f64>, Failure> {
    let (surface, location) = BRepTool::surface_with_location(face)?;
    let relative = location.predivided(edge.location())?;
    let tedge = edge_tshape(edge, "BRepLib::SameParameter")?;
    if tedge.curves().iter().any(|cr| {
        cr.get()
            .is_some_and(|cr| cr.is_curve_on_surface_at(&surface, &relative))
    }) {
        return Ok(None);
    }
    let mut plane = match surface.cast::<GeomRectangularTrimmedSurface>().get() {
        Some(trimmed) => trimmed.basis_surface().cast::<GeomPlane>(),
        None => surface.cast::<GeomPlane>(),
    };
    let c3d = BRepTool::curve_with_location(edge)?;
    let hc = GeomAdaptorCurve::with_curve(&c3d.curve)?;
    let lc = location.predivided(&c3d.location)?;
    if !lc.is_identity() {
        let p = plane
            .get()
            .ok_or_else(|| crate::brep_lib::errors::null_object("GetEdgeTol: not a plane"))?;
        plane = p.transformed(lc.transformation())?.cast::<GeomPlane>();
    }
    let plane_surface: Handle<dyn GeomSurface> = plane.cast();
    let hs: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plane_surface)?).cast();
    let hc: Handle<dyn Adaptor3dCurve> = Handle::new(hc).cast();
    let projection = ProjectedCurve::with_curve(&hs, &hc)?;
    let pc = make_curve(&projection)?;
    let pln = hs.get().expect("a surface adaptor").plane()?;
    Ok(Some(
        1.05 * plane_deviation(&hc, &pc, pln.position(), (c3d.first, c3d.last))?.sqrt(),
    ))
}

/// The largest square distance (below the epsilon of the larger square modulus counts
/// as 0) between the 3d curve and the plane image of the p-curve at 24 parameters.
fn plane_deviation(
    hc: &Handle<dyn Adaptor3dCurve>,
    pc: &Handle<dyn rustycad_tkg2d::geom2d::Geom2dCurve>,
    position: rustycad_tkmath::gp::Axis3,
    (first, last): (f64, f64),
) -> Result<f64, Failure> {
    let hc = hc.get().expect("a curve adaptor");
    let pc = pc
        .get()
        .ok_or_else(|| crate::brep_lib::errors::null_object("GetEdgeTol: null projection"))?;
    let mut d2: f64 = 0.;
    const NN: i32 = 23;
    let unsurnn: f64 = 1. / f64::from(NN);
    for i in 0..=NN {
        let t: f64 = unsurnn * f64::from(i);
        let u: f64 = first * (1. - t) + last * t;
        let pc3d = hc.value(u)?;
        let p2d = pc.value(u)?;
        let pcons = elslib::plane_value(p2d.x(), p2d.y(), position);
        let eps: f64 = epsilon(std_max(
            pc3d.xyz().square_modulus(),
            pcons.xyz().square_modulus(),
        ));
        let mut temp: f64 = pc3d.square_distance(&pcons);
        if temp <= eps {
            temp = 0.;
        }
        if temp > d2 {
            d2 = temp;
        }
    }
    Ok(d2)
}
