// Rust translation of FilletSurf_InternalBuilder::Add and the file statics isinlist,
// IntPlanEdge and ComputeEdgeParameter (FilletSurf_InternalBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The contour of the fillet surfaces: the given edges checked, the contour of the
//! first one reduced to them, and its elementary spine built at once; and the parameters
//! of the ends of the fillet on the edges.
use super::internal_builder::FilletSurfInternalBuilder;
use crate::chfi_ds::{ElSpine, FilSpine, SpineHandle, SpineObject, lock, shared};
use crate::chfi3d::perform_el_spine;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::int_curve_surface::HInter;
use rustycad_tkmath::elslib;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Direction3, Plane3};

/// `isinlist(E, L)`: the edge is in the list.
fn isinlist(e: &Edge, l: &[Shape]) -> bool {
    l.iter().any(|s| e.is_same(s))
}

/// The parameter on the edge of the point nearest to the origin of the plane among its
/// intersections with the plane and its ends on the plane (within `tol3d`);
/// `IntPlanEdge(Ed, P, w, tol3d)`.
fn int_plan_edge(
    ed: &Handle<dyn Adaptor3dCurve>,
    p: &Plane3,
    tol3d: f64,
) -> Result<Option<f64>, Failure> {
    let (f, l) = (ed.first_parameter()?, ed.last_parameter()?);
    let or = p.location();
    let plan: Handle<dyn Adaptor3dSurface> = Handle::new(GeomAdaptorSurface::with_surface(
        &Handle::new(GeomPlane::from_pln(*p)).cast(),
    )?)
    .cast();
    let mut intersection = HInter::new();
    intersection.perform(ed, &plan)?;
    let (mut w, mut dist) = (None, f64::MAX);
    if intersection.is_done() {
        for iip in 1..=intersection.nb_points()? {
            let ip = intersection.point(iip)?;
            let d = ip.pnt().distance(&or);
            if d < dist {
                (w, dist) = (Some(ip.w()), d);
            }
        }
    }
    for u in [f, l] {
        let pt = ed.value(u)?;
        let (pu, pv) = elslib::parameters(p, pt);
        if pt.distance(&elslib::value(pu, pv, p)) < tol3d {
            let d = pt.distance(&or);
            if d < dist {
                (w, dist) = (Some(u), d);
            }
        }
    }
    Ok(w)
}

/// `ComputeEdgeParameter(Spine, ind, pelsp, ped, tol3d)`: the parameter on the `ind`-th
/// edge of the spine of its crossing with the normal plane of the elementary spine at
/// `pelsp`.
pub(super) fn compute_edge_parameter(
    spine: &SpineHandle,
    ind: i32,
    pelsp: f64,
    tol3d: f64,
) -> Result<Option<f64>, Failure> {
    let (guide, edge) = {
        let mut g = lock(spine);
        let guide = g.el_spine_of_index(ind)?;
        g.current_elementary_spine(ind)?;
        (guide, g.edges(ind)?.clone())
    };
    let (p, v) = guide.d1(pelsp)?;
    let pln = Plane3::new(p, Direction3::from_vector(v)?)?;
    let ed: Handle<dyn Adaptor3dCurve> = Handle::new(BRepAdaptorCurve::with_edge(&edge)?).cast();
    int_plan_edge(&ed, &pln, tol3d)
}

/// The two faces of a sharp edge: the first face of its list and the next different one,
/// walking the list with the iterator of the edges as natively (`pos` is left on the
/// second face).
fn edge_faces(list: &[Shape], pos: &mut usize) -> Result<(Face, Face), Failure> {
    let (mut ff1, mut ff2) = (Face::default(), Face::default());
    *pos = 0;
    while *pos < list.len() {
        if ff1.is_null() {
            ff1 = topods::face(&list[*pos])?;
        } else {
            ff2 = topods::face(&list[*pos])?;
            if !ff2.is_same(&ff1) {
                break;
            }
        }
        *pos += 1;
    }
    Ok((ff1, ff2))
}

impl FilletSurfInternalBuilder {
    /// The check of the edges of `Add`: 4 for an edge not on the shape, 5 for an edge
    /// that is not sharp, 0 otherwise. The native code walks the faces of the first edge
    /// with the iterator of the list, so that only that edge is checked on a manifold
    /// shape (the walk goes on in the list of its faces).
    fn check_edges(&self, e: &[Shape]) -> Result<i32, Failure> {
        let mut list: Vec<Shape> = e.to_vec();
        let mut pos = 0usize;
        while pos < list.len() {
            let cured = topods::edge(&list[pos])?;
            if cured.is_null() || !self.fil.base.my_ef_map.contains(cured.as_ref()) {
                return Ok(4);
            }
            list = self
                .fil
                .base
                .my_ef_map
                .find_from_key(cured.as_ref())?
                .iter()
                .cloned()
                .collect();
            let (ff1, ff2) = edge_faces(&list, &mut pos)?;
            if ff1.is_null() || ff2.is_null() || ff1.is_same(&ff2) {
                return Ok(5);
            }
            if BRepTool::continuity_on_faces(&cured, &ff1, &ff2)? != Continuity::C0 {
                return Ok(5);
            }
            pos += 1;
        }
        Ok(0)
    }

    /// `Add(E, R)`: the contour of the edges `e` with the radius `r`: 0 when done, 1 for
    /// an empty list, 2 for edges not G1 or not continuous in the contour, 4 for an edge
    /// not on the shape, 5 for an edge that is not sharp.
    pub fn add(&mut self, e: &[Shape], r: f64) -> Result<i32, Failure> {
        if e.is_empty() {
            return Ok(1);
        }
        let check = self.check_edges(e)?;
        if check != 0 {
            return Ok(check);
        }
        let mut ed = topods::edge(&e[0])?;
        ed.set_orientation(Orientation::Forward);
        self.fil.add_with_radius(r, &ed)?;
        let st = self
            .fil
            .base
            .my_list_stripe
            .first()
            .cloned()
            .ok_or_else(|| {
                Failure::with_message("Standard_NoSuchObject", "FilletSurf_InternalBuilder::Add")
            })?;
        let sp = lock(&st)
            .spine()
            .cloned()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "Add: no spine"))?;
        for s in e {
            if self.fil.base.contains(&topods::edge(s)?)? == 0 {
                return Ok(2);
            }
        }
        let Some(newsp) = reduce_spine(&sp, e, r)? else {
            return Ok(2);
        };
        let sp = match newsp {
            Some(n) => {
                *lock(&st).change_spine() = Some(n.clone());
                n
            }
            None => sp,
        };
        self.el_spine_at_once(&sp)?;
        Ok(0)
    }

    /// The elementary spine of the whole contour built at once.
    fn el_spine_at_once(&self, sp: &SpineHandle) -> Result<(), Failure> {
        let hels = Handle::new(ElSpine::new());
        let (first, last) = {
            let g = lock(sp);
            (g.first_parameter(), g.last_parameter()?)
        };
        let (pfirst, tfirst) = lock(sp).d1(first)?;
        let (plast, tlast) = lock(sp).d1(last)?;
        hels.set_first_parameter(first);
        hels.set_first_point_and_tgt(pfirst, tfirst);
        hels.set_last_parameter(last);
        hels.set_last_point_and_tgt(plast, tlast);
        perform_el_spine(
            &hels,
            sp,
            self.fil.base.my_conti,
            self.fil.base.tolesp,
            false,
        )?;
        // The virtual AppendElSpine: the fillet spine computes the law of the new spine.
        SpineObject::append_el_spine(&mut *lock(sp), &hels)?;
        lock(sp).set_split_done(true);
        Ok(())
    }
}

/// The contour reduced to the edges of the list (None when they are not consecutive in
/// it), the new spine when some edges were removed (`Some(None)` when none were).
fn reduce_spine(
    sp: &SpineHandle,
    e: &[Shape],
    r: f64,
) -> Result<Option<Option<SpineHandle>>, Failure> {
    let periodic = lock(sp).is_periodic();
    let nb = lock(sp).nb_edges();
    let mut newsp = FilSpine::new();
    let (mut debut, mut premierquinyestpas, mut yatrou) = (false, 0, 0);
    for i in 1..=nb {
        let cured = lock(sp).edges(i)?.clone();
        if isinlist(&cured, e) {
            debut = true;
            if premierquinyestpas != 0 {
                yatrou = 1;
                break;
            }
            newsp.set_edges(&cured);
        } else if debut && premierquinyestpas == 0 {
            premierquinyestpas = i;
        }
    }
    if yatrou != 0 && (!periodic || !wrap_around(sp, e, &mut newsp, yatrou)?) {
        return Ok(None);
    }
    if newsp.nb_edges() == nb {
        return Ok(Some(None));
    }
    newsp.load()?;
    newsp.set_radius(r)?;
    let h: SpineHandle = shared(newsp);
    Ok(Some(Some(h)))
}

/// On a periodic contour, the edges of the list before the hole put first, walking back
/// from the last edge down to `yatrou` (the native bound); false on a true hole.
fn wrap_around(
    sp: &SpineHandle,
    e: &[Shape],
    newsp: &mut FilSpine,
    yatrou: i32,
) -> Result<bool, Failure> {
    let (mut vraitrou, mut a_local_debut) = (false, false);
    let nb = lock(sp).nb_edges();
    for i in ((yatrou + 1)..=nb).rev() {
        let cured = lock(sp).edges(i)?.clone();
        if isinlist(&cured, e) {
            if vraitrou {
                return Ok(false);
            }
            newsp.put_in_first(&cured);
        } else if a_local_debut {
            vraitrou = true;
        }
        a_local_debut = true;
    }
    Ok(true)
}
