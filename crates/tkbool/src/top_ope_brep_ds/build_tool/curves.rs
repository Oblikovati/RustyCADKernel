// Rust translation of the curve members of TopOpeBRepDS_BuildTool (UpdateEdgeCurveTol,
// ApproxCurves, ComputePCurves, PutPCurves, RecomputeCurves and the functions
// FUN_UisoLineOnSphe, FUN_getUV, FUN_reversePC and FUN_makeUisoLineOnSphe in
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_BuildTool.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The 3d curves and p-curves of the section edges: approximated from walking lines,
//! or projected on the faces, with the vertex parameters and tolerances updated.
use super::TopOpeBRepDSBuildTool;
use crate::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSHDataStructure};
use crate::top_ope_brep_tool::{
    SectionCurves, TopOpeBRepToolCurveTool, TopOpeBRepToolShapeTool, basis_curve2d,
};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCurve, GeomSphericalSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeomalgo::geom_api::{ProjectPointOnCurve, ProjectPointOnSurf};
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Point3, Vector2};

/// The new tolerance and range of a section edge: `(newtol, newparmin, newparmax)`.
pub type EdgeCurveTol = (f64, f64, f64);

/// Standard_NullObject of a null handle met by the build tool.
fn null_object(what: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("TopOpeBRepDS_BuildTool: null {what}; expected a geometry"),
        "TopOpeBRepDS_BuildTool",
    )
}

/// `FUN_UisoLineOnSphe(F, PC)`: whether the p-curve is an iso-U line on the spherical
/// basis surface of the face; `fun_uiso_line_on_sphe(&face, &pc)?`.
pub fn fun_uiso_line_on_sphe(f: &Shape, pc: &Handle<dyn Geom2dCurve>) -> Result<bool, Failure> {
    if pc.is_null() {
        return Ok(false);
    }
    let sss = TopOpeBRepToolShapeTool::basis_surface_of_face(&topods::face(f)?)?;
    let lll = basis_curve2d(pc)?;
    let sphere: Handle<GeomSphericalSurface> = sss.cast();
    let line: Handle<Geom2dLine> = lll.cast();
    match (sphere.is_null(), line.get()) {
        (false, Some(l)) => {
            Ok(l.direction().x().abs() < precision::parametric(precision::confusion()))
        }
        _ => Ok(false),
    }
}

/// `FUN_getUV(surf, C3D, par3d, u0, v0)`: the UV of the nearest projection of the curve
/// point on the surface; `fun_get_uv(&s, &c, 0.5)?`.
pub fn fun_get_uv(
    surf: &Handle<dyn GeomSurface>,
    c3d: &Handle<dyn GeomCurve>,
    par3d: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    let p3d = c3d
        .get()
        .ok_or_else(|| null_object("curve"))?
        .value(par3d)?;
    let pons = ProjectPointOnSurf::with_surface(p3d, surf, ExtAlgo::Grad)?;
    if pons.nb_points()? < 1 {
        return Ok(None);
    }
    Ok(Some(pons.lower_distance_parameters()?))
}

/// `FUN_reversePC(PCnew, F, P3DC3D, par2d, tol)`: whether the face point of the p-curve
/// at `par2d` is not within `tol` of `p3dc3d`; then the direction of its basis line is
/// reversed in place (Standard_NullObject natively when the basis is not a line);
/// `fun_reverse_pc(&pc, &face, p, 0.5, 1e-7)?`.
pub fn fun_reverse_pc(
    pcnew: &Handle<dyn Geom2dCurve>,
    f: &Shape,
    p3dc3d: Point3,
    par2d: f64,
    tol: f64,
) -> Result<bool, Failure> {
    let p2d = pcnew
        .get()
        .ok_or_else(|| null_object("p-curve"))?
        .value(par2d)?;
    let bas = BRepAdaptorSurface::with_face(&topods::face(f)?, false)?;
    let p3d = bas.value(p2d.x(), p2d.y())?;
    let reversed = !p3d.is_equal(&p3dc3d, tol);
    if reversed {
        let line: Handle<Geom2dLine> = basis_curve2d(pcnew)?.cast();
        let l = line.get().ok_or_else(|| null_object("2d line"))?;
        l.set_direction(l.direction().reversed());
    }
    Ok(reversed)
}

/// `FUN_makeUisoLineOnSphe(F, C3D, PCnew, tol3d)`: the basis line of the p-curve set in
/// place to the iso-U of the projections of two curve points (V growing or not),
/// through the projection of the curve start; false when the projections fail or are
/// not on one iso-U; `fun_make_uiso_line_on_sphe(&face, &c, &pc, 1e-7)?`.
pub fn fun_make_uiso_line_on_sphe(
    f: &Shape,
    c3d: &Handle<dyn GeomCurve>,
    pcnew: &Handle<dyn Geom2dCurve>,
    tol3d: f64,
) -> Result<bool, Failure> {
    let c = c3d.get().ok_or_else(|| null_object("curve"))?;
    let (p3df, p3dl) = (c.first_parameter(), c.last_parameter());
    let (deltainf, deltasup): (f64, f64) = (0.243234, 0.543345);
    let par3dinf = (1. - deltainf) * p3df + deltainf * p3dl;
    let par3dsup = (1. - deltasup) * p3df + deltasup * p3dl;
    let surf = BRepTool::surface(&topods::face(f)?)?;
    let Some((uinf, vinf)) = fun_get_uv(&surf, c3d, par3dinf)? else {
        return Ok(false);
    };
    let Some((usup, vsup)) = fun_get_uv(&surf, c3d, par3dsup)? else {
        return Ok(false);
    };
    let tol = precision::parametric(tol3d);
    if (uinf - usup).abs() > tol {
        return Ok(false);
    }
    let vdir = if vsup - vinf > -tol {
        Direction2::Y
    } else {
        Direction2::new(0., -1.)?
    };
    let scale = p3df - par3dinf;
    let origin =
        Point2::new(uinf, vinf).translated(Vector2::new(vdir.x() * scale, vdir.y() * scale));
    let line: Handle<Geom2dLine> = basis_curve2d(pcnew)?.cast();
    if let Some(l) = line.get() {
        l.set_lin2d(Line2::new(origin, vdir));
    } else if !basis_curve2d(pcnew)?.is_null() {
        return Err(null_object("2d line"));
    }
    Ok(true)
}

impl TopOpeBRepDSBuildTool {
    /// The parameters of the vertices of the edge (0 when it lacks one).
    fn oriented_parameters(
        &self,
        e: &Edge,
    ) -> Result<
        (
            rustycad_tkbrep::topods::Vertex,
            rustycad_tkbrep::topods::Vertex,
            f64,
            f64,
        ),
        Failure,
    > {
        let (vmin, vmax, pars) = self.get_oriented_edge_vertices(e)?;
        let (parmin, parmax) = pars.unwrap_or((0., 0.));
        Ok((vmin, vmax, parmin, parmax))
    }

    /// `UpdateEdgeCurveTol(F1, F2, E, C3Dnew, tol3d, tol2d1, tol2d2, newtol, newparmin,
    /// newparmax)`: the vertices of the edge moved to the bounds of a trimmed or
    /// B-spline new curve (tolerances at least 1e-7), the curve set with tolerance 1e-7
    /// and the INTERNAL vertices projected on it; `None` for a null curve (the outputs
    /// untouched natively); `bt.update_edge_curve_tol(&mut e, &c)?`.
    pub fn update_edge_curve_tol(
        &self,
        e: &mut Edge,
        c3dnew: &Handle<dyn GeomCurve>,
    ) -> Result<Option<EdgeCurveTol>, Failure> {
        let Some(c) = c3dnew.get() else {
            return Ok(None);
        };
        let bb = BRepBuilder::default();
        let newtol: f64 = 1.0e-7;
        let (vmin, vmax, parmin, parmax) = self.oriented_parameters(e)?;
        let tolmin = BRepTool::vertex_tolerance(&vmin)?.max(newtol);
        let tolmax = BRepTool::vertex_tolerance(&vmax)?.max(newtol);
        let trimmed: Handle<GeomTrimmedCurve> = c3dnew.clone().cast();
        let bspline: Handle<GeomBSplineCurve> = c3dnew.clone().cast();
        let (newparmin, newparmax) = if trimmed.is_null() && bspline.is_null() {
            (parmin, parmax)
        } else {
            (c.first_parameter(), c.last_parameter())
        };
        if vmin.orientation() == Orientation::Forward {
            bb.update_vertex_on_edge(&vmin, newparmin, e, tolmin)?;
            bb.update_vertex_on_edge(&vmax, newparmax, e, tolmax)?;
        } else {
            bb.update_vertex_on_edge(&vmin, newparmax, e, tolmin)?;
            bb.update_vertex_on_edge(&vmax, newparmin, e, tolmax)?;
        }
        let mut es = e.clone().into_shape();
        self.curve3d(&mut es, c3dnew, newtol)?;
        self.project_internal_vertices(e, c3dnew, (newparmin, newparmax))?;
        Ok(Some((newtol, newparmin, newparmax)))
    }

    /// The INTERNAL vertices of the edge moved to their projections on the curve.
    fn project_internal_vertices(
        &self,
        e: &Edge,
        c3dnew: &Handle<dyn GeomCurve>,
        (newparmin, newparmax): (f64, f64),
    ) -> Result<(), Failure> {
        let bb = BRepBuilder::default();
        for vi in Explorer::new(e.as_ref(), ShapeEnum::Vertex, ShapeEnum::Shape)? {
            if vi.orientation() != Orientation::Internal {
                continue;
            }
            let v = topods::vertex(&vi)?;
            let p = BRepTool::pnt(&v)?;
            let tolvi = TopOpeBRepToolShapeTool::tolerance(&vi)?;
            let dm = ProjectPointOnCurve::with_range(p, c3dnew, newparmin, newparmax)?;
            if dm.extrema()?.is_done() && dm.nb_points() > 0 {
                bb.update_vertex_on_edge(&v, dm.lower_distance_parameter()?, e, tolvi)?;
            }
        }
        Ok(())
    }

    /// The improvement of the input curves by `MakeCurves` written back where natively
    /// written through const_casts: the 3d curve of the data structure curve of `c`
    /// and the p-curves of its interferences.
    fn write_back_input(
        c: &TopOpeBRepDSCurve,
        input: &SectionCurves,
        hds: &TopOpeBRepDSHDataStructure,
    ) -> Result<(), Failure> {
        if c.curve().object_address() != input.c3d.object_address() && c.ds_index() > 0 {
            hds.change_curve(c.ds_index(), |dc| *dc.change_curve() = input.c3d.clone());
        }
        if c.curve1()?.object_address() != input.pc1.object_address() {
            c.set_curve1(&input.pc1)?;
        }
        if c.curve2()?.object_address() != input.pc2.object_address() {
            c.set_curve2(&input.pc2)?;
        }
        Ok(())
    }

    /// `ApproxCurves(C, E, inewC, HDS)`: a new curve of the structure (copy of `c`)
    /// with the approximation of the curves of `c` between the parameters of the edge
    /// vertices (or the curves of `c` with the edge tolerance when it fails); its index;
    /// `let inew = bt.approx_curves(&c, &mut e, &hds)?;`.
    pub fn approx_curves(
        &self,
        c: &TopOpeBRepDSCurve,
        e: &mut Edge,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
    ) -> Result<i32, Failure> {
        let h = hds.get().ok_or_else(|| null_object("data structure"))?;
        let mut new_c1 = TopOpeBRepDSCurve::default();
        let inew_c = h.make_curve(c, &mut new_c1)?;
        let new_c = h.curve(inew_c);
        let (f1, f2) = (new_c.shape1().clone(), new_c.shape2().clone());
        let mut input = SectionCurves {
            c3d: c.curve().clone(),
            pc1: c.curve1()?,
            pc2: c.curve2()?,
        };
        let (_, _, parmin, parmax) = self.oriented_parameters(e)?;
        let made = self
            .my_curve_tool
            .make_curves(parmin, parmax, &mut input, &f1, &f2)?;
        Self::write_back_input(c, &input, h)?;
        let (curves, (newtol, newparmin, newparmax)) = match made {
            None => (input, (BRepTool::edge_tolerance(e)?, parmin, parmax)),
            Some(m) => {
                let tol = self
                    .update_edge_curve_tol(e, &m.curves.c3d)?
                    .unwrap_or((0., 0., 0.));
                (m.curves, tol)
            }
        };
        h.change_curve(inew_c, |nc| -> Result<(), Failure> {
            if !curves.c3d.is_null() {
                nc.define_curve(&curves.c3d, newtol, false);
                nc.set_range(newparmin, newparmax);
            }
            if !curves.pc1.is_null() {
                nc.set_curve1(&curves.pc1)?;
            }
            if !curves.pc2.is_null() {
                nc.set_curve2(&curves.pc2)?;
            }
            Ok(())
        })?;
        Ok(inew_c)
    }

    /// The p-curves of a section edge without 3d curve (natively a projection of a
    /// null curve, refused by MakePCurveOnFace).
    fn compute_pcurves_without_curve(
        &self,
        c: &TopOpeBRepDSCurve,
        new_c: &mut TopOpeBRepDSCurve,
        (f1, f2): (&Shape, &Shape),
        (comppc1, comppc2): (bool, bool),
    ) -> Result<(), Failure> {
        let (mut tolreached2d1, mut tolreached2d2, mut tol) = (
            precision::confusion(),
            precision::confusion(),
            precision::confusion(),
        );
        let (mut pc1new, mut pc2new) = (
            Handle::<dyn Geom2dCurve>::default(),
            Handle::<dyn Geom2dCurve>::default(),
        );
        if comppc1 {
            (pc1new, tolreached2d1) =
                TopOpeBRepToolCurveTool::make_pcurve_on_face(f1, c.curve(), 0., 0.)?;
        }
        if comppc2 {
            (pc2new, tolreached2d2) =
                TopOpeBRepToolCurveTool::make_pcurve_on_face(f2, c.curve(), 0., 0.)?;
        }
        let r1 = TopOpeBRepToolShapeTool::resolution3d_of_face(&topods::face(f1)?, tolreached2d1)?;
        let r2 = TopOpeBRepToolShapeTool::resolution3d_of_face(&topods::face(f2)?, tolreached2d2)?;
        tol = tol.max(r1).max(r2);
        new_c.set_tolerance(tol);
        if !pc1new.is_null() {
            new_c.set_curve1(&pc1new)?;
        }
        if !pc2new.is_null() {
            new_c.set_curve2(&pc2new)?;
        }
        Ok(())
    }

    /// `ComputePCurves(C, E, newC, comppc1, comppc2, compc3d)`: the projections of the
    /// curve of `c` (trimmed to the vertex parameters on a periodic curve when
    /// `compc3d`) on both faces of `new_c`, the edge updated by UpdateEdgeCurveTol, the
    /// iso-U lines on spheres rebuilt, and `new_c` given the curves;
    /// `bt.compute_pcurves(&c, &mut e, &mut new_c, true, true, true)?`.
    pub fn compute_pcurves(
        &self,
        c: &TopOpeBRepDSCurve,
        e: &mut Edge,
        new_c: &mut TopOpeBRepDSCurve,
        comppc1: bool,
        comppc2: bool,
        compc3d: bool,
    ) -> Result<(), Failure> {
        let (f1, f2) = (new_c.shape1().clone(), new_c.shape2().clone());
        let (vmin, _, mut parmin, mut parmax) = self.oriented_parameters(e)?;
        let Some(c3d) = c.curve().get() else {
            return self.compute_pcurves_without_curve(c, new_c, (&f1, &f2), (comppc1, comppc2));
        };
        let mut c3dnew = c.curve().clone();
        if c3d.is_periodic() {
            let period = c3d.last_parameter() - c3d.first_parameter();
            let (f, l) = if vmin.orientation() == Orientation::Forward {
                (parmin, parmax)
            } else {
                (parmax, parmin)
            };
            (parmin, parmax) =
                elclib::adjust_periodic(f, f + period, precision::p_confusion(), f, l);
            if compc3d {
                c3dnew = Handle::new(GeomTrimmedCurve::new(
                    c.curve(),
                    parmin,
                    parmax,
                    true,
                    true,
                )?)
                .cast();
            }
        }
        let (mut pc1new, mut pc2new) = (
            Handle::<dyn Geom2dCurve>::default(),
            Handle::<dyn Geom2dCurve>::default(),
        );
        if comppc1 {
            pc1new = TopOpeBRepToolCurveTool::make_pcurve_on_face(&f1, &c3dnew, 0., 0.)?.0;
        }
        if comppc2 {
            pc2new = TopOpeBRepToolCurveTool::make_pcurve_on_face(&f2, &c3dnew, 0., 0.)?.0;
        }
        let (newtol, newparmin, newparmax) = self
            .update_edge_curve_tol(e, &c3dnew)?
            .unwrap_or((0., 0., 0.));
        for (f, pc) in [(&f1, &pc1new), (&f2, &pc2new)] {
            if fun_uiso_line_on_sphe(f, pc)? {
                fun_make_uiso_line_on_sphe(f, &c3dnew, pc, newtol)?;
            }
        }
        new_c.set_curve(&c3dnew, newtol);
        new_c.set_range(newparmin, newparmax);
        if !pc1new.is_null() {
            new_c.set_curve1(&pc1new)?;
        }
        if !pc2new.is_null() {
            new_c.set_curve2(&pc2new)?;
        }
        Ok(())
    }

    /// `PutPCurves(newC, E, comppc1, comppc2)`: the p-curves of `new_c` set on the edge
    /// on its faces (PCurve); `bt.put_pcurves(&new_c, &mut e, true, true)?`.
    pub fn put_pcurves(
        &self,
        new_c: &TopOpeBRepDSCurve,
        e: &mut Edge,
        comppc1: bool,
        comppc2: bool,
    ) -> Result<(), Failure> {
        let mut es = e.clone().into_shape();
        let pc1 = new_c.curve1()?;
        if !pc1.is_null() && comppc1 {
            let mut f1 = new_c.shape1().clone();
            self.pcurve(&mut f1, &mut es, &pc1)?;
        }
        let pc2 = new_c.curve2()?;
        if !pc2.is_null() && comppc2 {
            let mut f2 = new_c.shape2().clone();
            self.pcurve(&mut f2, &mut es, &pc2)?;
        }
        *e = topods::edge(&es)?;
        Ok(())
    }

    /// `RecomputeCurves(C, oldE, E, inewC, HDS)`: for a walking curve with
    /// approximation, the approximated curves (ApproxCurves) put on the edge; else,
    /// when p-curves are asked, a new curve of the structure with the walking p-curves
    /// or the projected ones (ComputePCurves) put on the edge; Standard_ProgramError for
    /// a missing face, vertex or 3d curve; the new curve index (0 when none is made);
    /// `let inew = bt.recompute_curves(&c, &old, &mut e, &hds)?;`.
    pub fn recompute_curves(
        &self,
        c: &TopOpeBRepDSCurve,
        _old_e: &Edge,
        e: &mut Edge,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
    ) -> Result<i32, Failure> {
        let gt = *self.my_curve_tool.get_geom_tool();
        let (compc3d, comppc1, comppc2) = (gt.comp_c3d(), gt.comp_pc1(), gt.comp_pc2());
        let (iswalk, approx) = (c.is_walk(), self.approximation());
        recompute_checks(c, e, (compc3d, comppc1, comppc2), iswalk && approx)?;
        let h = hds.get().ok_or_else(|| null_object("data structure"))?;
        if iswalk && approx {
            let inew_c = self.approx_curves(c, e, hds)?;
            self.put_pcurves(&h.curve(inew_c), e, comppc1, comppc2)?;
            return Ok(inew_c);
        }
        if !(comppc1 || comppc2) {
            return Ok(0);
        }
        let mut new_c1 = TopOpeBRepDSCurve::default();
        let inew_c = h.make_curve(c, &mut new_c1)?;
        let mut new_c = h.curve(inew_c);
        if iswalk {
            if compc3d && c.curve().is_null() {
                return Err(program_error("TopOpeBRepDS_BuildTool::RecomputeCurve 1"));
            }
            new_c.set_curve1(&c.curve1()?)?;
            new_c.set_curve2(&c.curve2()?)?;
        } else {
            self.compute_pcurves(c, e, &mut new_c, comppc1, comppc2, compc3d)?;
        }
        h.change_curve(inew_c, |nc| *nc = new_c.clone());
        self.put_pcurves(&new_c, e, comppc1, comppc2)?;
        Ok(inew_c)
    }
}

/// Standard_ProgramError of RecomputeCurves.
fn program_error(message: &str) -> Failure {
    Failure::with_type("Standard_ProgramError", message, "TopOpeBRepDS_BuildTool")
}

/// The preconditions of RecomputeCurves, in the native order.
fn recompute_checks(
    c: &TopOpeBRepDSCurve,
    e: &Edge,
    (compc3d, comppc1, comppc2): (bool, bool, bool),
    walk_approx: bool,
) -> Result<(), Failure> {
    if comppc1 && c.shape1().is_null() {
        return Err(program_error("TopOpeBRepDS_BuildTool::RecomputeCurve 2"));
    }
    if comppc2 && c.shape2().is_null() {
        return Err(program_error("TopOpeBRepDS_BuildTool::RecomputeCurve 3"));
    }
    let (vmin, vmax) = rustycad_tkbrep::topexp::vertices(e, false);
    if vmin.as_ref().is_null() {
        return Err(program_error("TopOpeBRepDS_BuildTool::RecomputeCurve 4"));
    }
    if vmax.as_ref().is_null() {
        return Err(program_error("TopOpeBRepDS_BuildTool::RecomputeCurve 5"));
    }
    if walk_approx && compc3d && c.curve().is_null() {
        return Err(program_error("TopOpeBRepDS_BuildTool::RecomputeCurve 1"));
    }
    Ok(())
}
