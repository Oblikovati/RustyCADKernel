// Rust translation of Draft_Modification::Perform of
// ModelingAlgorithms/TKOffset/Draft/Draft_Modification_1.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The new surfaces of the postponed faces, the new edge curves and the new vertices.
use super::add::{located_curve, trimmed};
use super::params::{choose, parameter, smart_or_plain};
use super::{DraftState, live, untrimmed_surface};
use crate::draft::info::DraftErrorStatus;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBoundedCurve, GeomCircle, GeomCurve, GeomCylindricalSurface, GeomLine, GeomPlane,
    GeomSurface, GeomSurfaceOfLinearExtrusion, GeomSweptSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnSurf;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkgeomalgo::geom_int::{GeomIntIntSS, GeomIntIntSSApprox};
use rustycad_tkgeomalgo::int_curve_surface::HInter;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtCS, ExtPC};
use rustycad_tkgeombase::geom_convert::CompCurveToBSplineCurve;
use rustycad_tkgeombase::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::elclib;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3, Vector2, Vector3};
use std::f64::consts::PI;

/// The surface as a plane, when it is one.
fn as_plane(s: &Handle<dyn GeomSurface>) -> Handle<GeomPlane> {
    s.clone().cast::<GeomPlane>()
}

impl DraftState {
    /// Sets the error of a failed step.
    fn fail(&mut self, status: DraftErrorStatus, shape: rustycad_tkbrep::topods::Shape) {
        self.err_stat = status;
        self.bad_shape = shape;
    }

    /// `Perform()`.
    pub(super) fn perform(&mut self) -> Result<(), Failure> {
        if !self.bad_shape.is_null() {
            return Err(Failure::with_message("Standard_ConstructionError", "Draft_Modification::Perform"));
        }

        if !self.my_comp {
            self.my_comp = true;
            if !self.propagate()? {
                return Ok(());
            }

            // Calculate eventual faces
            if !self.compute_postponed_faces()? {
                return Ok(());
            }

            // Calculate new edges.
            if !self.compute_edges()? {
                return Ok(());
            }

            // Calculate new vertices.
            if !self.compute_vertices()? {
                return Ok(());
            }
        }

        // small loop of validation/protection
        self.validate_edges()
    }

    /// The extrusions of the circles of the postponed faces between their two faces.
    fn compute_postponed_faces(&mut self) -> Result<bool, Failure> {
        for i in 1..=self.my_f_map.extent() {
            let fk = topods::face(self.my_f_map.find_key(i)?)?;
            let finf = self.my_f_map.find_from_index(i)?;
            if finf.new_geometry() && finf.geometry().is_null() {
                let f1 = finf.first_face().clone();
                let f2 = finf.second_face().clone();
                let ccir_curve = finf.curve().clone();

                if f1.is_null() || f2.is_null() {
                    self.fail(DraftErrorStatus::FaceRecomputation, fk.into_shape());
                    return Ok(false);
                }
                let mut s1 = self.my_f_map.find_from_key(&f1)?.geometry().clone();
                let mut s2 = self.my_f_map.find_from_key(&f2)?.geometry().clone();
                if s1.is_null() || s2.is_null() {
                    self.fail(DraftErrorStatus::FaceRecomputation, fk.into_shape());
                    return Ok(false);
                }
                s1 = untrimmed_surface(&s1)?;
                s2 = untrimmed_surface(&s2)?;
                let (p1, p2) = (as_plane(&s1), as_plane(&s2));
                let (Some(p1), Some(p2)) = (p1.get(), p2.get()) else {
                    self.fail(DraftErrorStatus::FaceRecomputation, fk.into_shape());
                    return Ok(false);
                };
                let pp1 = p1.pln();
                let pp2 = p2.pln();
                let i2p = QuadQuadGeo::planes(pp1, pp2, precision::angular(), precision::confusion())?;
                if !i2p.is_done() || i2p.type_inter()? != ResultType::Line {
                    self.fail(DraftErrorStatus::FaceRecomputation, fk.into_shape());
                    return Ok(false);
                }

                let mut extrdir = i2p.line(1)?.direction();

                // Preserve the same direction as the base face
                let mut ref_surf = BRepTool::surface(&fk)?;
                ref_surf = untrimmed_surface(&ref_surf)?;
                // The native reference direction stays the default one on other surfaces.
                let mut dir_ref = Direction3::default();
                let rs = live(&ref_surf, "surface")?;
                if rs.is_instance_name("Geom_CylindricalSurface") {
                    let axe_ref = live(&ref_surf.clone().cast::<GeomCylindricalSurface>(), "cylinder")?
                        .cylinder()?
                        .position();
                    dir_ref = axe_ref.direction();
                } else if rs.is_instance_name("Geom_SurfaceOfLinearExtrusion") {
                    dir_ref = live(&ref_surf.clone().cast::<GeomSurfaceOfLinearExtrusion>(), "extrusion")?.direction();
                }

                if extrdir.dot(dir_ref) < 0. {
                    extrdir.reverse();
                }

                // it is possible to accelerate speed by storing the info during
                // InternalAdd --> modification of FaceInfo to preserve the circle
                let ccir: Handle<dyn GeomCurve> = ccir_curve.cast::<GeomCircle>().cast();
                let mut new_s: Handle<dyn GeomSurface> =
                    Handle::new(GeomSurfaceOfLinearExtrusion::new(&ccir, extrdir)?).cast();

                let (_umin, _umax, mut vmin, mut vmax) = brep_tools::uv_bounds(&fk)?;
                if !precision::is_negative_infinite(vmin) && !precision::is_positive_infinite(vmax) {
                    let deltav = 2. * (vmax - vmin);
                    vmin -= deltav;
                    vmax += deltav;
                } else {
                    // very temporary
                    vmax = 300.;
                    vmin = -300.;
                }

                new_s = trimmed(&new_s, 1.9 * PI, vmin, vmax)?;
                *self.my_f_map.change_from_index(i)?.change_geometry() = new_s;
            }
        }
        Ok(true)
    }

    /// The new curves of the edges (intersections of their new faces, or the fixed lines
    /// of the tangency edges) and the original curves of the unmodified ones.
    fn compute_edges(&mut self) -> Result<bool, Failure> {
        for ii in 1..=self.my_e_map.extent() {
            let the_edge = topods::edge(self.my_e_map.find_key(ii)?)?;
            let mut c = located_curve(&the_edge)?;

            let (new_geometry, geometry_null) = {
                let einf = self.my_e_map.find_from_index(ii)?;
                (einf.new_geometry(), einf.geometry().is_null())
            };
            if new_geometry && geometry_null {
                let (tangent, ptfixe) = self.my_e_map.find_from_index(ii)?.is_tangent();
                let new_c = if !tangent {
                    match self.intersection_curve(ii, &the_edge, &c)? {
                        Some(new_c) => new_c,
                        None => return Ok(false),
                    }
                } else {
                    // case of tangency
                    match self.tangency_line(ii, &the_edge, &c, ptfixe)? {
                        Some(new_c) => new_c,
                        None => return Ok(false),
                    }
                };
                *self.my_e_map.change_from_index(ii)?.change_geometry() = super::down_cast_basis(&new_c);
            } else if !new_geometry {
                // set existing curve 3D
                c = super::down_cast_basis(&c);
                *self.my_e_map.change_from_index(ii)?.change_geometry() = c;
            }
        }
        Ok(true)
    }

    /// The new curve of a modified edge between its two new faces: the translated
    /// circle of an extrusion cut by a plane, else the intersection of the surfaces
    /// (the branch nearest to the first vertex, or the glued B-spline branches),
    /// oriented as the edge; `None` on failure (the error is set).
    fn intersection_curve(
        &mut self,
        ii: usize,
        the_edge: &Edge,
        c: &Handle<dyn GeomCurve>,
    ) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
        let (first_face, second_face) = {
            let einf = self.my_e_map.find_from_index(ii)?;
            (einf.first_face().clone(), einf.second_face().clone())
        };

        let mut s1 = self.my_f_map.find_from_key(&first_face)?.geometry().clone();
        let mut s2 = self.my_f_map.find_from_key(&second_face)?.geometry().clone();

        let mut detrompeur = 0;

        // Return FirstVertex and the tangent at this point.
        let fv = topexp::first_vertex(the_edge, false);
        let lv = topexp::last_vertex(the_edge, false);
        let mut pmin = 0.;
        let prmfv = BRepTool::parameter(&fv, the_edge)?;
        let prmlv = BRepTool::parameter(&lv, the_edge)?;
        let curve = live(c, "curve")?;
        let (mut pfv, d1fv) = curve.d1(prmfv)?;
        let (plv, _d1lv) = curve.d1(prmlv)?;

        let tol_f1 = BRepTool::face_tolerance(&first_face)?;
        let tol_f2 = BRepTool::face_tolerance(&second_face)?;

        // Pass the tolerance of the face to project
        let proj1 = ProjectPointOnSurf::with_tolerance(pfv, &s1, tol_f1, ExtAlgo::Grad)?;
        let proj2 = ProjectPointOnSurf::with_tolerance(plv, &s1, tol_f1, ExtAlgo::Grad)?;
        let proj3 = ProjectPointOnSurf::with_tolerance(pfv, &s2, tol_f2, ExtAlgo::Grad)?;
        let proj4 = ProjectPointOnSurf::with_tolerance(plv, &s2, tol_f2, ExtAlgo::Grad)?;

        if proj1.is_done()
            && proj2.is_done()
            && proj1.lower_distance()? <= precision::confusion()
            && proj2.lower_distance()? <= precision::confusion()
        {
            detrompeur = 1;
        }

        if proj3.is_done()
            && proj4.is_done()
            && proj3.lower_distance()? <= precision::confusion()
            && proj4.lower_distance()? <= precision::confusion()
        {
            detrompeur = 2;
        }

        let mut the_dir_extr = Direction3::default();
        let mut axis = rustycad_tkmath::gp::Axis3::default();
        let mut the_new_curve: Handle<dyn GeomCurve> = Handle::default();
        let mut k_part = false;

        s1 = untrimmed_surface(&s1)?;
        s2 = untrimmed_surface(&s2)?;

        let mut pc1 = true; // KPart on S1
        let (ls1, ls2) = (live(&s1, "surface")?, live(&s2, "surface")?);
        if ls1.is_instance_name("Geom_SurfaceOfLinearExtrusion") && ls2.is_instance_name("Geom_Plane") {
            k_part = true;
            axis = live(&as_plane(&s2), "plane")?.pln().position();
            let extrusion = s1.clone().cast::<GeomSurfaceOfLinearExtrusion>();
            let extrusion = live(&extrusion, "extrusion")?;
            the_new_curve = extrusion.basis_curve();
            the_dir_extr = extrusion.direction();
        } else if ls2.is_instance_name("Geom_SurfaceOfLinearExtrusion") && ls1.is_instance_name("Geom_Plane") {
            k_part = true;
            pc1 = false;
            axis = live(&as_plane(&s1), "plane")?.pln().position();
            let extrusion = s2.clone().cast::<GeomSurfaceOfLinearExtrusion>();
            let extrusion = live(&extrusion, "extrusion")?;
            the_new_curve = extrusion.basis_curve();
            the_dir_extr = extrusion.direction();
        }
        let a_circ = the_new_curve.clone().cast::<GeomCircle>();
        if k_part {
            // very temporary on circles !!!
            match a_circ.get() {
                None => k_part = false,
                Some(circ) => {
                    let axof_circ = rustycad_tkg3d::geom::GeomConic::position(circ).direction();
                    k_part = axof_circ.is_parallel(axis.direction(), precision::angular());
                }
            }
        }

        let new_c: Handle<dyn GeomCurve>;
        if k_part {
            // direct calculation of NewC
            let circ = live(&a_circ, "circle")?.circ()?;
            let a_local_real = Vector3::between(circ.location(), axis.location()).dot(Vector3::from(axis.direction()));
            let cos = the_dir_extr.dot(axis.direction());
            let vv = Vector3::from(the_dir_extr) * (a_local_real / cos);
            new_c = live(&the_new_curve, "curve")?.translated(vv)?.cast();
            // it is possible to calculate PCurve
            let l2d: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
                Point2::new(0., a_local_real / cos),
                Direction2::new(1., 0.)?,
            ))
            .cast();

            let einf = self.my_e_map.change_from_index(ii)?;
            if pc1 {
                *einf.change_first_pc() = l2d;
            } else {
                *einf.change_second_pc() = l2d;
            }
        } else {
            s1 = self.my_f_map.find_from_key(&first_face)?.geometry().clone();
            s2 = self.my_f_map.find_from_key(&second_face)?.geometry().clone();

            // PCurves are not calculated immediately for 2 reasons:
            // 1 - If ProjLib should make an Approx, it is stupid to approximate the
            //     entire intersection curve.
            // 2 - Additionally, if YaRev, there is a risk to not be SameRange.
            let i2s = GeomIntIntSS::with_surfaces(
                &s1,
                &s2,
                precision::confusion(),
                GeomIntIntSSApprox {
                    approx: true,
                    approx_s1: false,
                    approx_s2: false,
                },
            )?;

            if !i2s.is_done() || i2s.nb_lines() <= 0 {
                self.fail(DraftErrorStatus::EdgeRecomputation, the_edge.clone().into_shape());
                return Ok(None);
            }

            let mut glob2_min = f64::MAX;

            if !live(&i2s.line(1)?, "curve")?.is_instance_name("Geom_BSplineCurve") {
                let mut dist2_min;
                let mut imin = 0;
                let mut new_c_tmp: Handle<dyn GeomCurve>;
                for i in 1..=i2s.nb_lines() {
                    let the_curve = GeomAdaptorCurve::with_curve(&i2s.line(i)?)?;
                    let my_ext_pc = ExtPC::with_curve(pfv, &the_curve, 1.0e-10)?;

                    let mut locpmin = 0.;
                    if my_ext_pc.is_done() {
                        dist2_min = f64::MAX;
                        if my_ext_pc.nb_ext()? >= 1 {
                            dist2_min = my_ext_pc.square_distance(1)?;
                            locpmin = my_ext_pc.point(1)?.parameter();
                        }
                        if my_ext_pc.nb_ext()? == 2 && dist2_min > precision::square_confusion() {
                            // to avoid incorrectly choosing the image
                            // of the first vertex of the initial edge
                            let d1_2 = my_ext_pc.square_distance(1)?;
                            let d2_2 = my_ext_pc.square_distance(2)?;
                            if d1_2 > 1.21 * d2_2 {
                                dist2_min = my_ext_pc.square_distance(2)?;
                                locpmin = my_ext_pc.point(2)?.parameter();
                            } else if d2_2 > 1.21 * d1_2 {
                                dist2_min = my_ext_pc.square_distance(1)?;
                                locpmin = my_ext_pc.point(1)?.parameter();
                            } else {
                                let pfvpar = my_ext_pc.point(1)?.parameter();
                                let plvpar = my_ext_pc.point(2)?.parameter();
                                new_c_tmp = i2s.line(i)?;

                                let pfvprim = live(&new_c_tmp, "curve")?.value(pfvpar)?;
                                let plvprim = live(&new_c_tmp, "curve")?.value(plvpar)?;

                                let mut the_surf: Handle<dyn GeomSurface> = Handle::default();
                                if detrompeur == 1 {
                                    s1 = untrimmed_surface(&s1)?;
                                    the_surf = s1.clone();
                                } else if detrompeur == 2 {
                                    s2 = untrimmed_surface(&s2)?;
                                    the_surf = s2.clone();
                                }
                                if detrompeur != 0 && detrompeur != 4 {
                                    let (mut ul, mut vl, mut uf, mut vf) = (0., 0., 0., 0.);
                                    let (mut ufprim, mut ulprim, mut vfprim, mut vlprim) = (0., 0., 0., 0.);

                                    let ts = live(&the_surf, "surface")?;
                                    if ts.is_instance_name("Geom_Plane") {
                                        // The native code reads the second surface here.
                                        let pl = live(&as_plane(&s2), "plane")?.pln();
                                        (ul, vl) = elslib::plane_parameters(pl.position(), plv);
                                        (uf, vf) = elslib::plane_parameters(pl.position(), pfv);
                                        (ulprim, vlprim) = elslib::plane_parameters(pl.position(), plvprim);
                                        (ufprim, vfprim) = elslib::plane_parameters(pl.position(), pfvprim);
                                    } else if ts.is_instance_name("Geom_CylindricalSurface") {
                                        let cy = live(&s2.clone().cast::<GeomCylindricalSurface>(), "cylinder")?
                                            .cylinder()?;
                                        (ul, vl) = elslib::cylinder_parameters(cy.position(), cy.radius(), plv);
                                        (uf, vf) = elslib::cylinder_parameters(cy.position(), cy.radius(), pfv);
                                        (ulprim, vlprim) =
                                            elslib::cylinder_parameters(cy.position(), cy.radius(), plvprim);
                                        (ufprim, vfprim) =
                                            elslib::cylinder_parameters(cy.position(), cy.radius(), pfvprim);
                                    } else {
                                        detrompeur = 4;
                                    }

                                    if detrompeur == 1 || detrompeur == 2 {
                                        let v1 = Vector2::new(ul - ufprim, vl - vfprim);
                                        let norm = Vector2::new(vf - vfprim, ufprim - uf);
                                        let v2 = Vector2::new(ulprim - ufprim, vlprim - vfprim);
                                        if v1.dot(norm) * v2.dot(norm) < 0. {
                                            dist2_min = my_ext_pc.square_distance(2)?;
                                            locpmin = my_ext_pc.point(2)?.parameter();
                                        }
                                    }
                                }
                            }
                        }
                        let nb_ext = my_ext_pc.nb_ext()?;
                        if nb_ext == 1 || nb_ext > 2 || detrompeur == 4 {
                            dist2_min = my_ext_pc.square_distance(1)?;
                            locpmin = my_ext_pc.point(1)?.parameter();
                            for j in 2..=nb_ext {
                                let dist2 = my_ext_pc.square_distance(j)?;
                                if dist2 < dist2_min {
                                    dist2_min = dist2;
                                    locpmin = my_ext_pc.point(j)?.parameter();
                                }
                            }
                        } else if nb_ext < 1 {
                            let (dist1_2, dist2_2, _, _) = my_ext_pc.trimmed_square_distances();
                            if dist1_2 < dist2_2 {
                                dist2_min = dist1_2;
                                locpmin = the_curve.first_parameter()?;
                            } else {
                                dist2_min = dist2_2;
                                locpmin = the_curve.last_parameter()?;
                            }
                        }

                        if dist2_min < glob2_min {
                            glob2_min = dist2_min;
                            imin = i;
                            pmin = locpmin;
                        }
                    }
                }
                if imin == 0 {
                    self.fail(DraftErrorStatus::EdgeRecomputation, the_edge.clone().into_shape());
                    return Ok(None);
                }

                new_c = i2s.line(imin)?;

                let newd1;
                (pfv, newd1) = live(&new_c, "curve")?.d1(pmin)?;
                let ya_rev = d1fv.dot(newd1) < 0.;

                if ya_rev {
                    live(&new_c, "curve")?.reverse()?;
                }

                let einf = self.my_e_map.change_from_index(ii)?;
                if i2s.has_line_on_s1(imin)? {
                    *einf.change_first_pc() = i2s.line_on_s1(imin)?;
                    if ya_rev {
                        live(einf.first_pc(), "pcurve")?.reverse()?;
                    }
                }

                if i2s.has_line_on_s2(imin)? {
                    *einf.change_second_pc() = i2s.line_on_s2(imin)?;
                    if ya_rev {
                        live(einf.second_pc(), "pcurve")?.reverse()?;
                    }
                }
            } else {
                // i2s.Line(1) is BSplineCurve
                // Find the first curve to glue
                let mut candidates: Vec<Handle<dyn GeomCurve>> = Vec::new();
                let ls1 = live(&s1, "surface")?;
                if ls1.is_instance_name("Geom_CylindricalSurface") || ls1.is_instance_name("Geom_ConicalSurface") {
                    for i in 1..=i2s.nb_lines() {
                        let a_curve = i2s.line(i)?;
                        let ac = live(&a_curve, "curve")?;
                        let mut pnt = ac.value(ac.first_parameter())?;
                        let mut projector =
                            ProjectPointOnSurf::with_tolerance(pnt, &s1, precision::confusion(), ExtAlgo::Grad)?;
                        let (mut u, _) = projector.lower_distance_parameters()?;
                        if u.abs() <= precision::confusion() || (u - 2. * PI).abs() <= precision::confusion() {
                            candidates.push(a_curve);
                        } else {
                            pnt = ac.value(ac.last_parameter())?;
                            projector.init_tolerance(pnt, &s1, precision::confusion(), ExtAlgo::Grad)?;
                            (u, _) = projector.lower_distance_parameters()?;
                            if u.abs() <= precision::confusion() || (u - 2. * PI).abs() <= precision::confusion() {
                                ac.reverse()?;
                                candidates.push(a_curve);
                            }
                        }
                    }

                    if candidates.is_empty() {
                        for i in 1..=i2s.nb_lines() {
                            candidates.push(i2s.line(i)?);
                        }
                    }
                } else {
                    for i in 1..=i2s.nb_lines() {
                        candidates.push(i2s.line(i)?);
                    }
                }

                let mut first_curve: Handle<dyn GeomCurve> = Handle::default();
                if candidates.len() > 1 {
                    let mut dist_min = precision::infinite();
                    for a_curve in &candidates {
                        let ac = live(a_curve, "curve")?;
                        let pnt = ac.value(ac.first_parameter())?;
                        let dist = pnt.distance(&pfv);
                        if dist - dist_min < -precision::confusion() {
                            dist_min = dist;
                            first_curve = a_curve.clone();
                        }
                    }
                } else {
                    first_curve = candidates
                        .first()
                        .cloned()
                        .ok_or_else(|| Failure::with_message("Standard_OutOfRange", "NCollection_Sequence::Value"))?;
                }

                // Glueing
                let mut curves: Vec<Handle<dyn GeomCurve>> = Vec::new();
                for i in 1..=i2s.nb_lines() {
                    let line = i2s.line(i)?;
                    if first_curve != line {
                        curves.push(line);
                    }
                }

                let mut to_glue: Vec<Handle<dyn GeomCurve>> = Vec::new();
                let fc = live(&first_curve, "curve")?;
                let mut end_point = fc.value(fc.last_parameter())?;
                let mut added = true;
                while added {
                    added = false;
                    for i in 0..curves.len() {
                        let a_curve = curves[i].clone();
                        let ac = live(&a_curve, "curve")?;
                        let pfirst = ac.value(ac.first_parameter())?;
                        let plast = ac.value(ac.last_parameter())?;
                        if pfirst.distance(&end_point) <= precision::confusion() {
                            to_glue.push(a_curve);
                            end_point = plast;
                            curves.remove(i);
                            added = true;
                            break;
                        }
                        if plast.distance(&end_point) <= precision::confusion() {
                            ac.reverse()?;
                            to_glue.push(a_curve);
                            end_point = pfirst;
                            curves.remove(i);
                            added = true;
                            break;
                        }
                    }
                }

                let bounded = |c: &Handle<dyn GeomCurve>| -> Handle<dyn GeomBoundedCurve> {
                    c.clone().cast::<GeomBSplineCurve>().cast()
                };
                let mut concat =
                    CompCurveToBSplineCurve::with_curve(&bounded(&first_curve), ParameterisationType::TgtThetaOver2)?;
                for glued in &to_glue {
                    concat.add(&bounded(glued), precision::confusion(), true, true, 0)?;
                }

                new_c = concat.bspline_curve().cast();

                let the_curve = GeomAdaptorCurve::with_curve(&new_c)?;
                let my_ext_pc = ExtPC::with_curve(pfv, &the_curve, 1.0e-10)?;
                let mut dist2_min = f64::MAX;
                for i in 1..=my_ext_pc.nb_ext()? {
                    if my_ext_pc.is_min(i)? {
                        let dist2 = my_ext_pc.square_distance(i)?;
                        if dist2 < dist2_min {
                            dist2_min = dist2;
                            pmin = my_ext_pc.point(i)?.parameter();
                        }
                    }
                }
                let newd1;
                (pfv, newd1) = live(&new_c, "curve")?.d1(pmin)?;
                let ya_rev = d1fv.dot(newd1) < 0.;

                if ya_rev {
                    live(&new_c, "curve")?.reverse()?;
                }
            }

            let einf = self.my_e_map.change_from_index(ii)?;
            einf.set_tolerance(einf.tolerance().max(i2s.tol_reached3d()));
        }
        let _ = pfv;
        Ok(Some(new_c))
    }

    /// The line of a tangency edge through its fixed point along the extrusion
    /// direction (with the p-curves on an extrusion of a circle), oriented as the edge;
    /// `None` on failure (the error is set).
    fn tangency_line(
        &mut self,
        ii: usize,
        the_edge: &Edge,
        c: &Handle<dyn GeomCurve>,
        ptfixe: Point3,
    ) -> Result<Option<Handle<dyn GeomCurve>>, Failure> {
        let (f1, f2) = {
            let einf = self.my_e_map.find_from_index(ii)?;
            (einf.first_face().clone(), einf.second_face().clone())
        };

        let mut a_local_s1 = self.my_f_map.find_from_key(&f1)?.geometry().clone();
        let mut a_local_s2 = self.my_f_map.find_from_key(&f2)?.geometry().clone();
        if a_local_s1.is_null() || a_local_s2.is_null() {
            self.fail(DraftErrorStatus::EdgeRecomputation, the_edge.clone().into_shape());
            return Ok(None);
        }
        a_local_s1 = untrimmed_surface(&a_local_s1)?;
        a_local_s2 = untrimmed_surface(&a_local_s2)?;

        // The native direction stays the default one on other surfaces.
        let mut dirextr = Direction3::default();
        let einf = self.my_e_map.change_from_index(ii)?;
        let (ls1, ls2) = (live(&a_local_s1, "surface")?, live(&a_local_s2, "surface")?);
        let circle_pcurve = |s: &Handle<dyn GeomSurface>| -> Result<Option<Handle<dyn Geom2dCurve>>, Failure> {
            let sel = live(&s.clone().cast::<GeomSurfaceOfLinearExtrusion>(), "extrusion")?.basis_curve();
            match sel.cast::<GeomCircle>().get() {
                Some(gcir) => {
                    let u = elclib::parameter(&gcir.circ()?, ptfixe)?;
                    Ok(Some(
                        Handle::new(Geom2dLine::from_point(Point2::new(u, 0.), Direction2::new(0., 1.)?)).cast(),
                    ))
                }
                None => Ok(None),
            }
        };
        if ls1.is_instance_name("Geom_CylindricalSurface") {
            let cyl = live(&a_local_s1.clone().cast::<GeomCylindricalSurface>(), "cylinder")?.cylinder()?;
            dirextr = cyl.axis().direction();
        } else if ls1.is_instance_name("Geom_SurfaceOfLinearExtrusion") {
            dirextr = live(&a_local_s1.clone().cast::<GeomSurfaceOfLinearExtrusion>(), "extrusion")?.direction();
            // Here it is possible to calculate PCurve.
            if let Some(pc1) = circle_pcurve(&a_local_s1)? {
                *einf.change_first_pc() = pc1;
            }
        } else if ls2.is_instance_name("Geom_CylindricalSurface") {
            let cyl = live(&a_local_s2.clone().cast::<GeomCylindricalSurface>(), "cylinder")?.cylinder()?;
            dirextr = cyl.axis().direction();
        } else if ls2.is_instance_name("Geom_SurfaceOfLinearExtrusion") {
            dirextr = live(&a_local_s2.clone().cast::<GeomSurfaceOfLinearExtrusion>(), "extrusion")?.direction();
            // Here it is possible to calculate PCurve.
            if let Some(pc2) = circle_pcurve(&a_local_s2)? {
                *einf.change_second_pc() = pc2;
            }
        }
        let new_c: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(ptfixe, dirextr)).cast();

        let (_, d1fv) = live(c, "curve")?.d1(0.)?;
        let (_, newd1) = live(&new_c, "curve")?.d1(0.)?;
        let ya_rev = d1fv.dot(newd1) < 0.;
        if ya_rev {
            live(&new_c, "curve")?.reverse()?;
            if !einf.first_pc().is_null() {
                live(einf.first_pc(), "pcurve")?.reverse()?;
            }
            if !einf.second_pc().is_null() {
                live(einf.second_pc(), "pcurve")?.reverse()?;
            }
        }
        Ok(Some(new_c))
    }

    /// The new points of the vertices: the intersection of the curve of a reference
    /// edge with the surface of a face of a non tangent edge, else the projection on the
    /// curves of the two first edges; then the parameters on the new curves.
    fn compute_vertices(&mut self) -> Result<bool, Failure> {
        for ii in 1..=self.my_v_map.extent() {
            let tvv = topods::vertex(self.my_v_map.find_key(ii)?)?;
            let chosen = {
                let DraftState { my_f_map, my_e_map, my_v_map, .. } = self;
                choose(my_f_map, my_e_map, &tvv, my_v_map.change_from_index(ii)?)?
            };
            let Some((ac, as_surface)) = chosen else {
                // no concerted edge => alignment of two consecutive edges.
                let DraftState { my_f_map, my_e_map, my_v_map, .. } = self;
                let vinf = my_v_map.change_from_index(ii)?;
                let mut pvt = Point3::default();
                *vinf.change_geometry() = pvt;
                vinf.init_edge_iterator();
                if vinf.more_edge() {
                    let edg1 = vinf.edge()?;
                    let vtori = BRepTool::pnt(&tvv)?;
                    let projector = ProjectPointOnCurve::with_curve(vtori, my_e_map.find_from_key(&edg1)?.geometry())?; // patch
                    pvt = projector.nearest_point()?;

                    let dion = pvt.square_distance(&vtori);
                    vinf.next_edge();
                    if vinf.more_edge() {
                        let edg2 = vinf.edge()?;
                        let einf2 = my_e_map.change_from_key(&edg2)?;
                        let opvt = live(einf2.geometry(), "edge geometry")?.value(vinf.parameter(&edg2)?)?;

                        if opvt.square_distance(&vtori) < dion {
                            pvt = opvt;
                        }
                        let geometry = einf2.geometry().clone();
                        let param = smart_or_plain(my_f_map, einf2, &geometry, BRepTool::edge_tolerance(&edg2)?, pvt)?;
                        *vinf.change_parameter(&edg2)? = param;
                    }

                    *vinf.change_geometry() = pvt;
                    let einf1 = my_e_map.change_from_key(&edg1)?;
                    let geometry = einf1.geometry().clone();
                    let param = smart_or_plain(my_f_map, einf1, &geometry, BRepTool::edge_tolerance(&edg1)?, pvt)?;
                    *vinf.change_parameter(&edg1)? = param;
                    continue;
                }

                self.fail(DraftErrorStatus::VertexRecomputation, tvv.into_shape());
                return Ok(false);
            };

            let ac = Handle::new(ac);
            let as_adaptor = Handle::new(GeomAdaptorSurface::with_surface(&as_surface)?);
            let hac: Handle<dyn Adaptor3dCurve> = ac.clone().cast();
            let has: Handle<dyn Adaptor3dSurface> = as_adaptor.clone().cast();
            let mut myintcs = HInter::default();
            myintcs.perform(&hac, &has)?;
            if !myintcs.is_done() {
                self.fail(DraftErrorStatus::VertexRecomputation, tvv.into_shape());
                return Ok(false);
            }

            let vtori = BRepTool::pnt(&tvv)?;

            let nbsol = myintcs.nb_points()?;
            let pvt: Point3 = if nbsol <= 0 {
                let extr = ExtCS::with_curve(live(&ac, "adaptor")?, live(&as_adaptor, "adaptor")?, precision::p_confusion(), precision::p_confusion())?;

                if !extr.is_done() || extr.nb_ext()? == 0 {
                    self.fail(DraftErrorStatus::VertexRecomputation, tvv.into_shape());
                    return Ok(false);
                }

                let mut disref = f64::MAX;
                let mut iref = 0;
                for i in 1..=extr.nb_ext()? {
                    let (pc, _) = extr.points(i)?;
                    let distemp = pc.value().square_distance(&vtori);
                    if distemp < disref {
                        disref = distemp;
                        iref = i;
                    }
                }
                *extr.points(iref)?.0.value()
            } else {
                let mut disref = f64::MAX;
                let mut iref = 0;
                for i in 1..=nbsol {
                    let distemp = myintcs.point(i)?.pnt().square_distance(&vtori);
                    if distemp < disref {
                        disref = distemp;
                        iref = i;
                    }
                }
                myintcs.point(iref)?.pnt()
            };

            let DraftState { my_f_map, my_e_map, my_v_map, .. } = self;
            let vinf = my_v_map.change_from_index(ii)?;
            *vinf.change_geometry() = pvt;

            vinf.init_edge_iterator();
            while vinf.more_edge() {
                let edg = vinf.edge()?;
                let initpar = vinf.parameter(&edg)?;
                let einf = my_e_map.change_from_key(&edg)?;
                let geometry = einf.geometry().clone();
                let (param, done) = parameter(&geometry, pvt)?;
                if done != 0 {
                    let s1 = my_f_map.find_from_key(einf.first_face())?.geometry().clone();
                    let s2 = my_f_map.find_from_key(einf.second_face())?.geometry().clone();
                    *vinf.change_parameter(&edg)? =
                        super::params::smart_parameter(einf, BRepTool::edge_tolerance(&edg)?, pvt, done, &s1, &s2)?;
                } else {
                    if (initpar - param).abs() > precision::p_confusion() {
                        let a_c = BRepTool::curve_with_location(&edg)?;
                        if live(&a_c.curve, "curve")?.is_instance_name("Geom_TrimmedCurve") {
                            einf.set_new_geometry(true);
                        }
                    }
                    *vinf.change_parameter(&edg)? = param;
                }
                vinf.next_edge();
            }
        }
        Ok(true)
    }

    /// The small loop of validation/protection: an intersection edge running against
    /// its source edge is reversed, and the vertex parameters must increase along each
    /// edge (closed curves are unwrapped).
    fn validate_edges(&mut self) -> Result<(), Failure> {
        for i in 1..=self.my_e_map.extent() {
            let edg = topods::edge(self.my_e_map.find_key(i)?)?;

            let (mut vf, mut vl) = topexp::vertices(&edg, false);
            if edg.orientation() == Orientation::Reversed {
                vf.reverse();
                vl.reverse();
            }

            if self.my_v_map.contains(&vf) && self.my_v_map.contains(&vl) {
                // Here, we compare directions of the source edge (from input shape)
                // and corresponding selected part of the intersection edge.
                // If these directions are opposite then we reverse intersection edge
                // and recompute corresponding vertex-parameters.

                let a_par_f = self.my_v_map.change_from_key(&vf)?.parameter(&edg)?;
                let a_par_l = self.my_v_map.change_from_key(&vl)?.parameter(&edg)?;

                if a_par_l < a_par_f {
                    let (a_s_curve, _, _) = BRepTool::curve(&edg)?;
                    let an_int_curv = self.my_e_map.find_from_key(&edg)?.geometry().clone();
                    let sc = live(&a_s_curve, "curve")?;
                    let (_, mut a_dir_of) = sc.d1(BRepTool::parameter(&vf, &edg)?)?;
                    let (_, mut a_dir_ol) = sc.d1(BRepTool::parameter(&vl, &edg)?)?;
                    let ic = live(&an_int_curv, "edge geometry")?;
                    let (a_pf, mut a_dir_nf) = ic.d1(a_par_f)?;
                    let (a_pl, mut a_dir_nl) = ic.d1(a_par_l)?;

                    for dir in [&mut a_dir_nf, &mut a_dir_nl, &mut a_dir_of, &mut a_dir_ol] {
                        let a_sq_magn = dir.square_magnitude();
                        if a_sq_magn > precision::square_confusion() {
                            *dir /= a_sq_magn.sqrt();
                        }
                    }

                    let a_cos_f = a_dir_nf.dot(a_dir_of);
                    let a_cos_l = a_dir_nl.dot(a_dir_ol);
                    let a_cos_max = if a_cos_f.abs() > a_cos_l.abs() { a_cos_f } else { a_cos_l };

                    if a_cos_max < 0.0 {
                        ic.reverse()?;
                        *self.my_e_map.change_from_key(&edg)?.change_geometry() = an_int_curv.clone();
                        let (a_par, an_err) = parameter(&an_int_curv, a_pf)?;
                        if an_err == 0 {
                            *self.my_v_map.change_from_key(&vf)?.change_parameter(&edg)? = a_par;
                        }
                        let (a_par, an_err) = parameter(&an_int_curv, a_pl)?;
                        if an_err == 0 {
                            *self.my_v_map.change_from_key(&vl)?.change_parameter(&edg)? = a_par;
                        }
                    }
                }
            }

            let mut pf = match self.new_parameter(&vf, &edg)? {
                Some((p, _)) => p,
                None => BRepTool::parameter(&vf, &edg)?,
            };
            let mut pl = match self.new_parameter(&vl, &edg)? {
                Some((p, _)) => p,
                None => BRepTool::parameter(&vl, &edg)?,
            };
            if pl <= pf {
                let the_curve = self.my_e_map.find_from_key(&edg)?.geometry().clone();
                let curve = live(&the_curve, "edge geometry")?;
                if curve.is_closed()? {
                    // pf >= pl
                    let (first_par, last_par) = (curve.first_parameter(), curve.last_parameter());
                    let pconf = precision::p_confusion();
                    if (pf - last_par).abs() <= pconf {
                        pf = first_par;
                    } else if (pl - first_par).abs() <= pconf {
                        pl = last_par;
                    }

                    if pl <= pf {
                        pl += last_par - first_par;
                    }
                }
                if pl <= pf {
                    self.fail(DraftErrorStatus::EdgeRecomputation, edg.into_shape());
                    return Ok(());
                }
            }
            if self.my_v_map.contains(&vf) {
                *self.my_v_map.change_from_key(&vf)?.change_parameter(&edg)? = pf;
            }
            if self.my_v_map.contains(&vl) {
                *self.my_v_map.change_from_key(&vl)?.change_parameter(&edg)? = pl;
            }
        }
        Ok(())
    }
}
