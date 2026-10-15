// Rust translation of the file statics CompCommonPoint, Tri, FillSD,
// CoincidentDomainEdge and SetCoincidentDomainEdge, and of the hatchings of
// ChFi3d_Builder::SplitKPart (ChFi3d_Builder_SpKP.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The hatching of the traces of a particular case by the faces, and the surface data
//! cut to each domain of the hatching.
use super::trunc::{adjust_param, comp_tra};
use crate::brep_blend::WalkingDomain;
use crate::chfi_ds::{CommonPoint, SurfDataHandle, lock};
use crate::chfi3d::compute_hatching_domains;
use rustycad_tkbool::top_ope_brep_ds::TopOpeBRepDSDataStructure;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkgeomalgo::geom2d_hatch::{Hatcher, Intersector};
use rustycad_tkgeomalgo::hatch_gen::{Domain, PointOnElement, PointOnHatching};
use std::collections::HashMap;

/// The elements of a hatcher by their index (`M1`, `M2`).
pub(super) type Elements = HashMap<i32, Handle<dyn Adaptor2dCurve2d>>;

/// The hatching of the trace of the fillet on one face: the hatcher, the index of the
/// hatching, its number of domains, its elements, the trace and its adaptor.
pub(super) struct SideHatch {
    pub h: Hatcher,
    pub ih: i32,
    pub nb: i32,
    pub m: Elements,
    pub ll: Geom2dAdaptorCurve,
}

impl SideHatch {
    /// The hatching of the trace `c` by the arcs of the domain `it`; `None` when its
    /// domains cannot be computed or there is none.
    pub(super) fn build(
        c: &Handle<dyn Geom2dCurve>,
        it: &WalkingDomain,
        (tol2d, tolapp3d): (f64, f64),
    ) -> Result<Option<Self>, Failure> {
        let pitol = precision::p_intersection();
        let inter = Intersector::new(pitol, pitol);
        let mut side = SideHatch {
            h: Hatcher::new(&inter, tol2d, tolapp3d, false, false),
            ih: 0,
            nb: 1,
            m: HashMap::new(),
            ll: Geom2dAdaptorCurve::default(),
        };
        if c.is_null() {
            return Ok(Some(side));
        }
        side.ll = Geom2dAdaptorCurve::with_curve(c)?;
        it.init();
        while it.more() {
            let v = it.value()?;
            let bc: Handle<BRepAdaptorCurve2d> = v.clone().cast();
            let ie = match bc.get() {
                Some(b) => side.h.add_element(
                    b.geom2d_adaptor().shallow_adaptor()?,
                    b.edge().orientation(),
                ),
                None => {
                    let gc: Handle<Geom2dAdaptorCurve> = v.clone().cast();
                    let g = gc.get().ok_or_else(|| {
                        Failure::with_message("Standard_NullObject", "SplitKPart: arc")
                    })?;
                    side.h
                        .add_element(g.shallow_adaptor()?, Orientation::Forward)
                }
            };
            side.m.insert(ie, v);
            it.next();
        }
        side.ih = side.h.trim_curve(side.ll.shallow_adaptor()?)?;
        if !compute_hatching_domains(&mut side.h, side.ih)? {
            return Ok(None);
        }
        side.nb = side.h.nb_domains(side.ih)?;
        if side.nb == 0 {
            return Ok(None);
        }
        Ok(Some(side))
    }

    /// The `i`-th domain of the hatching, copied.
    pub(super) fn domain(&self, i: i32) -> Result<Domain, Failure> {
        Ok(self.h.domain(self.ih, i)?.clone())
    }
}

/// `CompCommonPoint(FilPoint, arc, PE, Or)`: the common point at the vertex of the arc
/// given by the position of the point on it.
fn comp_common_point(
    fil_point: &mut CommonPoint,
    arc: &Edge,
    pe: &PointOnElement,
    or: Orientation,
) {
    let v = if pe.position() == Orientation::Forward {
        topexp::first_vertex(arc, false)
    } else {
        topexp::last_vertex(arc, false)
    };
    fil_point.set_vertex(&v);
    fil_point.set_arc(
        precision::p_intersection(),
        arc,
        pe.parameter(),
        top_abs::compose(arc.orientation(), or),
    );
}

/// `Tri(H, iH, Ind, wref, period, pitol, Nbdom)`: the domains sorted by their adjusted
/// first parameter into `ind`; a domain without first point merged with the one without
/// last point (false when there is none).
pub(super) fn tri(
    side: &mut SideHatch,
    ind: &mut Vec<i32>,
    (wref, period, pitol): (f64, f64, f64),
) -> Result<bool, Failure> {
    *ind = (1..=side.nb).collect();
    let mut invert = true;
    while invert {
        invert = false;
        for i in 0..(side.nb as usize).saturating_sub(1) {
            let (_, f1, _) = adjust_param(&side.domain(ind[i])?, wref, period, pitol)?;
            let (_, f2, _) = adjust_param(&side.domain(ind[i + 1])?, wref, period, pitol)?;
            if f2 < f1 {
                ind.swap(i, i + 1);
                invert = true;
            }
        }
    }
    let (mut i_sans_first, mut i_sans_last) = (0usize, 0usize);
    if side.nb != 1 {
        for i in 1..=side.nb as usize {
            let d = side.domain(ind[i - 1])?;
            if !d.has_first_point() {
                i_sans_first = i;
            }
            if !d.has_second_point() {
                i_sans_last = i;
            }
        }
    }
    if i_sans_first != 0 {
        if i_sans_last == 0 {
            return Ok(false);
        }
        let last_dom = side.domain(ind[i_sans_last - 1])?;
        let mut ph: PointOnHatching = last_dom.first_point()?.clone();
        let new_par = side.h.hatching_curve(side.ih)?.first_parameter()? - period
            + last_dom.first_point()?.parameter();
        ph.set_parameter(new_par);
        let hatching = side.h.hatching_mut(side.ih)?;
        hatching
            .change_domain(ind[i_sans_last - 1])?
            .set_first_point(&ph);
        hatching
            .change_domain(ind[i_sans_first - 1])?
            .set_first_point(&ph);
        ind.remove(i_sans_last - 1);
        side.nb -= 1;
    }
    Ok(true)
}

/// The index of the point on element of a hatching point that lies on an arc through
/// `bout` (the first one otherwise, the second when the first does not).
fn point_type(pph: &PointOnHatching, m: &Elements, bout: &Vertex) -> Result<i32, Failure> {
    let mut le_type = 1;
    let nb_int = pph.nb_points();
    if nb_int <= 1 {
        return Ok(le_type);
    }
    let mut trouve = true;
    let (mut v1, mut v2) = (Vertex::default(), Vertex::default());
    let mut suite = true;
    while trouve {
        let ie = pph.point(le_type)?.index();
        let he: Handle<BRepAdaptorCurve2d> = element(m, ie)?.cast();
        match he.get() {
            Some(h) => (v1, v2) = topexp::vertices(&h.edge(), false),
            None => suite = false,
        }
        if (v1.is_same(bout) || v2.is_same(bout)) && suite {
            break;
        }
        suite = true;
        trouve = false;
        le_type += 1;
        if le_type > nb_int {
            trouve = true;
            le_type = 1;
        }
    }
    Ok(le_type)
}

/// `M(IE)`: the element of index `ie` (Standard_NoSuchObject when unbound).
fn element(m: &Elements, ie: i32) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
    m.get(&ie)
        .cloned()
        .ok_or_else(|| Failure::with_message("Standard_NoSuchObject", "NCollection_DataMap::Find"))
}

/// The end of the side `ons` at `ponh`: the parameter of the interference and the point
/// of the common point on the surface.
fn set_end_point(
    dstr: &TopOpeBRepDSDataStructure,
    cd: &SurfDataHandle,
    ons: i32,
    ponh: f64,
    is_first: bool,
) -> Result<(), Failure> {
    let mut g = lock(cd);
    let surf = dstr.surface(g.surf()).surface().clone();
    let pc = g.interference(ons).pcurve_on_surf().clone();
    let xy = pc
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "FillSD"))?
        .value(ponh)?;
    g.change_interference(ons).set_parameter(ponh, is_first);
    let p = surf
        .get()
        .ok_or_else(|| Failure::with_message("Standard_NullObject", "FillSD"))?
        .value(xy.x(), xy.y())?;
    g.change_vertex(is_first, ons).set_point(p);
    Ok(())
}

/// `FillSD(DStr, CD, M, Dom, ponH, isFirst, ons, pitol, bout)`: the end of the surface
/// data on the domain of the hatching of the side `ons` at `ponh`: on the arc of the end
/// point of the domain (on its vertex `bout` when it ends there), and the other side cut
/// there unless it ends on an arc.
#[allow(clippy::too_many_arguments)]
pub(super) fn fill_sd(
    dstr: &TopOpeBRepDSDataStructure,
    cd: &SurfDataHandle,
    m: &Elements,
    dom: &Domain,
    ponh: f64,
    is_first: bool,
    ons: i32,
    pitol: f64,
    bout: &Vertex,
) -> Result<(), Failure> {
    let opp = 3 - ons;
    let pph = if is_first && dom.has_first_point() {
        Some(dom.first_point()?.clone())
    } else if !is_first && dom.has_second_point() {
        Some(dom.second_point()?.clone())
    } else {
        None
    };
    match pph {
        None => set_end_point(dstr, cd, ons, ponh, is_first)?,
        Some(pph) => {
            let le_type = point_type(&pph, m, bout)?;
            let pe = *pph.point(le_type)?;
            let he: Handle<BRepAdaptorCurve2d> = element(m, pe.index())?.cast();
            let Some(h) = he.get() else {
                return Ok(());
            };
            let e = h.edge();
            {
                let mut g = lock(cd);
                let tr = g.interference(ons).transition();
                if pe.position() != Orientation::Internal {
                    let o = if is_first { top_abs::reverse(tr) } else { tr };
                    comp_common_point(g.change_vertex(is_first, ons), &e, &pe, o);
                } else {
                    g.change_vertex(is_first, ons).set_arc(
                        pitol,
                        &e,
                        pe.parameter(),
                        comp_tra(tr, e.orientation(), is_first),
                    );
                }
            }
            set_end_point(dstr, cd, ons, ponh, is_first)?;
        }
    }
    if !lock(cd).vertex(is_first, opp).is_on_arc() {
        set_end_point(dstr, cd, opp, ponh, is_first)?;
    }
    Ok(())
}

/// `CoincidentDomainEdge(theElements, theDomain)`: the edge whose ends bound the domain
/// (and whether it runs against the hatching), if any.
fn coincident_domain_edge(
    elements: &Elements,
    dom: &Domain,
) -> Result<Option<(Edge, bool)>, Failure> {
    if !dom.has_first_point() || !dom.has_second_point() {
        return Ok(None);
    }
    let (a_first, a_second) = (dom.first_point()?, dom.second_point()?);
    if !a_first.segment_beginning() || !a_second.segment_end() {
        return Ok(None);
    }
    for i in 1..=a_first.nb_points() {
        let fe = a_first.point(i)?;
        if fe.position() == Orientation::Internal {
            continue;
        }
        for j in 1..=a_second.nb_points() {
            let se = a_second.point(j)?;
            if fe.index() != se.index()
                || se.position() == Orientation::Internal
                || fe.position() == se.position()
            {
                continue;
            }
            let Some(el) = elements.get(&fe.index()) else {
                continue;
            };
            let an_element: Handle<BRepAdaptorCurve2d> = el.clone().cast();
            if let Some(e) = an_element.get() {
                let hatching_delta = a_second.parameter() - a_first.parameter();
                let element_delta = se.parameter() - fe.parameter();
                return Ok(Some((e.edge(), hatching_delta * element_delta < 0.0)));
            }
        }
    }
    Ok(None)
}

/// `SetCoincidentDomainEdge(theDS, theInterference, theElements, theDomain,
/// theWholeDomain)`: the edge bounding the whole domain given to the curve of the
/// interference.
pub(super) fn set_coincident_domain_edge(
    dstr: &mut TopOpeBRepDSDataStructure,
    cd: &SurfDataHandle,
    ons: i32,
    elements: &Elements,
    dom: &Domain,
    whole: bool,
) -> Result<(), Failure> {
    if !whole {
        return Ok(());
    }
    if let Some((e, rev)) = coincident_domain_edge(elements, dom)? {
        let li = lock(cd).interference(ons).line_index();
        dstr.change_curve(li).set_existing_edge(&e, rev);
    }
    Ok(())
}
