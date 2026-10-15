// Rust translation of the contour members of ChFi3d_ChBuilder and the file statics
// SearchCommonFaces (ChFi3d_ChBuilder.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The contours of the chamfer builder and their distances: one, two (on a reference
//! face) or a distance and an angle; and the simulation of a contour.
use super::ChFi3dChBuilder;
use crate::chfi_ds::{
    ChamfMethod, ChamfMode, ChamfSpine, CircSection, Map, SpineHandle, Stripe, lock, shared,
};
use crate::chfi3d::builder::ChFi3dBuilderOps;
use crate::chfi3d::concave_side;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{self, Edge, Face};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;

/// `SearchCommonFaces(EFMap, E, F1, F2)`: the two faces of the edge (the same face twice
/// for a seam).
pub(super) fn search_common_faces(ef_map: &Map, e: &Edge) -> Result<(Face, Face), Failure> {
    let (mut f1, mut f2) = (Face::default(), Face::default());
    for f in ef_map.find_from_key(e.as_ref())?.iter() {
        let fc = topods::face(f)?;
        if f1.is_null() {
            f1 = fc;
        } else if !fc.is_same(&f1) {
            f2 = fc;
            break;
        }
    }
    if !f1.is_null() && f2.is_null() && brep_tools::is_really_closed(e, &f1)? {
        f2 = f1.clone();
    }
    Ok((f1, f2))
}

/// The choice of the concave side of the faces of the edge.
fn concave_choice(f1: &Face, f2: &Face, e: &Edge) -> Result<i32, Failure> {
    let (sb1, sb2) = (
        BRepAdaptorSurface::with_face(f1, true)?,
        BRepAdaptorSurface::with_face(f2, true)?,
    );
    Ok(concave_side(&sb1, &sb2, e)?.0)
}

/// Standard_DomainError: the face is not along the contour.
fn face_not_common(what: &str) -> Failure {
    Failure::with_message("Standard_DomainError", what)
}

impl ChFi3dChBuilder {
    /// The chamfer spine of the contour `ic` (Standard_ConstructionError when it is not
    /// one), edited by `f`.
    fn with_chamf_spine<R>(
        &self,
        ic: i32,
        f: impl FnOnce(&mut ChamfSpine) -> Result<R, Failure>,
    ) -> Result<R, Failure> {
        let sp = self.spine_of(ic)?;
        let mut g = lock(&sp);
        let csp = g.as_chamf_spine_mut().ok_or_else(|| {
            Failure::with_message(
                "Standard_ConstructionError",
                "ChFi3d_ChBuilder: not a chamfer spine",
            )
        })?;
        f(csp)
    }

    /// The spine of the contour `ic` (Standard_NullObject without one).
    fn spine_of(&self, ic: i32) -> Result<SpineHandle, Failure> {
        self.base.value(ic)?.ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ChFi3d_ChBuilder: contour without spine",
            )
        })
    }

    /// A new contour on the chamfer spine through `e` (forward) when `e` is an edge of
    /// the shape not yet in a contour; the contour built and kept when `PerformElement`
    /// succeeds, `edit` applied to its spine first.
    fn new_contour(
        &mut self,
        e: &Edge,
        offset: f64,
        first_face: &Face,
        edit: impl FnOnce(&mut ChamfSpine),
        mode: bool,
    ) -> Result<(), Failure> {
        if self.base.contains(e)? != 0 || !self.base.my_ef_map.contains(e.as_ref()) {
            return Ok(());
        }
        let csp = ChamfSpine::with_tolerance(self.base.tolesp);
        let spine: SpineHandle = shared(csp);
        let mut stripe = Stripe::new();
        *stripe.change_spine() = Some(spine.clone());
        let mut e_wnt = e.clone();
        e_wnt.set_orientation(Orientation::Forward);
        {
            let mut g = lock(&spine);
            if mode && let Some(c) = g.as_chamf_spine_mut() {
                c.set_chamfer_mode(self.my_mode);
            }
            g.set_edges(&e_wnt);
        }
        if !self.base.perform_element(&spine, offset, first_face)? {
            return Ok(());
        }
        lock(&spine).load()?;
        self.base.my_list_stripe.push(shared(stripe));
        if let Some(c) = lock(&spine).as_chamf_spine_mut() {
            edit(c);
        }
        self.base.perform_extremity(&spine)
    }

    /// `Add(E)`: a contour of the tangent edges through `e`; `b.add(&edge)?`.
    pub fn add(&mut self, e: &Edge) -> Result<(), Failure> {
        if self.base.contains(e)? != 0 || !self.base.my_ef_map.contains(e.as_ref()) {
            return Ok(());
        }
        let spine: SpineHandle = shared(ChamfSpine::with_tolerance(self.base.tolesp));
        let mut stripe = Stripe::new();
        *stripe.change_spine() = Some(spine.clone());
        let mut e_wnt = e.clone();
        e_wnt.set_orientation(Orientation::Forward);
        lock(&spine).set_edges(&e_wnt);
        if self.base.perform_element(&spine, -1., &Face::default())? {
            self.base.perform_extremity(&spine)?;
            lock(&spine).load()?;
            self.base.my_list_stripe.push(shared(stripe));
        }
        Ok(())
    }

    /// `Add(Dis, E)`: a contour through `e` with the symmetric distance `dis`;
    /// `b.add_with_dist(2.0, &edge)?`.
    pub fn add_with_dist(&mut self, dis: f64, e: &Edge) -> Result<(), Failure> {
        self.new_contour(e, -1., &Face::default(), |c| c.set_dist(dis), true)
    }

    /// `Add(Dis1, Dis2, E, F)`: a contour through `e` with the distance `dis1` on the face
    /// `f` and `dis2` on the other; `b.add_with_dists(1.0, 3.0, &edge, &face)?`.
    pub fn add_with_dists(
        &mut self,
        dis1: f64,
        dis2: f64,
        e: &Edge,
        f: &Face,
    ) -> Result<(), Failure> {
        let offset = if self.my_mode == ChamfMode::ConstThroatWithPenetrationChamfer {
            dis1.min(dis2)
        } else {
            -1.
        };
        self.new_contour(e, offset, f, |c| c.set_dists(dis1, dis2), true)
    }

    /// `AddDA(Dis1, Angle, E, F)`: a contour through `e` with the distance `dis1` on the
    /// face `f` and the angle `angle`; `b.add_dist_angle(1.0, 0.5, &edge, &face)?`.
    pub fn add_dist_angle(
        &mut self,
        dis1: f64,
        angle: f64,
        e: &Edge,
        f: &Face,
    ) -> Result<(), Failure> {
        self.new_contour(e, -1., f, |c| c.set_dist_angle(dis1, angle), false)
    }

    /// The faces of the first edge of the contour, and of its first edge having the
    /// face `f` (`f` first); Standard_DomainError when no edge has it.
    fn faces_along(
        &self,
        ic: i32,
        f: &Face,
        what: &str,
    ) -> Result<((Face, Face), (Face, Face), Edge), Failure> {
        let sp = self.spine_of(ic)?;
        let g = lock(&sp);
        let mut first = (Face::default(), Face::default());
        for i in 1..=g.nb_edges() {
            let e = g.edges(i)?.clone();
            let (f1, f2) = search_common_faces(&self.base.my_ef_map, &e)?;
            if i == 1 {
                first = (f1.clone(), f2.clone());
            }
            if f1.is_same(f) || f2.is_same(f) {
                let faces = if f2.is_same(f) {
                    (f.clone(), f1)
                } else {
                    (f1, f2)
                };
                return Ok((faces, first, e));
            }
        }
        Err(face_not_common(what))
    }

    /// `SetDist(Dis, IC, F)`: the symmetric distance of the contour `ic`, which must run
    /// along `f`; `b.set_dist(2.0, 1, &face)?`.
    pub fn set_dist(&mut self, dis: f64, ic: i32, f: &Face) -> Result<(), Failure> {
        if ic > self.base.nb_elements() {
            return Ok(());
        }
        self.faces_along(
            ic,
            f,
            "the face is not common to any of edges of the contour",
        )?;
        self.with_chamf_spine(ic, |c| {
            c.set_dist(dis);
            Ok(())
        })
    }

    /// `GetDist(IC, Dis)`; `b.get_dist(1)?`.
    pub fn get_dist(&self, ic: i32) -> Result<f64, Failure> {
        self.with_chamf_spine(ic, |c| c.get_dist())
    }

    /// `SetDists(Dis1, Dis2, IC, F)`: the distances of the contour `ic`, `dis1` on the face
    /// `f` (swapped when the concave side differs from the first edge's);
    /// `b.set_dists(1.0, 2.0, 1, &face)?`.
    pub fn set_dists(&mut self, dis1: f64, dis2: f64, ic: i32, f: &Face) -> Result<(), Failure> {
        if ic > self.base.nb_elements() {
            return Ok(());
        }
        let ((f1, f2), (ff1, ff2), e) = self.faces_along(
            ic,
            f,
            "the face is not common to any of edges of the contour",
        )?;
        let choix = concave_choice(&f1, &f2, &e)?;
        let e1 = lock(&self.spine_of(ic)?).edges(1)?.clone();
        let choix_conge = concave_choice(&ff1, &ff2, &e1)?;
        self.with_chamf_spine(ic, |c| {
            if choix_conge % 2 != choix % 2 {
                c.set_dists(dis2, dis1);
            } else {
                c.set_dists(dis1, dis2);
            }
            Ok(())
        })
    }

    /// `Dists(IC, Dis1, Dis2)`; `b.dists(1)?`.
    pub fn dists(&self, ic: i32) -> Result<(f64, f64), Failure> {
        self.with_chamf_spine(ic, |c| c.dists())
    }

    /// `SetDistAngle(Dis, Angle, IC, F)`: the distance and angle of the contour `ic`,
    /// which must run along `f`; `b.set_dist_angle(1.0, 0.5, 1, &face)?`.
    pub fn set_dist_angle(
        &mut self,
        dis: f64,
        angle: f64,
        ic: i32,
        f: &Face,
    ) -> Result<(), Failure> {
        if ic > self.base.nb_elements() {
            return Ok(());
        }
        self.faces_along(ic, f, "the face is not common to any edges of the contour")?;
        self.with_chamf_spine(ic, |c| {
            c.set_dist_angle(dis, angle);
            Ok(())
        })
    }

    /// `GetDistAngle(IC, Dis, Angle)`; `b.get_dist_angle(1)?`.
    pub fn get_dist_angle(&self, ic: i32) -> Result<(f64, f64), Failure> {
        self.with_chamf_spine(ic, |c| c.get_dist_angle())
    }

    /// `SetMode(theMode)`: the mode of the chamfers added next.
    pub fn set_mode(&mut self, mode: ChamfMode) {
        self.my_mode = mode;
    }

    /// `Mode()`.
    pub fn mode(&self) -> ChamfMode {
        self.my_mode
    }

    /// `IsChamfer(IC)`: the method of the contour; `b.is_chamfer(1)?`.
    pub fn is_chamfer(&self, ic: i32) -> Result<ChamfMethod, Failure> {
        self.with_chamf_spine(ic, |c| Ok(c.is_chamfer()))
    }

    /// `ResetContour(IC)`; `b.reset_contour(1)?`.
    pub fn reset_contour(&mut self, ic: i32) -> Result<(), Failure> {
        if ic > self.base.nb_elements() {
            return Ok(());
        }
        self.with_chamf_spine(ic, |c| crate::chfi_ds::SpineObject::reset(c, true))
    }

    /// `Simulate(IC)`; `b.simulate(1)?`.
    pub fn simulate(&mut self, ic: i32) -> Result<(), Failure> {
        let Some(st) = usize::try_from(ic - 1)
            .ok()
            .and_then(|k| self.base.my_list_stripe.get(k))
            .cloned()
        else {
            return Ok(());
        };
        (self as &mut dyn ChFi3dBuilderOps).perform_set_of_surf(&st, true)
    }

    /// `NbSurf(IC)`; `b.nb_surf(1)`.
    pub fn nb_surf(&self, ic: i32) -> i32 {
        let Some(st) = usize::try_from(ic - 1)
            .ok()
            .and_then(|k| self.base.my_list_stripe.get(k))
        else {
            return 0;
        };
        lock(st)
            .set_of_surf_data()
            .map(|s| lock(s).len() as i32)
            .unwrap_or(0)
    }

    /// `Sect(IC, IS)`; `b.sect(1, 1)?`.
    pub fn sect(&self, ic: i32, is: i32) -> Result<Handle<HArray1<CircSection>>, Failure> {
        let Some(st) = usize::try_from(ic - 1)
            .ok()
            .and_then(|k| self.base.my_list_stripe.get(k))
        else {
            return Ok(Handle::default());
        };
        let sd = crate::chfi3d::surf_data_at(st, is)?;
        let simul = lock(&sd).simul();
        Ok(simul.cast())
    }

    /// `ConexFaces(Spine, IEdge, F1, F2)`: the faces of the edge `iedge` of the spine, in
    /// the order of the concave side of its first edge.
    pub(super) fn conex_faces_of(
        &self,
        spine: &SpineHandle,
        iedge: i32,
    ) -> Result<(Face, Face), Failure> {
        let (e1, ei) = {
            let g = lock(spine);
            (g.edges(1)?.clone(), g.edges(iedge)?.clone())
        };
        let (mut ff1, mut ff2) = search_common_faces(&self.base.my_ef_map, &e1)?;
        ff1.set_orientation(Orientation::Forward);
        ff2.set_orientation(Orientation::Forward);
        let rc = concave_choice(&ff1, &ff2, &e1)?;
        let (f1, f2) = search_common_faces(&self.base.my_ef_map, &ei)?;
        let choix = concave_choice(&f1, &f2, &ei)?;
        if rc % 2 != choix % 2 {
            return Ok((f2, f1));
        }
        Ok((f1, f2))
    }
}
