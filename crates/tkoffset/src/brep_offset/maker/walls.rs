// Rust translation of BRepOffset_MakeOffset::MakeMissingWalls of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The walls of a thickened shell: along each free boundary contour, the face between
//! each boundary edge and its offset (cylindrical, planar, conical or ruled), with the
//! arc faces of the arc joins.
use super::BRepOffsetMakeOffset;
use super::statics::{self, compute_max_dist, not_bound};
use crate::brep_offset::BRepOffsetError;
use crate::brep_offset::tool::live;
use rustycad_tkbo::int_tools::FClass2d;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::brep_tools::update_face;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCurve, GeomPlane, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkgeomalgo::geom_fill::GeomFillGenerator;
use rustycad_tkgeombase::gc::{MakeCylindricalSurface, MakeLine2d};
use rustycad_tkgeombase::gce::{MakeCone, MakeDir};
use rustycad_tkgeombase::geom_lib;
use rustycad_tkmath::geom_abs::{CurveType, JoinType, Shape as Continuity};
use rustycad_tkmath::gp::{self, Axis3, Direction2, Line3, Point2, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tktopalgo::brep_lib::{self, BRepLibFindSurface, BRepLibMakeEdge, BRepLibMakeFace};
use std::f64::consts::FRAC_PI_4;

/// A 2d line through a point along a direction.
fn line2d(x: f64, y: f64, d: Direction2) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::from_point(Point2::new(x, y), d)).cast()
}

/// `BRepLib_MakeEdge(V1, V2)`.
fn edge_between(v1: &Vertex, v2: &Vertex) -> Result<Edge, Failure> {
    BRepLibMakeEdge::from_vertices(v1, v2)?.edge()
}

/// The 3d curve of the line between two 2d points on the surface (GeomLib::BuildCurve3d
/// with the native defaults), put on the edge with the line.
fn connect_on_surface(
    an_e: &Edge,
    p2d: Point2,
    p2d_oe: Point2,
    the_surf: &Handle<dyn GeomSurface>,
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let builder = BRepBuilder::default();
    let a_line: Handle<dyn Geom2dCurve> = MakeLine2d::from_points(p2d, p2d_oe)?.value()?.cast();
    let (first_par, last_par) = (0., p2d.distance(p2d_oe));
    let hc2d: Handle<dyn Adaptor2dCurve2d> = Handle::new(Geom2dAdaptorCurve::with_range(
        &a_line, first_par, last_par,
    )?)
    .cast();
    let hsurf: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(the_surf)?).cast();
    let con_s = Adaptor3dCurveOnSurface::with_curve(&hc2d, &hsurf)?;
    let mut a_curve: Handle<dyn GeomCurve> = Handle::default();
    let (max_deviation, _) = geom_lib::build_curve3d(
        precision::confusion(),
        &con_s,
        (first_par, last_par),
        &mut a_curve,
        Continuity::C1,
        14,
        30,
    )?;
    builder.update_edge_curve(an_e, &a_curve, max_deviation)?;
    builder.update_edge_pcurve(
        an_e,
        &a_line,
        the_surf,
        &Location::default(),
        max_deviation,
        None,
    )?;
    builder.range(an_e, first_par, last_par, false)?;
    Ok(a_line)
}

impl BRepOffsetMakeOffset {
    /// `MakeMissingWalls(Range)`: the walls along the free boundary contours of the
    /// shape, appended to `myWalls`.
    pub(super) fn make_missing_walls(&mut self, the_range: &ProgressRange) -> Result<(), Failure> {
        // The start vertex and connected edges of each free boundary contour, and the
        // face of each contour edge.
        let mut contours = IndexedDataMapOfShapeListOfShape::default();
        let mut map_ef = crate::brep_offset::ShapeShapeMap::default();
        let offset_val = self.my_offset.abs();
        statics::fill_contours(
            &self.my_face_comp,
            &self.my_analyse,
            &mut contours,
            &mut map_ef,
        )?;
        let a_ps = ProgressScope::with_literal(
            the_range,
            "Making missing walls",
            contours.extent() as f64,
            false,
        );
        for ic in 1..=contours.extent() {
            if !a_ps.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(());
            }
            let _ = a_ps.next(1.);
            let start_vertex = topods::vertex(contours.find_key(ic)?)?;
            let a_contour: Vec<Shape> = contours.find_from_index(ic)?.iter().cloned().collect();
            let mut wall = WallState {
                start_vertex: start_vertex.clone(),
                start_edge: Edge::default(),
                first_step: true,
                prev_edge: Edge::default(),
                prev_vertex: start_vertex,
                is_build_from_scratch: false,
            };
            for es in &a_contour {
                self.make_wall(&topods::edge(es)?, &map_ef, offset_val, &mut wall)?;
            }
        }
        Ok(())
    }

    /// The wall of one contour edge.
    fn make_wall(
        &mut self,
        an_edge: &Edge,
        map_ef: &crate::brep_offset::ShapeShapeMap,
        offset_val: f64,
        w: &mut WallState,
    ) -> Result<(), Failure> {
        let builder = BRepBuilder::default();
        let a_face_of_edge = topods::face(map_ef.seek(an_edge.as_ref()).ok_or_else(not_bound)?)?;
        // The offset must exist.
        if !self.my_init_offset_edge.has_image(an_edge.as_ref()) {
            return Ok(());
        }
        // Two different vertices must exist.
        let loe = self.my_init_offset_edge.last_image(an_edge.as_ref());
        let loe2 = self
            .my_image_offset
            .last_image(loe.last().ok_or_else(not_bound)?);
        let mut oe = topods::edge(loe2.last().ok_or_else(not_bound)?)?;
        let (mut v4, mut v3) = topexp::vertices(&oe, false);
        let (mut v1, mut v2) = topexp::vertices(an_edge, false);
        let (a_c, a_f, a_l) = BRepTool::curve(an_edge)?;
        if v3.is_null() && v4.is_null() {
            // The offset edge is created without vertices and gets them when trimmed by
            // the intersection lines of the extended offset faces; when those are invalid
            // (mixed connectivity, for one) the edge keeps none. This workaround only
            // avoids the exception: the vertex points are invalid.
            let an_oe_ori = oe.orientation();
            oe.set_orientation(Orientation::Forward);
            let (an_oe_c, _, _) = BRepTool::curve(&oe)?;
            let c = live(&an_oe_c, "curve")?;
            let (a_p1, a_p2) = (c.value(a_f)?, c.value(a_l)?);
            let a_tol = BRepTool::vertex_tolerance(&v1)?.max(BRepTool::vertex_tolerance(&v2)?);
            let mut an_oe_v1 = Vertex::default();
            builder.make_vertex_at(&mut an_oe_v1, a_p1, a_tol)?;
            an_oe_v1.set_orientation(Orientation::Forward);
            let mut an_oe_v2 = Vertex::default();
            builder.make_vertex_at(&mut an_oe_v2, a_p2, a_tol)?;
            an_oe_v2.set_orientation(Orientation::Reversed);
            let mut oe_shape = oe.as_ref().clone();
            builder.add(&mut oe_shape, an_oe_v1.as_ref())?;
            builder.add(&mut oe_shape, an_oe_v2.as_ref())?;
            oe = topods::edge(&oe_shape)?;
            builder.range(&oe, a_f, a_l, false)?;
            oe.set_orientation(an_oe_ori);
            (v4, v3) = topexp::vertices(&oe, false);
        }
        if let Some(c) = a_c.get()
            && !c.is_closed()?
            && !c.is_periodic()
        {
            let a_pnt_f = BRepTool::pnt(&v1)?;
            let a_pnt_l = BRepTool::pnt(&v2)?;
            let a_dist_e = a_pnt_f.square_distance(&a_pnt_l);
            if a_dist_e < precision::square_confusion() {
                // A bad case: not closed, but the vertices map to the same point.
                return Ok(());
            }
            let an_edge_tol = BRepTool::edge_tolerance(an_edge)?;
            if a_dist_e < an_edge_tol {
                // Potential problems not detected by the shape check.
                let a_pnt_of = BRepTool::pnt(&v4)?;
                let a_pnt_ol = BRepTool::pnt(&v3)?;
                if a_pnt_of.square_distance(&a_pnt_ol) > gp::resolution() {
                    // No complex analytical continuation of sin / arcsin.
                    let a_sin_value = (2. * an_edge_tol / a_pnt_of.distance(&a_pnt_ol)).min(1.0);
                    let a_max_angle = a_sin_value.asin().abs().min(FRAC_PI_4);
                    let a_current_angle = Vector3::between(a_pnt_f, a_pnt_l)
                        .angle(Vector3::between(a_pnt_of, a_pnt_ol))?;
                    if a_c.is_kind_name("Geom_Line") && a_current_angle.abs() > a_max_angle {
                        // The edge is not collinear to its offset.
                        w.is_build_from_scratch = true;
                        self.my_is_perform_sewing = true;
                        return Ok(());
                    }
                }
            }
        }
        let mut to_reverse = false;
        if !v1.is_same(&w.prev_vertex) {
            std::mem::swap(&mut v1, &mut v2);
            std::mem::swap(&mut v3, &mut v4);
            to_reverse = true;
        }
        oe.set_orientation(top_abs::reverse(an_edge.orientation()));
        let arc_on_v2 =
            self.my_join == JoinType::Arc && self.my_init_offset_edge.has_image(v2.as_ref());
        let mut e4 = if w.first_step || w.is_build_from_scratch {
            let e4 = edge_between(&v1, &v4)?;
            if w.first_step {
                w.start_edge = e4.clone();
            }
            e4
        } else {
            w.prev_edge.clone()
        };
        let mut e3 = if v2.is_same(&w.start_vertex) && !arc_on_v2 {
            w.start_edge.clone()
        } else {
            edge_between(&v2, &v3)?
        };
        e4.reverse();
        if w.is_build_from_scratch {
            e3.reverse();
            e4.reverse();
        }
        let an_edge_fwd = topods::edge(&an_edge.oriented(Orientation::Forward))?;
        let par_v1 = BRepTool::parameter(&v1, &an_edge_fwd)?;
        let par_v2 = BRepTool::parameter(&v2, &an_edge_fwd)?;
        let mut wire_w = Wire::default();
        builder.make_wire(&mut wire_w);
        let mut the_wire = wire_w.into_shape();
        if to_reverse {
            for s in [
                an_edge.reversed(),
                e3.reversed(),
                oe.reversed(),
                e4.reversed(),
            ] {
                builder.add(&mut the_wire, &s)?;
            }
        } else {
            for s in [
                an_edge.as_ref().clone(),
                e3.as_ref().clone(),
                oe.as_ref().clone(),
                e4.as_ref().clone(),
            ] {
                builder.add(&mut the_wire, &s)?;
            }
        }
        brep_lib::build_curves3d(&the_wire, self.my_tol, Continuity::C1, 14, 0)?;
        the_wire.tshape().set_closed(true);
        let mut new_face = Face::default();
        let mut the_surf: Handle<dyn GeomSurface> = Handle::default();
        let ba_curve = BRepAdaptorCurve::with_edge(an_edge)?;
        let ba_curve_oe = BRepAdaptorCurve::with_edge(&oe)?;
        let fpar = ba_curve.first_parameter()?;
        let lpar = ba_curve.last_parameter()?;
        let pon_e = ba_curve.value(fpar)?;
        let pon_oe = ba_curve_oe.value(fpar)?;
        let offset_dir = MakeDir::from_points(pon_e, pon_oe)?.value()?;
        let mut edge_line2d: Handle<dyn Geom2dCurve> = Handle::default();
        let mut oe_line2d: Handle<dyn Geom2dCurve> = Handle::default();
        let mut is_planar = false;
        let loc = Location::default();
        if ba_curve.get_type()? == CurveType::Circle && ba_curve_oe.get_type()? == CurveType::Circle
        {
            let a_circ = ba_curve.circle()?;
            let a_circ_oe = ba_curve_oe.circle()?;
            let an_axis_line = Line3::from_axis(a_circ.axis());
            let mut circ_axis_dir = a_circ.axis().direction();
            if a_circ
                .axis()
                .is_parallel(a_circ_oe.axis(), precision::confusion())
                && an_axis_line.contains(a_circ_oe.location(), precision::confusion())
            {
                // A cylinder, a plane or a cone.
                if (a_circ.radius() - a_circ_oe.radius()).abs() <= precision::confusion() {
                    // A cylinder.
                    the_surf = MakeCylindricalSurface::from_circ(a_circ)?.value()?.cast();
                } else if a_circ.location().distance(&a_circ_oe.location())
                    <= precision::confusion()
                {
                    // A plane.
                    is_planar = true;
                    let pon_el = ba_curve.value(lpar)?;
                    if pon_el.distance(&pon_e) <= precision::p_confusion() {
                        new_face = planar_annulus(
                            an_edge,
                            &oe,
                            a_circ.radius() > a_circ_oe.radius(),
                            a_circ.location(),
                            circ_axis_dir,
                        )?;
                    }
                } else {
                    // A cone.
                    let mut the_cone = MakeCone::from_points_radii(
                        a_circ.location(),
                        a_circ_oe.location(),
                        a_circ.radius(),
                        a_circ_oe.radius(),
                    )?
                    .value()?;
                    let mut the_ax3 = Axis3::from_axis2(a_circ.position());
                    if circ_axis_dir.dot(the_cone.axis().direction()) < 0. {
                        the_ax3.z_reverse();
                        circ_axis_dir.reverse();
                    }
                    the_cone.set_position(the_ax3);
                    the_surf = Handle::new(GeomConicalSurface::from_cone(the_cone)).cast();
                }
                if !is_planar {
                    edge_line2d = line2d(0., 0., Direction2::X);
                    builder.update_edge_pcurve(
                        an_edge,
                        &edge_line2d,
                        &the_surf,
                        &loc,
                        precision::confusion(),
                        None,
                    )?;
                    let coeff = if offset_dir.dot(circ_axis_dir) > 0. {
                        1.
                    } else {
                        -1.
                    };
                    oe_line2d = line2d(0., offset_val * coeff, Direction2::X);
                    builder.update_edge_pcurve(
                        &oe,
                        &oe_line2d,
                        &the_surf,
                        &loc,
                        precision::confusion(),
                        None,
                    )?;
                    let dir = Direction2::new(0., coeff)?;
                    let a_line2d = line2d(par_v2, 0., dir);
                    let a_line2d2 = line2d(par_v1, 0., dir);
                    if e3.is_same(&e4) {
                        if coeff > 0. {
                            builder.update_edge_pcurves(
                                &e3,
                                (&a_line2d, &a_line2d2),
                                &the_surf,
                                &loc,
                                precision::confusion(),
                                None,
                            )?;
                        } else {
                            builder.update_edge_pcurves(
                                &e3,
                                (&a_line2d2, &a_line2d),
                                &the_surf,
                                &loc,
                                precision::confusion(),
                                None,
                            )?;
                            let mut ww = Wire::default();
                            builder.make_wire(&mut ww);
                            the_wire = ww.into_shape();
                            for s in [
                                an_edge.oriented(Orientation::Reversed),
                                e4.as_ref().clone(),
                                oe.oriented(Orientation::Forward),
                                e3.as_ref().clone(),
                            ] {
                                builder.add(&mut the_wire, &s)?;
                            }
                            the_wire.tshape().set_closed(true);
                        }
                    } else {
                        for e in [&e3, &e4] {
                            builder.same_parameter(e, false)?;
                            builder.same_range(e, false)?;
                        }
                        builder.update_edge_pcurve(
                            &e3,
                            &a_line2d,
                            &the_surf,
                            &loc,
                            precision::confusion(),
                            None,
                        )?;
                        builder.range_on_surface(&e3, &the_surf, &loc, 0., offset_val)?;
                        builder.update_edge_pcurve(
                            &e4,
                            &a_line2d2,
                            &the_surf,
                            &loc,
                            precision::confusion(),
                            None,
                        )?;
                        builder.range_on_surface(&e4, &the_surf, &loc, 0., offset_val)?;
                    }
                    new_face = BRepLibMakeFace::from_surface_wire(
                        &the_surf,
                        &topods::wire(&the_wire)?,
                        true,
                    )?
                    .face()?;
                }
            }
        }
        if new_face.is_null() {
            let an_edge_tol = BRepTool::edge_tolerance(an_edge)?;
            // The tolerances of the input shape are not raised by BRepLib_MakeFace.
            let a_find_plane = BRepLibFindSurface::new(&the_wire, an_edge_tol, true, false)?;
            is_planar = false;
            if a_find_plane.found() && a_find_plane.tolerance_reached() <= an_edge_tol {
                let (a_gc, f, l) = BRepTool::curve(an_edge)?;
                let a_pln_h = a_find_plane.surface().cast::<GeomPlane>();
                let a_pln = live(&a_pln_h, "plane")?.pln();
                let a_max_dist = compute_max_dist(&a_pln, &a_gc, f, l)?;
                if a_max_dist <= an_edge_tol
                    && let Ok(mf) =
                        BRepLibMakeFace::from_plane_wire(a_pln, &topods::wire(&the_wire)?, true)
                    && mf.is_done()
                {
                    new_face = mf.face()?;
                    let mut an_it_e = ShapeIterator::new(&the_wire, true, true);
                    while an_it_e.more() {
                        let an_e = topods::edge(an_it_e.value()?)?;
                        an_it_e.next();
                        if an_e.is_same(an_edge) {
                            continue;
                        }
                        let (a_gc, f, l) = BRepTool::curve(&an_e)?;
                        let a_max_dist = compute_max_dist(&a_pln, &a_gc, f, l)?;
                        builder.update_edge_tolerance(&an_e, a_max_dist)?;
                    }
                    is_planar = true;
                }
            }
            if !is_planar {
                // An extrusion (by thru sections).
                let (edge_curve, fp, lp) = BRepTool::curve(an_edge)?;
                let tr_edge_curve: Handle<dyn GeomCurve> =
                    Handle::new(GeomTrimmedCurve::new(&edge_curve, fp, lp, true, true)?).cast();
                let (offset_curve, fpar_oe, lpar_oe) = BRepTool::curve(&oe)?;
                let tr_offset_curve: Handle<dyn GeomCurve> = Handle::new(GeomTrimmedCurve::new(
                    &offset_curve,
                    fpar_oe,
                    lpar_oe,
                    true,
                    true,
                )?)
                .cast();
                let mut thrusec_generator = GeomFillGenerator::new();
                thrusec_generator.add_curve(&tr_edge_curve)?;
                thrusec_generator.add_curve(&tr_offset_curve)?;
                thrusec_generator.perform(precision::p_confusion())?;
                the_surf = thrusec_generator.surface();
                let (uf, ul, vf, vl) = live(&the_surf, "surface")?.bounds();
                edge_line2d = line2d(0., vf, Direction2::X);
                builder.update_edge_pcurve(
                    an_edge,
                    &edge_line2d,
                    &the_surf,
                    &loc,
                    precision::confusion(),
                    None,
                )?;
                oe_line2d = line2d(0., vl, Direction2::X);
                builder.update_edge_pcurve(
                    &oe,
                    &oe_line2d,
                    &the_surf,
                    &loc,
                    precision::confusion(),
                    None,
                )?;
                let u_on_v1 = if to_reverse { ul } else { uf };
                let u_on_v2 = if to_reverse { uf } else { ul };
                let a_line2d = line2d(u_on_v2, 0., Direction2::Y);
                let a_line2d2 = line2d(u_on_v1, 0., Direction2::Y);
                let s = live(&the_surf, "surface")?;
                if e3.is_same(&e4) {
                    builder.update_edge_pcurves(
                        &e3,
                        (&a_line2d, &a_line2d2),
                        &the_surf,
                        &loc,
                        precision::confusion(),
                        None,
                    )?;
                    let bspl_c34 = s.u_iso(uf)?;
                    builder.update_edge_curve(&e3, &bspl_c34, precision::confusion())?;
                    builder.range(&e3, vf, vl, false)?;
                } else {
                    for e in [&e3, &e4] {
                        builder.same_parameter(e, false)?;
                        builder.same_range(e, false)?;
                    }
                    builder.update_edge_pcurve(
                        &e3,
                        &a_line2d,
                        &the_surf,
                        &loc,
                        precision::confusion(),
                        None,
                    )?;
                    builder.range_on_surface(&e3, &the_surf, &loc, vf, vl)?;
                    builder.update_edge_pcurve(
                        &e4,
                        &a_line2d2,
                        &the_surf,
                        &loc,
                        precision::confusion(),
                        None,
                    )?;
                    builder.range_on_surface(&e4, &the_surf, &loc, vf, vl)?;
                    let bspl_c3 = s.u_iso(u_on_v2)?;
                    builder.update_edge_curve(&e3, &bspl_c3, precision::confusion())?;
                    // Only for the 3d curve.
                    builder.range(&e3, vf, vl, true)?;
                    let bspl_c4 = s.u_iso(u_on_v1)?;
                    builder.update_edge_curve(&e4, &bspl_c4, precision::confusion())?;
                    builder.range(&e4, vf, vl, true)?;
                }
                new_face =
                    BRepLibMakeFace::from_surface_wire(&the_surf, &topods::wire(&the_wire)?, true)?
                        .face()?;
            }
        }
        if !is_planar {
            let fpar_oe = ba_curve_oe.first_parameter()?;
            let lpar_oe = ba_curve_oe.last_parameter()?;
            if (fpar - fpar_oe).abs() > precision::confusion() {
                let an_e4 = if to_reverse { &e3 } else { &e4 };
                let fp2d = live(&edge_line2d, "line")?.value(fpar)?;
                let fp2d_oe = live(&oe_line2d, "line")?.value(fpar_oe)?;
                connect_on_surface(an_e4, fp2d, fp2d_oe, &the_surf)?;
            }
            if (lpar - lpar_oe).abs() > precision::confusion() {
                let an_e3 = if to_reverse { &e4 } else { &e3 };
                let lp2d = live(&edge_line2d, "line")?.value(lpar)?;
                let lp2d_oe = live(&oe_line2d, "line")?.value(lpar_oe)?;
                connect_on_surface(an_e3, lp2d, lp2d_oe, &the_surf)?;
            }
        }
        if !is_planar {
            // For planar faces these operations are useless: no curves on surface.
            brep_lib::same_parameter_shape(new_face.as_ref(), 1.0e-5, false)?;
            update_face(&new_face)?;
        }
        // The orientation.
        let an_or =
            statics::orientation_of_edge_in_face(an_edge.as_ref(), a_face_of_edge.as_ref())?;
        let or_in_new_face =
            statics::orientation_of_edge_in_face(an_edge.as_ref(), new_face.as_ref())?;
        if or_in_new_face != top_abs::reverse(an_or) {
            new_face.reverse();
        }
        self.my_walls.append(new_face.as_ref().clone());
        if arc_on_v2 {
            let mut an_arc = topods::edge(
                self.my_init_offset_edge
                    .image(v2.as_ref())
                    .first()
                    .ok_or_else(not_bound)?,
            )?;
            let (mut arc_v1, mut arc_v2) = topexp::vertices(&an_arc, false);
            let mut arc_reverse = false;
            if !arc_v1.is_same(&v3) {
                std::mem::swap(&mut arc_v1, &mut arc_v2);
                arc_reverse = true;
            }
            let mut ea1 = e3.clone();
            ea1.reverse();
            if to_reverse {
                ea1.reverse();
            }
            let mut ea2 = if v2.is_same(&w.start_vertex) {
                w.start_edge.clone()
            } else {
                edge_between(&v2, &arc_v2)?
            };
            an_arc.set_orientation(if arc_reverse {
                Orientation::Reversed
            } else {
                Orientation::Forward
            });
            if ea1.orientation() == Orientation::Reversed {
                an_arc.reverse();
            }
            ea2.set_orientation(top_abs::reverse(ea1.orientation()));
            let mut aw = Wire::default();
            builder.make_wire(&mut aw);
            let mut arc_wire = aw.into_shape();
            builder.add(&mut arc_wire, ea1.as_ref())?;
            builder.add(&mut arc_wire, an_arc.as_ref())?;
            builder.add(&mut arc_wire, ea2.as_ref())?;
            brep_lib::build_curves3d(&arc_wire, self.my_tol, Continuity::C1, 14, 0)?;
            arc_wire.tshape().set_closed(true);
            let arc_face = BRepLibMakeFace::from_wire(&topods::wire(&arc_wire)?, true)?.face()?;
            update_face(&arc_face)?;
            self.my_walls.append(arc_face.as_ref().clone());
            w.prev_edge = topods::edge(&ea2.oriented(Orientation::Forward))?;
            w.prev_vertex = v2;
        } else if w.is_build_from_scratch {
            w.prev_edge = e4;
            w.prev_vertex = v1;
            w.is_build_from_scratch = false;
        } else {
            w.prev_edge = e3;
            w.prev_vertex = v2;
        }
        w.first_step = false;
        Ok(())
    }
}

/// The state of the walls along a contour.
struct WallState {
    start_vertex: Vertex,
    start_edge: Edge,
    first_step: bool,
    prev_edge: Edge,
    prev_vertex: Vertex,
    is_build_from_scratch: bool,
}

/// The planar annulus between two concentric circles: the outer wire kept when not a
/// hole, the inner one reversed when it is not one.
fn planar_annulus(
    an_edge: &Edge,
    oe: &Edge,
    edge_outer: bool,
    center: rustycad_tkmath::gp::Point3,
    axis_dir: rustycad_tkmath::gp::Direction3,
) -> Result<Face, Failure> {
    let builder = BRepBuilder::default();
    let (a_e1, a_e2) = if edge_outer {
        (an_edge.clone(), oe.clone())
    } else {
        (oe.clone(), an_edge.clone())
    };
    let a_pl: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_point(center, axis_dir)?).cast();
    let mut wires = Vec::new();
    for (i, a_e) in [&a_e1, &a_e2].into_iter().enumerate() {
        let mut aw = Wire::default();
        builder.make_wire(&mut aw);
        let mut a_w = aw.into_shape();
        builder.add(&mut a_w, a_e.as_ref())?;
        let mut a_face = Face::default();
        builder.make_face_on_surface(&mut a_face, &a_pl, precision::confusion())?;
        let mut face_shape = a_face.as_ref().clone();
        builder.add(&mut face_shape, &a_w)?;
        let a_clsf = FClass2d::new(&a_face, precision::confusion())?;
        let b_is_hole = a_clsf.is_hole();
        if (b_is_hole && i == 0) || (!b_is_hole && i == 1) {
            let mut aw2 = Wire::default();
            builder.make_wire(&mut aw2);
            a_w = aw2.into_shape();
            builder.add(&mut a_w, &a_e.reversed())?;
        }
        wires.push(a_w);
    }
    let mut new_face = Face::default();
    builder.make_face_on_surface(&mut new_face, &a_pl, precision::confusion())?;
    let mut nf = new_face.as_ref().clone();
    for w in &wires {
        builder.add(&mut nf, w)?;
    }
    Ok(new_face)
}
