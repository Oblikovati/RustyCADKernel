// Rust translation of ChFi3d_Builder::Trunc and the file statics CompTra, CpInterf, CpSD,
// AdjustParam, ComputeAbscissa, ParamOnSpine, YaUnVoisin and ResetProl
// (ChFi3d_Builder_SpKP.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The particular cases of an edge copied, placed on the spine and cut at its ends.
use super::ChFi3dBuilder;
use crate::chfi_ds::{FaceInterference, SpineHandle, SurfData, SurfDataHandle, lock, shared};
use crate::chfi3d::{inter_plane_edge, kparticular};
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSDataStructure, TopOpeBRepDSSurface,
};
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkgeomalgo::hatch_gen::Domain;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::Direction3;
use std::f64::consts::PI;

/// `CompTra(O1, O2, isfirst)`: the composed orientation, reversed at the first end.
pub(super) fn comp_tra(o1: Orientation, o2: Orientation, isfirst: bool) -> Orientation {
    let c = top_abs::compose(o1, o2);
    if isfirst { top_abs::reverse(c) } else { c }
}

/// `CpInterf(DStr, FI)`: the interference with copies of its curve (added to the data
/// structure) and pcurves.
fn cp_interf(
    dstr: &mut TopOpeBRepDSDataStructure,
    fi: &FaceInterference,
) -> Result<FaceInterference, Failure> {
    let mut new_f = fi.clone();
    let (toc_curve, toc_tol) = {
        let toc = dstr.curve(fi.line_index());
        (toc.curve().clone(), toc.tolerance())
    };
    let new_c: Handle<dyn GeomCurve> = match toc_curve.get() {
        Some(c) => c.copy()?.cast(),
        None => Handle::default(),
    };
    new_f.set_line_index(dstr.add_curve(&mut TopOpeBRepDSCurve::new(&new_c, toc_tol, false)));
    if let Some(c) = fi.pcurve_on_face().get() {
        *new_f.change_pcurve_on_face() = c.copy()?.cast();
    }
    if let Some(c) = fi.pcurve_on_surf().get() {
        *new_f.change_pcurve_on_surf() = c.copy()?.cast();
    }
    Ok(new_f)
}

/// `CpSD(DStr, Data)`: a new surface data with a copy of the surface and the
/// interferences of `data`.
pub(super) fn cp_sd(
    dstr: &mut TopOpeBRepDSDataStructure,
    data: &SurfDataHandle,
) -> Result<SurfDataHandle, Failure> {
    let (surf, is1, is2, orientation, fi1, fi2) = {
        let d = lock(data);
        (
            d.surf(),
            d.index_of_s1(),
            d.index_of_s2(),
            d.orientation(),
            d.interference(1).clone(),
            d.interference(2).clone(),
        )
    };
    let (s, tol) = {
        let tos = dstr.surface(surf);
        (tos.surface().clone(), tos.tolerance())
    };
    let new_s: Handle<dyn GeomSurface> = s
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "CpSD"))?
        .copy()?
        .cast();
    let mut nd = SurfData::new();
    nd.change_surf(dstr.add_surface(&TopOpeBRepDSSurface::new(&new_s, tol)));
    nd.change_index_of_s1(is1);
    nd.change_index_of_s2(is2);
    *nd.change_orientation() = orientation;
    *nd.change_interference(1) = cp_interf(dstr, &fi1)?;
    *nd.change_interference(2) = cp_interf(dstr, &fi2)?;
    Ok(shared(nd))
}

/// `AdjustParam(Dom, f, l, wref, period, pitol)`: the range of the domain (whole period
/// for missing ends) in the period after `wref`; true when it wraps (then `f` is moved
/// back by a period).
pub(super) fn adjust_param(
    dom: &Domain,
    wref: f64,
    period: f64,
    pitol: f64,
) -> Result<(bool, f64, f64), Failure> {
    let mut f = if dom.has_first_point() {
        dom.first_point()?.parameter()
    } else {
        0.
    };
    let mut l = if dom.has_second_point() {
        dom.second_point()?.parameter()
    } else {
        period
    };
    if period == 0. {
        return Ok((false, f, l));
    }
    f = elclib::in_period(f, wref - pitol, wref + period - pitol);
    l = elclib::in_period(l, wref + pitol, wref + period + pitol);
    if l < f {
        f -= period;
        return Ok((true, f, l));
    }
    Ok((false, f, l))
}

/// `ComputeAbscissa(C, U)`: the abscissa of the parameter on a line or a circle (0
/// otherwise).
pub(super) fn compute_abscissa(c: &BRepAdaptorCurve, u: f64) -> Result<f64, Failure> {
    Ok(match c.get_type()? {
        CurveType::Line => u,
        CurveType::Circle => c.circle()?.radius() * u,
        _ => 0.,
    })
}

/// The abscissa on the elementary spine of the `iedge`-th edge of a parameter.
pub(super) fn abscissa_on_edge(spine: &SpineHandle, iedge: i32, u: f64) -> Result<f64, Failure> {
    let mut sp = lock(spine);
    compute_abscissa(sp.current_elementary_spine(iedge)?, u)
}

/// The edge index after `ii` in the sense, cyclic on a periodic spine.
fn next_index(ii: i32, nbed: i32, periodic: bool) -> i32 {
    if !periodic {
        return ii;
    }
    if ii <= 0 {
        ii + nbed
    } else if ii > nbed {
        ii - nbed
    } else {
        ii
    }
}

/// The search of `ParamOnSpine` along the neighbour edges: the abscissa where the plane
/// normal to the fillet curve cuts them.
#[allow(clippy::too_many_arguments)]
fn param_on_neighbours(
    dstr: &TopOpeBRepDSDataStructure,
    ptg: f64,
    cd: &SurfDataHandle,
    spine: &SpineHandle,
    iedge: i32,
    mut nl: f64,
    f: f64,
    tol: f64,
) -> Result<(f64, bool), Failure> {
    let c3d = {
        let g = lock(cd);
        let mut c = Handle::<dyn GeomCurve>::default();
        if g.interference(1).line_index() != 0 {
            c = dstr.curve(g.interference(1).line_index()).curve().clone();
        }
        if c.is_null() {
            c = dstr.curve(g.interference(2).line_index()).curve().clone();
        }
        c
    };
    let (pp, vv) = c3d
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "ParamOnSpine"))?
        .d1(ptg)?;
    let pln: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::from_point(pp, Direction3::from_vector(vv)?)?).cast();
    let plan: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&pln)?).cast();
    let sens = if nl <= f { -1 } else { 1 };
    let (nbed, periodic) = {
        let sp = lock(spine);
        (sp.nb_edges(), sp.is_periodic())
    };
    let mut ii = next_index(iedge + sens, nbed, periodic);
    if !periodic && (ii < 1 || ii > nbed) {
        return Ok((nl, true));
    }
    let (mut found, mut fini) = (false, false);
    while !found && !fini {
        let edge = lock(spine).edges(ii)?.clone();
        let o = edge.orientation();
        let first =
            (o == Orientation::Forward && sens == 1) || (o == Orientation::Reversed && sens == -1);
        let he = Handle::new(BRepAdaptorCurve::with_edge(&edge)?);
        let tolc = he.resolution(tol)?;
        let hec: Handle<dyn Adaptor3dCurve> = he.clone().cast();
        let w = inter_plane_edge(&plan, &hec, first, tolc)?;
        found = w.is_some();
        if let Some(w) = w {
            nl = w;
        }
        he.value(nl)?;
        if found {
            nl = lock(spine).absc_on_edge(nl, ii)?;
        }
        lock(spine).value(nl)?;
        ii = next_index(ii + sens, nbed, periodic);
        fini = if periodic {
            ii == iedge
        } else {
            ii < 1 || ii > nbed
        };
    }
    Ok((nl, found))
}

/// `ParamOnSpine(DStr, ptg, CD, Spine, iedge, intf, intl, tol, pok)`: the abscissa on the
/// spine of the parameter of the fillet on its edge, else of the cut of a neighbour edge
/// by the plane normal to the fillet curve; `(abscissa, pok)`.
#[allow(clippy::too_many_arguments)]
pub(super) fn param_on_spine(
    dstr: &TopOpeBRepDSDataStructure,
    ptg: f64,
    cd: &SurfDataHandle,
    spine: &SpineHandle,
    iedge: i32,
    (intf, intl): (bool, bool),
    tol: f64,
) -> Result<(f64, bool), Failure> {
    let (f, l) = {
        let sp = lock(spine);
        (sp.first_parameter_of(iedge)?, sp.last_parameter_of(iedge)?)
    };
    let nl = abscissa_on_edge(spine, iedge, ptg)? + f;
    if (nl >= f - tol || intf) && (nl <= l + tol || intl) {
        return Ok((nl, true));
    }
    param_on_neighbours(dstr, ptg, cd, spine, iedge, nl, f, tol)
}

/// `YaUnVoisin(Spine, iedge, ivois, isfirst)`: the neighbour edge before (after) the
/// `iedge`-th one, if any.
pub(super) fn ya_un_voisin(spine: &SpineHandle, iedge: i32, isfirst: bool) -> Option<i32> {
    let sp = lock(spine);
    let nbed = sp.nb_edges();
    if nbed == 1 {
        return None;
    }
    let mut ivois = if isfirst { iedge - 1 } else { iedge + 1 };
    if sp.is_periodic() {
        if ivois == 0 {
            ivois = nbed;
        }
        if ivois == nbed + 1 {
            ivois = 1;
        }
    }
    (ivois > 0 && ivois <= nbed).then_some(ivois)
}

/// The ends of a surface data reset to the parameter `par` on both sides (the points on
/// the surface), and its spine parameter at the end set to `sppar`.
fn reset_end(
    dstr: &TopOpeBRepDSDataStructure,
    cd: &SurfDataHandle,
    isfirst: bool,
    par: f64,
    sppar: f64,
) -> Result<(), Failure> {
    let mut g = lock(cd);
    let surf = dstr.surface(g.surf()).surface().clone();
    let s = surf
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "ResetProl"))?;
    for i in 1..=2 {
        g.change_interference(i).set_parameter(par, isfirst);
        let pc: Handle<dyn Geom2dCurve> = g.interference(i).pcurve_on_surf().clone();
        let xy = pc
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "ResetProl"))?
            .value(par)?;
        let cp = g.change_vertex(isfirst, i);
        cp.reset();
        cp.set_point(s.value(xy.x(), xy.y())?);
        if isfirst {
            g.set_first_spine_param(sppar);
        } else {
            g.set_last_spine_param(sppar);
        }
    }
    Ok(())
}

/// `ResetProl(DStr, CD, Spine, iedge, isfirst)`: the end of the particular case put back at
/// the end of its edge; the spine parameter there.
pub(super) fn reset_prol(
    dstr: &TopOpeBRepDSDataStructure,
    cd: &SurfDataHandle,
    spine: &SpineHandle,
    iedge: i32,
    isfirst: bool,
) -> Result<f64, Failure> {
    let (edglen, sppar) = {
        let mut sp = lock(spine);
        let bc = sp.current_elementary_spine(iedge)?;
        let edglen = bc.last_parameter()? - bc.first_parameter()?;
        let sppar = if isfirst {
            sp.first_parameter_of(iedge)?
        } else {
            sp.last_parameter_of(iedge)?
        };
        (edglen, sppar)
    };
    reset_end(dstr, cd, isfirst, if isfirst { 0. } else { edglen }, sppar)?;
    Ok(sppar)
}

impl ChFi3dBuilder {
    /// Whether the end of the particular case must be cut at the end of its edge: its
    /// tangent or its position differ from the spine's, or the neighbour edge has a
    /// particular case or a vertex of less than three edges.
    #[allow(clippy::too_many_arguments)]
    fn must_trunc(
        &self,
        sd: &SurfDataHandle,
        spine: &SpineHandle,
        (s1, s2): (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
        iedge: i32,
        isfirst: bool,
        wtg: f64,
    ) -> Result<bool, Failure> {
        let (wsp, _) =
            param_on_spine(&self.ds, wtg, sd, spine, iedge, (false, false), self.tolesp)?;
        let (bout1, ped, ded) = {
            let mut sp = lock(spine);
            let forward = sp.edges(iedge)?.orientation() == Orientation::Forward;
            let bc = sp.current_elementary_spine(iedge)?;
            let support = bc.edge();
            let (mut b1, mut b2) = topexp::vertices(&support, false);
            if support.orientation() == Orientation::Reversed {
                std::mem::swap(&mut b1, &mut b2);
            }
            let (edf, edl) = (bc.first_parameter()?, bc.last_parameter()?);
            let (ped, ded) = if forward {
                bc.d1(wtg + edf)?
            } else {
                let (p, mut d) = bc.d1(-wtg + edl)?;
                d.reverse();
                (p, d)
            };
            (if isfirst { b1 } else { b2 }, ped, ded)
        };
        let (psp, dsp) = lock(spine).d1(wsp)?;
        let (p1, p2) = {
            let g = lock(sd);
            let surf = self.ds.surface(g.surf()).surface().clone();
            let s = surf.get().ok_or_else(|| Failure::new("Trunc"))?;
            let at = |k: i32| -> Result<rustycad_tkmath::gp::Point3, Failure> {
                let pc = g.interference(k).pcurve_on_surf().clone();
                let uv = pc.get().ok_or_else(|| Failure::new("Trunc"))?.value(wtg)?;
                s.value(uv.x(), uv.y())
            };
            (at(1)?, at(2)?)
        };
        let mut tron = dsp.angle(ded)? > PI / 18. || psp.distance(&ped) >= 0.1 * p1.distance(&p2);
        if !tron && let Some(ivois) = ya_un_voisin(spine, iedge, isfirst) {
            let bs1: Handle<BRepAdaptorSurface> = s1.clone().cast();
            let bs2: Handle<BRepAdaptorSurface> = s2.clone().cast();
            if let (Some(b1), Some(b2)) = (bs1.get(), bs2.get()) {
                let (f1, f2) = (b1.face(), b2.face());
                let (cp1, cp2) = {
                    let g = lock(sd);
                    (g.vertex(isfirst, 1).clone(), g.vertex(isfirst, 2).clone())
                };
                let mut fbid = Face::default();
                if !((cp1.is_on_arc() && self.search_face(Some(spine), &cp1, &f1, &mut fbid)?)
                    || (cp2.is_on_arc() && self.search_face(Some(spine), &cp2, &f2, &mut fbid)?))
                {
                    tron = kparticular(spine, ivois, b1, b2)?;
                }
            }
        }
        if !tron && ya_un_voisin(spine, iedge, isfirst).is_some() {
            let nbed = self.my_ve_map.find_from_key(bout1.as_ref())?.iter().count() as i32 - 2;
            if nbed < 3 {
                tron = true;
            }
        }
        Ok(tron)
    }

    /// `Trunc(SD, Spine, S1, S2, iedge, isfirst, cntlFiOnS)`: the end of the particular
    /// case cut at the end of its edge when it must be (not beyond the end of the
    /// interference `cntl_fi_on_s` when given).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn trunc(
        &self,
        sd: &SurfDataHandle,
        spine: &SpineHandle,
        faces: (&Handle<dyn Adaptor3dSurface>, &Handle<dyn Adaptor3dSurface>),
        iedge: i32,
        isfirst: bool,
        cntl_fi_on_s: i32,
    ) -> Result<(), Failure> {
        let wtg = lock(sd).interference(1).parameter(isfirst);
        if !self.must_trunc(sd, spine, faces, iedge, isfirst, wtg)? {
            return Ok(());
        }
        let edglen = {
            let mut sp = lock(spine);
            let bc = sp.current_elementary_spine(iedge)?;
            bc.last_parameter()? - bc.first_parameter()?
        };
        let mut par = if isfirst { 0. } else { edglen };
        let mut dpar = 0.;
        if cntl_fi_on_s != 0 {
            let fi_par = lock(sd).interference(cntl_fi_on_s).parameter(!isfirst);
            let is_the_case = if isfirst { par > fi_par } else { par < fi_par };
            if is_the_case {
                dpar = par - fi_par;
                par = fi_par;
            }
        }
        let sppar = {
            let sp = lock(spine);
            if isfirst {
                sp.first_parameter_of(iedge)? - dpar
            } else {
                sp.last_parameter_of(iedge)? - dpar
            }
        };
        reset_end(&self.ds, sd, isfirst, par, sppar)
    }
}
