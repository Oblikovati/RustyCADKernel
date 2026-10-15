// Rust translation of the static functions of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx (RemoveCorks,
// IsConnectedShell, MakeList, EvalMax, OrientationOfEdgeInFace, FindParameter,
// GetEdgePoints, FillContours, ComputeMaxDist, UpdateTolerance, CorrectSolid,
// checkSinglePoint, RemoveShapes, UpdateHistory, TrimEdges, TrimEdge, GetEnlargedFaces,
// BuildShellsCompleteInter, GetSubShapes, RemoveSeamAndDegeneratedEdges, IsSolid,
// AppendToList) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static services of the offset algorithm.
use crate::brep_offset::inter3d::{OffsetFaces, ShapeShapeMap};
use crate::brep_offset::tool::live;
use crate::brep_offset::{BRepOffsetAnalyse, BRepOffsetError, BRepOffsetInter2d, BRepOffsetTool};
use rustycad_tkbo::bop_algo::BOPAlgoMakerVolume;
use rustycad_tkbo::bop_tools::make_split_edge;
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_tools::{self, Quilt, WireExplorer};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{
    self, Compound, Edge, Face, Shape, ShapeIterator, Solid, Vertex, Wire,
};
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_check::{BRepCheckEdge, BRepCheckVertex};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use rustycad_tktopalgo::brep_lib;

/// A new compound.
pub(super) fn compound() -> Shape {
    let mut c = Compound::default();
    BRepBuilder::default().make_compound(&mut c);
    c.into_shape()
}

/// `RemoveCorks(S, Faces)`: the compound of the faces of the shape that are not caps;
/// the caps re-added to the map with their orientation in the shape.
pub(super) fn remove_corks(s: &mut Shape, faces: &mut IndexedMapOfShape) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut ss = compound();
    // The shape without caps, the caps oriented as in the shape.
    for cork in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        if !faces.contains(&cork) {
            builder.add(&mut ss, &cork)?;
        } else {
            faces.remove_key(&cork);
            // To reset it with the proper orientation.
            faces.add(cork);
        }
    }
    *s = ss;
    Ok(())
}

/// `IsConnectedShell(S)`: whether the faces glue into one shell.
pub(super) fn is_connected_shell(s: &Shape) -> Result<bool, Failure> {
    let mut glue = Quilt::new();
    glue.add(s)?;
    let ss = glue.shells()?;
    let mut explo = Explorer::new(&ss, ShapeEnum::Shell, ShapeEnum::Shape)?;
    explo.next()?;
    Ok(!explo.more())
}

/// `MakeList(OffsetFaces, myInitOffsetFace, myFaces)`: the images of the roots that are
/// not caps.
pub(super) fn make_list(
    offset_faces: &mut List<Shape>,
    my_init_offset_face: &BRepAlgoImage,
    my_faces: &IndexedMapOfShape,
) {
    for root in my_init_offset_face.roots() {
        if !my_faces.contains(root) && my_init_offset_face.has_image(root) {
            for s in my_init_offset_face.image(root) {
                offset_faces.append(s);
            }
        }
    }
}

/// `EvalMax(S, Tol)`: the tolerance raised to the greatest vertex tolerance.
pub(super) fn eval_max(s: &Shape, tol: &mut f64) -> Result<(), Failure> {
    for v in Explorer::new(s, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let tol_v = BRepTool::vertex_tolerance(&topods::vertex(&v)?)?;
        if tol_v > *tol {
            *tol = tol_v;
        }
    }
    Ok(())
}

/// `OrientationOfEdgeInFace(theEdge, theFace)`: the orientation of the edge in the
/// face, EXTERNAL when absent.
pub(super) fn orientation_of_edge_in_face(
    the_edge: &Shape,
    the_face: &Shape,
) -> Result<Orientation, Failure> {
    for an_edge in Explorer::new(the_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if an_edge.is_same(the_edge) {
            return Ok(an_edge.orientation());
        }
    }
    Ok(Orientation::External)
}

/// `FindParameter(V, E, U)`: the parameter of the vertex on the edge (its range end
/// for a FORWARD or REVERSED vertex of the edge, else the stored point on the 3d curve
/// or on the first p-curve, adjusted on closed curves); `None` without one.
pub(super) fn find_parameter(v: &Vertex, e: &Edge) -> Result<Option<f64>, Failure> {
    // The vertex in the edge.
    let mut rev = false;
    let mut vf = Shape::default();
    let mut orient = Orientation::Internal;
    let mut itv = ShapeIterator::new(&e.oriented(Orientation::Forward), true, true);
    // An edge without vertices and degenerated: the vertex orientation.
    if !itv.more() && BRepTool::degenerated(e)? {
        orient = v.orientation();
    }
    while itv.more() {
        let vcur = itv.value()?.clone();
        if v.is_same(&vcur) {
            if vf.is_null() {
                vf = vcur;
            } else {
                rev = e.orientation() == Orientation::Reversed;
                if vcur.orientation() == v.orientation() {
                    vf = vcur;
                }
            }
        }
        itv.next();
    }
    if !vf.is_null() {
        orient = vf.orientation();
    }
    if orient == Orientation::Forward {
        let (f, l) = BRepTool::range(e)?;
        return Ok(Some(if rev { l } else { f }));
    }
    if orient == Orientation::Reversed {
        let (f, l) = BRepTool::range(e)?;
        return Ok(Some(if rev { f } else { l }));
    }
    let tv = vertex_tshape(v.as_ref(), "FindParameter")?;
    let found = BRepTool::curve_with_location(e)?;
    let (c, f, l) = (found.curve.clone(), found.first, found.last);
    if !c.is_null() || BRepTool::degenerated(e)? {
        // The location relative to the vertex (the edge location composed with the
        // representation one, divided by the vertex location).
        let loc = found.location.predivided(v.location())?;
        for pr in tv.points() {
            let p_rep = live(&pr, "point representation")?;
            if !p_rep.is_point_on_curve_at(&c, &loc) {
                continue;
            }
            let p = p_rep.parameter();
            let mut res = p;
            if let Some(curve) = c.get() {
                // Closed curves.
                if precision::is_negative_infinite(f) || precision::is_positive_infinite(l) {
                    return Ok(Some(p_rep.parameter()));
                }
                let pf = curve.value(f)?.transformed(loc.transformation());
                let pl = curve.value(l)?.transformed(loc.transformation());
                let tol = BRepTool::vertex_tolerance(v)?;
                if pf.distance(&pl) < tol && pf.distance(&BRepTool::pnt(v)?) < tol {
                    res = if v.orientation() == Orientation::Forward {
                        f
                    } else {
                        l
                    };
                }
            }
            return Ok(Some(res));
        }
    } else {
        // No 3d curve: the first p-curve.
        let cos = BRepTool::first_curve_on_surface(e)?;
        let loc = cos.location.predivided(v.location())?;
        for pr in tv.points() {
            let p_rep = live(&pr, "point representation")?;
            if !p_rep.is_point_on_curve_on_surface_at(&cos.pcurve, &cos.surface, &loc) {
                continue;
            }
            let mut p = p_rep.parameter();
            let pc = live(&cos.pcurve, "pcurve")?;
            // Closed curves.
            if pc.is_closed()? && (p == pc.first_parameter() || p == pc.last_parameter()) {
                p = if v.orientation() == Orientation::Forward {
                    pc.first_parameter()
                } else {
                    pc.last_parameter()
                };
            }
            return Ok(Some(p));
        }
    }
    Ok(None)
}

/// `GetEdgePoints(anEdge, aFace, fPnt, mPnt, lPnt)`: the first, middle and last points
/// of the p-curve of the edge on the surface of the face.
pub(super) fn get_edge_points(
    an_edge: &Edge,
    a_face: &Face,
) -> Result<(Point3, Point3, Point3), Failure> {
    let pc = BRepTool::curve_on_face(an_edge, a_face)?;
    let c = live(&pc.pcurve, "pcurve")?;
    let (f, l) = (pc.first, pc.last);
    let (f2d, l2d, m2d) = (c.value(f)?, c.value(l)?, c.value(0.5 * (f + l))?);
    let a_surf = BRepTool::surface(a_face)?;
    let s = live(&a_surf, "surface")?;
    Ok((
        s.value(f2d.x(), f2d.y())?,
        s.value(m2d.x(), m2d.y())?,
        s.value(l2d.x(), l2d.y())?,
    ))
}

/// `FillContours(aShape, Analyser, Contours, MapEF)`: the free boundary edges (with
/// their face) chained into contours keyed by their start vertex.
pub(super) fn fill_contours(
    a_shape: &Shape,
    analyser: &BRepOffsetAnalyse,
    contours: &mut IndexedDataMapOfShapeListOfShape,
    map_ef: &mut ShapeShapeMap,
) -> Result<(), Failure> {
    let mut edges: Vec<Shape> = Vec::new();
    for fs in Explorer::new(a_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let a_face = topods::face(&fs)?;
        let mut itf = ShapeIterator::new(&fs, true, true);
        while itf.more() {
            let a_wire = topods::wire(itf.value()?)?;
            itf.next();
            let mut wexp = WireExplorer::with_wire_on_face(&a_wire, &a_face)?;
            while wexp.more() {
                let an_edge = wexp.current().clone();
                wexp.next()?;
                if BRepTool::degenerated(&an_edge)? {
                    continue;
                }
                let lint = analyser.type_of(&an_edge)?;
                if let Some(first) = lint.first()
                    && first.get_type()? == TypeOfConcavity::FreeBound
                {
                    map_ef.bind(an_edge.as_ref().clone(), fs.clone());
                    edges.push(an_edge.as_ref().clone());
                }
            }
        }
    }
    while !edges.is_empty() {
        let start_edge = topods::edge(&edges.remove(0))?;
        let (start_vertex, mut cur_vertex) = topexp::vertices(&start_edge, true);
        let mut a_contour = List::default();
        a_contour.append(start_edge.as_ref().clone());
        while !cur_vertex.is_same(&start_vertex) {
            for k in 0..edges.len() {
                let an_edge = topods::edge(&edges[k])?;
                let (v1, v2) = topexp::vertices(&an_edge, false);
                if v1.is_same(&cur_vertex) || v2.is_same(&cur_vertex) {
                    a_contour.append(an_edge.as_ref().clone());
                    cur_vertex = if v1.is_same(&cur_vertex) { v2 } else { v1 };
                    edges.remove(k);
                    break;
                }
            }
        }
        contours.add(start_vertex.as_ref().clone(), a_contour);
    }
    Ok(())
}

/// `ComputeMaxDist(thePlane, theCrv, theFirst, theLast)`: 1.05 times the greatest
/// distance of 23 points of the curve to the plane (infinite at an infinite point).
pub(super) fn compute_max_dist(
    the_plane: &Plane3,
    the_crv: &Handle<dyn GeomCurve>,
    the_first: f64,
    the_last: f64,
) -> Result<f64, Failure> {
    const NCONTROL: i32 = 23;
    let c = live(the_crv, "curve")?;
    let mut a_max_dist = 0.;
    for i in 0..NCONTROL {
        let a_prm = (f64::from(NCONTROL - 1 - i) * the_first + f64::from(i) * the_last)
            / f64::from(NCONTROL - 1);
        let a_p = c.value(a_prm)?;
        if precision::is_infinite(a_p.x())
            || precision::is_infinite(a_p.y())
            || precision::is_infinite(a_p.z())
        {
            return Ok(precision::infinite());
        }
        let a_dist2 = the_plane.square_distance(a_p);
        if a_dist2 > a_max_dist {
            a_max_dist = a_dist2;
        }
    }
    Ok(a_max_dist.sqrt() * 1.05)
}

/// `UpdateTolerance(S, Faces, theInitShape)`: the tolerances of the edges of the faces
/// (neither caps nor initial faces) raised to their distances to their planes and to
/// their curve deviations, then their vertices recomputed (their points cleaned).
pub(super) fn update_tolerance(
    s: &mut Shape,
    faces: &IndexedMapOfShape,
    the_init_shape: &Shape,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut view = MapOfShape::default();
    // The edges of the caps are not modified.
    for j in 1..=faces.extent() {
        for e in Explorer::new(faces.find_key(j)?, ShapeEnum::Edge, ShapeEnum::Shape)? {
            view.add(e);
        }
    }
    // The edges of the initial shape are not modified.
    let mut a_map_init_f = MapOfShape::default();
    if !the_init_shape.is_null() {
        for f in Explorer::new(the_init_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            a_map_init_f.add(f.clone());
            for e in Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)? {
                view.add(e.clone());
                let mut it_v = ShapeIterator::new(&e, true, true);
                while it_v.more() {
                    view.add(it_v.value()?.clone());
                    it_v.next();
                }
            }
        }
    }
    for f in Explorer::new(s, ShapeEnum::Face, ShapeEnum::Shape)? {
        if faces.contains(&f) || a_map_init_f.contains(&f) {
            continue;
        }
        let a_bas = BRepAdaptorSurface::with_face(&topods::face(&f)?, false)?;
        for es in Explorer::new(&f, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let e = topods::edge(&es)?;
            let mut is_updated = false;
            let a_curr_tol = BRepTool::edge_tolerance(&e)?;
            if a_bas.get_type()? == SurfaceType::Plane {
                // The edge seems to have no p-curve on the plane, so the edge corrector
                // does not include it in the tolerance.
                let (a_crv, a_first, a_last) = BRepTool::curve(&e)?;
                let a_max_dist = compute_max_dist(&a_bas.plane()?, &a_crv, a_first, a_last)?;
                if a_max_dist > a_curr_tol {
                    builder.update_edge_tolerance(&e, a_max_dist)?;
                    is_updated = true;
                }
            }
            if view.add(es.clone()) {
                e.tshape().set_locked(false);
                let edge_corrector = BRepCheckEdge::new(&e)?;
                let tol = edge_corrector.tolerance()?;
                if tol > a_curr_tol {
                    builder.update_edge_tolerance(&e, tol)?;
                    is_updated = true;
                }
            }
            if is_updated {
                let tol = BRepTool::edge_tolerance(&e)?;
                // The update of the vertices.
                let (va, vb) = topexp::vertices(&e, false);
                for v in [va, vb] {
                    v.tshape().set_locked(false);
                    if view.add(v.as_ref().clone()) {
                        let tv = vertex_tshape(v.as_ref(), "UpdateTolerance")?;
                        tv.set_tolerance(0.);
                        let vertex_corrector = BRepCheckVertex::new(&v)?;
                        builder.update_vertex_tolerance(&v, vertex_corrector.tolerance()?)?;
                        // The occasion to clean the vertices.
                        tv.change_points(|p| p.clear());
                    }
                    builder.update_vertex_tolerance(&v, tol)?;
                }
            }
        }
    }
    Ok(())
}

/// `CorrectSolid(theSol, theSolList)`: the shell of the greatest volume made the outer
/// shell (positive), the shells inside it kept as voids, the others made solids.
pub(super) fn correct_solid(
    the_sol: &mut Shape,
    the_sol_list: &mut List<Shape>,
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut an_outer_shell = Shape::default();
    let mut a_vols = Vec::new();
    let (mut a_vol_max, mut an_outer_vol) = (0., 0.);
    let mut an_it = ShapeIterator::new(the_sol, true, true);
    while an_it.more() {
        let a_sh = an_it.value()?.clone();
        an_it.next();
        let mut a_v_props = GProps::new();
        BRepGProp::volume_properties(&a_sh, &mut a_v_props, true, false, false)?;
        if a_v_props.mass().abs() > a_vol_max {
            an_outer_vol = a_v_props.mass();
            a_vol_max = an_outer_vol.abs();
            an_outer_shell = a_sh.clone();
        }
        a_vols.push(a_v_props.mass());
    }
    if an_outer_vol.abs() < precision::confusion() {
        return Ok(());
    }
    if an_outer_vol < 0. {
        an_outer_shell.reverse();
    }
    let mut new_sol = Solid::default();
    builder.make_solid(&mut new_sol);
    let mut a_new_sol = new_sol.into_shape();
    a_new_sol.tshape().set_closed(true);
    builder.add(&mut a_new_sol, &an_outer_shell)?;
    let mut a_sol_class = BRepClass3dSolidClassifier::with_shape(&a_new_sol)?;
    let mut an_it = ShapeIterator::new(the_sol, true, true);
    let mut k = 0;
    while an_it.more() {
        let mut a_sh = topods::shell(an_it.value()?)?.into_shape();
        an_it.next();
        let vol = a_vols[k];
        k += 1;
        if a_sh.is_same(&an_outer_shell) {
            continue;
        }
        let a_v = topods::vertex(
            &Explorer::new(&a_sh, ShapeEnum::Vertex, ShapeEnum::Shape)?
                .current()
                .clone(),
        )?;
        let a_p = BRepTool::pnt(&a_v)?;
        a_sol_class.perform(&a_p, BRepTool::vertex_tolerance(&a_v)?)?;
        if a_sol_class.state() == State::In {
            if vol > 0. {
                a_sh.reverse();
            }
            builder.add(&mut a_new_sol, &a_sh)?;
        } else {
            if vol < 0. {
                a_sh.reverse();
            }
            let mut sol = Solid::default();
            builder.make_solid(&mut sol);
            let mut a_sol = sol.into_shape();
            a_sol.tshape().set_closed(true);
            builder.add(&mut a_sol, &a_sh)?;
            the_sol_list.append(a_sol);
        }
    }
    *the_sol = a_new_sol;
    Ok(())
}

/// `checkSinglePoint(theUParam, theVParam, theSurf, theBadPoints)`: bad normals where a
/// derivative vanishes (out of the known degenerated points) or the isolines are
/// parallel.
pub(super) fn check_single_point(
    the_u_param: f64,
    the_v_param: f64,
    the_surf: &Handle<dyn GeomSurface>,
    the_bad_points: &[Point3],
) -> Result<BRepOffsetError, Failure> {
    let d1 = live(the_surf, "surface")?.d1(the_u_param, the_v_param)?;
    if d1.d1u.square_magnitude() < precision::square_confusion()
        || d1.d1v.square_magnitude() < precision::square_confusion()
    {
        let is_known_bad_pnt = the_bad_points
            .iter()
            .any(|b| d1.point.square_distance(b) < precision::square_confusion());
        return Ok(if is_known_bad_pnt {
            BRepOffsetError::NoError
        } else {
            BRepOffsetError::BadNormalsOnGeometry
        });
    }
    if d1.d1u.is_parallel(d1.d1v, precision::confusion())? {
        // The isolines are collinear.
        return Ok(BRepOffsetError::BadNormalsOnGeometry);
    }
    Ok(BRepOffsetError::NoError)
}

/// `RemoveShapes(theS, theLS)`: the sub-shapes removed from the shape (its free flag
/// kept).
pub(super) fn remove_shapes(the_s: &mut Shape, the_ls: &List<Shape>) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let b_free = the_s.tshape().free();
    the_s.tshape().set_free(true);
    for a_si in the_ls.iter() {
        builder.remove(the_s, a_si)?;
    }
    the_s.tshape().set_free(b_free);
    Ok(())
}

/// `UpdateHistory(theLF, theGF, theImage)`: the splits of the faces in the image.
pub(super) fn update_history(
    the_lf: &List<Shape>,
    the_gf: &BOPAlgoMakerVolume,
    the_image: &mut BRepAlgoImage,
) -> Result<(), Failure> {
    for a_f in the_lf.iter() {
        let a_lf_im = the_gf.modified(a_f)?;
        if !a_lf_im.is_empty() {
            if the_image.has_image(a_f) {
                the_image.add_many(a_f, &a_lf_im)?;
            } else {
                the_image.bind_many(a_f, &a_lf_im)?;
            }
        }
    }
    Ok(())
}

/// `AppendToList(theList, theShape)`: the shape appended unless already there.
pub(super) fn append_to_list(the_list: &mut List<Shape>, the_shape: &Shape) {
    if the_list.iter().any(|s| s.is_same(the_shape)) {
        return;
    }
    the_list.append(the_shape.clone());
}

/// The origins of an offset edge (`theEdgesOrigins`).
pub(super) type EdgesOrigins = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// `TrimEdges(theShape, theOffset, Analyse, theMapSF, theMES, theBuild, theAsDes,
/// theAsDes2d, theNewEdges, theETrimEInf, theEdgesOrigins)`: the new edges of the faces
/// (and of the generated faces) trimmed at their outermost 2d vertices, recorded with
/// their origins; false when a trimming fails.
#[allow(clippy::too_many_arguments)]
pub(super) fn trim_edges(
    the_shape: &Shape,
    the_offset: f64,
    analyse: &BRepOffsetAnalyse,
    the_map_sf: &OffsetFaces,
    the_mes: &ShapeShapeMap,
    the_build: &ShapeShapeMap,
    the_as_des: &BRepAlgoAsDes,
    the_as_des2d: &BRepAlgoAsDes,
    the_new_edges: &mut IndexedMapOfShape,
    the_e_trim_e_inf: &mut ShapeShapeMap,
    the_edges_origins: &mut EdgesOrigins,
) -> Result<bool, Failure> {
    let mut a_l_faces: Vec<Shape> =
        Explorer::new(the_shape, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    let mut a_mf_generated = MapOfShape::default();
    let mut a_dmef = IndexedDataMapOfShapeListOfShape::default();
    for a_fg in analyse.new_faces() {
        a_l_faces.push(a_fg.clone());
        a_mf_generated.add(a_fg.clone());
        topexp::map_shapes_and_unique_ancestors(
            a_fg,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut a_dmef,
            false,
        )?;
    }
    for fi_s in &a_l_faces {
        let o = the_map_sf.seek(fi_s).ok_or_else(not_bound)?;
        let mut nf = o.face().as_ref().clone();
        if let Some(m) = the_mes.seek(&nf) {
            nf = m.clone();
        }
        let mut view = MapOfShape::default();
        let mut ve_map = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(
            &fi_s.oriented(Orientation::Forward),
            ShapeEnum::Edge,
            &mut ve_map,
        )?;
        topexp::map_shapes_of_type(
            &fi_s.oriented(Orientation::Forward),
            ShapeEnum::Vertex,
            &mut ve_map,
        )?;
        for i in 1..=ve_map.extent() {
            let a_s = ve_map.find_key(i)?.clone();
            if !view.add(a_s.clone()) {
                continue;
            }
            if let Some(ne) = the_build.seek(&a_s).cloned() {
                // The connection to the original edges.
                for nec in Explorer::new(&ne, ShapeEnum::Edge, ShapeEnum::Shape)? {
                    append_to_list(the_edges_origins.try_emplaced(nec, List::default), &a_s);
                }
                // The trimming of the edges.
                if ne.shape_type()? == ShapeEnum::Edge {
                    // NCollection_IndexedMap::Add returns the index of the key, present or not: the
                    // native condition always holds.
                    the_new_edges.add(ne.clone());
                    {
                        let mut e = topods::edge(&ne)?;
                        if !trim_edge(&mut e, the_as_des2d, the_as_des, the_e_trim_e_inf)? {
                            return Ok(false);
                        }
                    }
                } else {
                    // The intersections are on several edges: the pieces without
                    // intersection with their neighbours are removed from the AsDes.
                    for nec_s in Explorer::new(&ne, ShapeEnum::Edge, ShapeEnum::Shape)? {
                        let mut nec = topods::edge(&nec_s)?;
                        the_new_edges.add(nec_s.clone());
                        {
                            if the_as_des2d.descendant(&nec_s).next().is_some() {
                                if !trim_edge(&mut nec, the_as_des2d, the_as_des, the_e_trim_e_inf)?
                                {
                                    return Ok(false);
                                }
                            } else if the_as_des.has_ascendant(&nec_s) {
                                the_as_des.remove_shared(&nec_s)?;
                            }
                        }
                    }
                }
            } else {
                if a_s.shape_type()? != ShapeEnum::Edge {
                    continue;
                }
                if a_mf_generated.contains(fi_s) && a_dmef.find_from_key(&a_s)?.len() == 1 {
                    continue;
                }
                let mut ne = o.generated(&a_s);
                ne.set_orientation(a_s.orientation());
                append_to_list(
                    the_edges_origins.try_emplaced(ne.clone(), List::default),
                    &a_s,
                );
                if let Some(m) = the_mes.seek(&ne) {
                    ne = m.clone();
                    ne.set_orientation(a_s.orientation());
                    the_new_edges.add(ne.clone());
                    {
                        let mut e = topods::edge(&ne)?;
                        if !trim_edge(&mut e, the_as_des2d, the_as_des, the_e_trim_e_inf)? {
                            return Ok(false);
                        }
                    }
                } else {
                    let an_edge = topods::edge(&ne)?;
                    let a_bac = BRepAdaptorCurve::with_edge(&an_edge)?;
                    if a_bac.get_type()? == CurveType::Line {
                        let mut a_new_edge = Edge::default();
                        BRepOffsetInter2d::extent_edge(&an_edge, &mut a_new_edge, the_offset)?;
                        the_e_trim_e_inf
                            .bind(an_edge.as_ref().clone(), a_new_edge.as_ref().clone());
                    }
                }
                the_as_des.add_shared(&nf, &ne);
            }
        }
    }
    Ok(true)
}

/// `TrimEdge(NE, AsDes2d, AsDes, theETrimEInf)`: the edge trimmed at the extreme of its
/// 2d descendant vertices (which become its vertices, also in the AsDes), the untrimmed
/// copy recorded; an untrimmed line removed from the AsDes; false when a vertex
/// cannot be placed on the edge.
pub(super) fn trim_edge(
    ne: &mut Edge,
    as_des2d: &BRepAlgoAsDes,
    as_des: &BRepAlgoAsDes,
    the_e_trim_e_inf: &mut ShapeShapeMap,
) -> Result<bool, Failure> {
    let (mut v1, mut v2) = topexp::vertices(ne, false);
    let (a_t1, a_t2) = BRepTool::range(ne)?;
    let mut a_source_edge = Edge::default();
    make_split_edge(ne, &v1, a_t1, &v2, a_t2, &mut a_source_edge)?;
    let a_same_par_tol = precision::confusion();
    let mut u_min = precision::infinite();
    let mut u_max = -u_min;
    let le: Vec<Shape> = as_des2d.descendant(ne.as_ref()).collect();
    let mut b_trim = false;
    if le.len() > 1 {
        for s in &le {
            let mut v = topods::vertex(s)?;
            if ne.orientation() == Orientation::Reversed {
                v.reverse();
            }
            let u = match find_parameter(&v, ne)? {
                Some(u) => u,
                None => {
                    let (the_curve, _, _) = BRepTool::curve(ne)?;
                    let the_point = BRepTool::pnt(&v)?;
                    let projector = ProjectPointOnCurve::with_curve(the_point, &the_curve)?;
                    if projector.nb_points() == 0 {
                        return Ok(false);
                    }
                    projector.lower_distance_parameter()?
                }
            };
            if u < u_min {
                u_min = u;
                v1 = v.clone();
            }
            if u > u_max {
                u_max = u;
                v2 = v.clone();
            }
        }
        if v1.is_null() || v2.is_null() {
            return Ok(false);
        }
        if !v1.is_same(&v2) {
            ne.tshape().set_free(true);
            let builder = BRepBuilder::default();
            let or = ne.orientation();
            ne.set_orientation(Orientation::Forward);
            let (vf, vl) = topexp::vertices(ne, false);
            let mut ne_shape = ne.as_ref().clone();
            builder.remove(&mut ne_shape, vf.as_ref())?;
            builder.remove(&mut ne_shape, vl.as_ref())?;
            builder.add(&mut ne_shape, &v1.oriented(Orientation::Forward))?;
            builder.add(&mut ne_shape, &v2.oriented(Orientation::Reversed))?;
            *ne = topods::edge(&ne_shape)?;
            builder.range(ne, u_min, u_max, false)?;
            ne.set_orientation(or);
            as_des.add_shared(ne.as_ref(), &v1.oriented(Orientation::Forward));
            as_des.add_shared(ne.as_ref(), &v2.oriented(Orientation::Reversed));
            brep_lib::same_parameter_shape(ne.as_ref(), a_same_par_tol, true)?;
            b_trim = true;
        }
    }
    if !b_trim {
        let a_bac = BRepAdaptorCurve::with_edge(ne)?;
        if a_bac.get_type()? == CurveType::Line && as_des.has_ascendant(ne.as_ref()) {
            as_des.remove_shared(ne.as_ref())?;
        }
    } else if the_e_trim_e_inf.seek(ne.as_ref()).is_none() {
        the_e_trim_e_inf.bind(ne.as_ref().clone(), a_source_edge.as_ref().clone());
    }
    Ok(true)
}

/// `GetEnlargedFaces(theFaces, theMapSF, theMES, theFacesOrigins, theImage, theLSF)`:
/// the enlarged offset faces (roots of the image) with their initial faces.
pub(super) fn get_enlarged_faces(
    the_faces: &[Shape],
    the_map_sf: &OffsetFaces,
    the_mes: &ShapeShapeMap,
    the_faces_origins: &mut ShapeShapeMap,
    the_image: &mut BRepAlgoImage,
    the_lsf: &mut List<Shape>,
) -> Result<(), Failure> {
    for fi in the_faces {
        let ofi = the_map_sf
            .seek(fi)
            .ok_or_else(not_bound)?
            .face()
            .as_ref()
            .clone();
        if let Some(a_local_face) = the_mes.seek(&ofi) {
            the_lsf.append(a_local_face.clone());
            the_image.set_root(a_local_face);
            the_faces_origins.bind(a_local_face.clone(), fi.clone());
        }
    }
    Ok(())
}

/// `GetSubShapes(theShape, theSSType, theResult)`: the compound of the sub-shapes of
/// the type; false without one.
pub(super) fn get_sub_shapes(
    the_shape: &Shape,
    the_ss_type: ShapeEnum,
    the_result: &mut Shape,
) -> Result<bool, Failure> {
    let subs: Vec<Shape> = Explorer::new(the_shape, the_ss_type, ShapeEnum::Shape)?.collect();
    if subs.is_empty() {
        return Ok(false);
    }
    let mut a_result = compound();
    for a_ss in &subs {
        BRepBuilder::default().add(&mut a_result, a_ss)?;
    }
    *the_result = a_result;
    Ok(true)
}

/// A MakerVolume of the faces, without errors; `None` otherwise.
fn maker_volume(faces: &List<Shape>, intersect: bool) -> Option<BOPAlgoMakerVolume> {
    let mut a_mv = BOPAlgoMakerVolume::new();
    a_mv.set_arguments(faces);
    a_mv.set_intersect(intersect);
    a_mv.set_avoid_internal_shapes(true);
    a_mv.perform();
    if a_mv.options().has_errors() {
        return None;
    }
    Some(a_mv)
}

/// `BuildShellsCompleteInter(theLF, theImage, theShells, Range)`: the shells of the
/// solids of the faces (MakerVolume); with several solids, rebuilt from the faces of
/// one solid only, then from the faces whose normals agree with their origins.
pub(super) fn build_shells_complete_inter(
    the_lf: &List<Shape>,
    the_image: &mut BRepAlgoImage,
    the_shells: &mut Shape,
) -> Result<bool, Failure> {
    // The solids, the faces intersected to treat the tangent faces.
    let Some(a_mv1) = maker_volume(the_lf, true) else {
        return Ok(false);
    };
    update_history(the_lf, &a_mv1, the_image)?;
    let a_result1 = a_mv1.shape().clone();
    if a_result1.shape_type()? == ShapeEnum::Solid {
        // A single solid: nothing to do.
        return get_sub_shapes(&a_result1, ShapeEnum::Shell, the_shells);
    }
    // The image supports no multiple ancestors: the origins of the splits, to resolve
    // the non-manifold solids by the normals of the splits and their origins.
    let mut an_origins: EdgesOrigins = DataMap::default();
    for a_fr in the_image.roots().to_vec() {
        for a_f_im in the_image.last_image(&a_fr) {
            an_origins
                .try_emplaced(a_f_im, List::default)
                .append(a_fr.clone());
        }
    }
    // The solids rebuilt without the internal faces: the faces of one solid only.
    let mut a_dmfs = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&a_result1, ShapeEnum::Face, ShapeEnum::Solid, &mut a_dmfs)?;
    if a_dmfs.extent() == 0 {
        // No solid.
        return Ok(false);
    }
    let mut a_lf = List::default();
    for i in 1..=a_dmfs.extent() {
        if a_dmfs.find_from_index(i)?.len() == 1 {
            a_lf.append(a_dmfs.find_key(i)?.clone());
        }
    }
    // The solids of the new list, without intersection.
    let Some(a_mv2) = maker_volume(&a_lf, false) else {
        return Ok(false);
    };
    let a_result2 = a_mv2.shape().clone();
    if a_result2.shape_type()? == ShapeEnum::Solid {
        return get_sub_shapes(&a_result2, ShapeEnum::Shell, the_shells);
    }
    let faces: Vec<Shape> = Explorer::new(&a_result2, ShapeEnum::Face, ShapeEnum::Shape)?.collect();
    if faces.is_empty() {
        return Ok(false);
    }
    // A non-manifold result: the faces whose normals agree with their origins.
    let mut a_lf = List::default();
    for a_fs in &faces {
        let a_f = topods::face(a_fs)?;
        let Some(p_lf_or) = an_origins.seek(a_fs) else {
            // The origins map does not contain the split face (a native assertion).
            continue;
        };
        for a_f_or in p_lf_or.iter() {
            if BRepOffsetTool::check_planes_normals(&a_f, &topods::face(a_f_or)?, 1.0e-8)? {
                a_lf.append(a_fs.clone());
                break;
            }
        }
    }
    // The solid of the outer faces with the right normals.
    let Some(a_mv3) = maker_volume(&a_lf, false) else {
        return Ok(false);
    };
    let a_result3 = a_mv3.shape().clone();
    get_sub_shapes(&a_result3, ShapeEnum::Shell, the_shells)
}

/// `RemoveSeamAndDegeneratedEdges(theFace, theOldFace)`: the wires of a face made planar
/// rebuilt without its degenerated and seam edges.
pub(super) fn remove_seam_and_degenerated_edges(
    the_face: &Face,
    the_old_face: &Face,
) -> Result<(), Failure> {
    let mut a_face = the_face.as_ref().oriented(Orientation::Forward);
    let mut a_is_deg_or_seam_found = false;
    let mut a_eseq: Vec<Shape> = Vec::new();
    for es in Explorer::new(&a_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let an_edge = topods::edge(&es)?;
        if BRepTool::degenerated(&an_edge)? || brep_tools::is_really_closed(&an_edge, the_old_face)?
        {
            a_is_deg_or_seam_found = true;
        } else {
            a_eseq.push(es);
        }
    }
    if !a_is_deg_or_seam_found {
        return Ok(());
    }
    // The wires rebuilt.
    let builder = BRepBuilder::default();
    let mut a_wlist = Vec::new();
    let mut an_it_face = ShapeIterator::new(&a_face, true, true);
    while an_it_face.more() {
        a_wlist.push(an_it_face.value()?.clone());
        an_it_face.next();
    }
    a_face.tshape().set_free(true);
    for w in &a_wlist {
        builder.remove(&mut a_face, w)?;
    }
    while !a_eseq.is_empty() {
        let mut new_wire = Wire::default();
        builder.make_wire(&mut new_wire);
        let mut a_new_wire = new_wire.into_shape();
        let mut a_cur_edge = topods::edge(&a_eseq.remove(0))?;
        builder.add(&mut a_new_wire, a_cur_edge.as_ref())?;
        let (a_first_vertex, mut a_cur_vertex) = topexp::vertices(&a_cur_edge, true);
        while !a_cur_vertex.is_same(&a_first_vertex) {
            let mut found = None;
            for (ind, es) in a_eseq.iter().enumerate() {
                a_cur_edge = topods::edge(es)?;
                let (a_v1, a_v2) = topexp::vertices(&a_cur_edge, true);
                if a_v1.is_same(&a_cur_vertex) {
                    found = Some((ind, a_v2));
                    break;
                }
            }
            let Some((ind, a_v2)) = found else {
                // An error: the wire is not closed.
                break;
            };
            builder.add(&mut a_new_wire, a_cur_edge.as_ref())?;
            a_eseq.remove(ind);
            a_cur_vertex = a_v2;
        }
        builder.add(&mut a_face, &a_new_wire)?;
    }
    Ok(())
}

/// `IsSolid(theS)`: whether the shape has a solid.
pub(super) fn is_solid(the_s: &Shape) -> Result<bool, Failure> {
    Ok(Explorer::new(the_s, ShapeEnum::Solid, ShapeEnum::Shape)?.more())
}

/// Standard_NoSuchObject of a map lookup.
pub(super) fn not_bound() -> Failure {
    Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find")
}
