// Rust translation of ChFi3d_FilBuilder::PerformTwoCorner and the file static RemoveSD
// (ChFi3d_FilBuilder_C2.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of two fillet contours at a vertex: a bevel by intersection when the
//! fillets are on all the same faces, else a corner surface (a torus between planes, or
//! a filling) between the fillets, the common face and the pivot.
use super::ChFi3dFilBuilder;
use crate::chfi_ds::{
    CommonPoint, FaceInterference, State, StripeHandle, SurfData, SurfDataHandle, lock, shared,
};
use crate::chfi3d::builder::{spine_of, two_stripes_at};
use crate::chfi3d::{InFront, index_of_surf_data, is_in_front};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::{BRepAdaptorCurve, BRepAdaptorSurface};
use rustycad_tkbrep::brep_lprop::BRepLPropCLProps;
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use std::f64::consts::PI;

/// The state of the corner of two contours: the contours, their surface data at the
/// vertex, their sides and what `ChFi3d_IsInFront` found.
pub(super) struct TwoCorner {
    pub index: i32,
    pub vtx: Vertex,
    pub st: [StripeHandle; 2],
    pub isd: [i32; 2],
    pub sens: [i32; 2],
    pub res: InFront,
    pub okinter: bool,
    pub sd: [SurfDataHandle; 2],
}

impl TwoCorner {
    /// Whether the end of the contour `k` is its first end.
    pub fn isfirst(&self, k: usize) -> bool {
        self.sens[k] == 1
    }
    /// The side of the fillet `k` on the common face (`IFaCo`).
    pub fn ifaco(&self, k: usize) -> i32 {
        if k == 0 { self.res.jf.0 } else { self.res.jf.1 }
    }
    /// The side of the fillet `k` on the face of the arc (`IFaArc`).
    pub fn ifaarc(&self, k: usize) -> i32 {
        3 - self.ifaco(k)
    }
    /// The parameter of the meeting of the pcurves on the common face (`UIntPC`).
    pub fn uintpc(&self, k: usize) -> f64 {
        if k == 0 { self.res.p.0 } else { self.res.p.1 }
    }
    /// The common point of the fillet `k` at the corner on the side `on_s`.
    pub fn vertex(&self, k: usize, on_s: i32) -> CommonPoint {
        lock(&self.sd[k]).vertex(self.isfirst(k), on_s).clone()
    }
    /// The interference of the fillet `k` on the side `on_s`.
    pub fn interference(&self, k: usize, on_s: i32) -> FaceInterference {
        lock(&self.sd[k]).interference(on_s).clone()
    }
}

/// `RemoveSD(Stripe, num1, num2)`: the surface data `num1..=num2` removed.
fn remove_sd(stripe: &StripeHandle, num1: i32, num2: i32) {
    let Some(seq) = lock(stripe).set_of_surf_data().cloned() else {
        return;
    };
    let mut seq = lock(&seq);
    if seq.is_empty() {
        return;
    }
    let (a, b) = ((num1 - 1).max(0) as usize, (num2 as usize).min(seq.len()));
    if a < b {
        seq.drain(a..b);
    }
}

/// The number of surface data of the contour.
fn nb_surf(stripe: &StripeHandle) -> i32 {
    lock(stripe)
        .set_of_surf_data()
        .map(|s| lock(s).len() as i32)
        .unwrap_or(0)
}

/// The edge of the contour at its end of sense `sens`.
fn end_edge(stripe: &StripeHandle, sens: i32) -> Result<Edge, Failure> {
    let sp = spine_of(stripe)?;
    let g = lock(&sp);
    if sens == 1 {
        return Ok(g.edges(1)?.clone());
    }
    Ok(g.edges(g.nb_edges())?.clone())
}

/// The tangent of the edge at the vertex, oriented away from the vertex at a first end.
fn tangent_at(
    e: &Edge,
    vtx: &Vertex,
    sens: i32,
) -> Result<rustycad_tkmath::gp::Direction3, Failure> {
    let curv = BRepAdaptorCurve::with_edge(e)?;
    let par = BRepTool::parameter(vtx, e)?;
    let mut cl = BRepLPropCLProps::with_parameter(&curv, par, 1, 1.0e-4)?;
    let mut dir = cl.tangent()?;
    if sens == -1 {
        dir.reverse();
    }
    Ok(dir)
}

impl ChFi3dFilBuilder {
    /// The corner degenerates to a filling of more than three sides
    /// (`PerformMoreThreeCorner(Index, 2); done = true;`).
    pub(super) fn corner_by_plate(&mut self, index: i32, nconges: i32) -> Result<(), Failure> {
        self.base.perform_more_three_corner(index, nconges)?;
        self.base.done = true;
        Ok(())
    }

    /// `PerformTwoCorner(Index)`: the corner of two fillet contours at the `index`-th
    /// vertex.
    pub(super) fn perform_two_corner_impl(&mut self, index: i32) -> Result<(), Failure> {
        self.base.done = false;
        let vtx = self.base.my_vdata_map.find_key(index)?;
        let list = self.base.my_vdata_map.find_from_index(index)?.to_vec();
        let (st, isd, sens) = two_stripes_at(&list, &vtx)?;
        let (e1, e2) = (end_edge(&st[0], sens[0])?, end_edge(&st[1], sens[1])?);
        let (dir1, dir2) = (
            tangent_at(&e1, &vtx, sens[0])?,
            tangent_at(&e2, &vtx, sens[1])?,
        );
        if dir1.angle(&dir2).abs() < PI / 180. {
            return self.corner_by_plate(index, 2);
        }
        let mut tc = TwoCorner {
            index,
            vtx,
            st,
            isd,
            sens,
            res: InFront::default(),
            okinter: false,
            sd: [shared(SurfData::new()), shared(SurfData::new())],
        };
        let isd = tc.isd;
        tc.okinter = self.in_front(&mut tc, isd)?;
        if !tc.res.visavis && !self.search_in_front(&mut tc)? {
            return self.corner_by_plate(index, 2);
        }
        tc.sd = [
            crate::chfi3d::surf_data_at(&tc.st[0], tc.isd[0])?,
            crate::chfi3d::surf_data_at(&tc.st[1], tc.isd[1])?,
        ];
        self.two_corner_by_kind(&mut tc)
    }

    /// `ChFi3d_IsInFront` on the surface data `isd` of the two contours (`Check2dDistance`).
    fn in_front(&self, tc: &mut TwoCorner, isd: [i32; 2]) -> Result<bool, Failure> {
        is_in_front(
            &self.base.ds,
            (&tc.st[0], &tc.st[1]),
            (isd[0], isd[1]),
            (tc.sens[0], tc.sens[1]),
            &mut tc.res,
            &tc.vtx,
            (true, false),
        )
    }

    /// The opposite surface data searched among the two last of each contour; the
    /// surplus one removed when found.
    fn search_in_front(&self, tc: &mut TwoCorner) -> Result<bool, Failure> {
        let nbsurf = [nb_surf(&tc.st[0]), nb_surf(&tc.st[1])];
        let range = |k: usize| -> (i32, i32) {
            if nbsurf[k] == 1 {
                (1, 1)
            } else if tc.sens[k] == 1 {
                (1, 2)
            } else {
                (nbsurf[k] - 1, nbsurf[k])
            }
        };
        let ((deb1, fin1), (deb2, fin2)) = (range(0), range(1));
        let mut found: Option<[i32; 2]> = None;
        'outer: for i1 in deb1..=fin1 {
            for i2 in deb2..=fin2 {
                tc.okinter = self.in_front(tc, [i1, i2])?;
                if tc.res.visavis {
                    found = Some([i1, i2]);
                    break 'outer;
                }
            }
        }
        let Some([isd1, isd2]) = found else {
            return Ok(false);
        };
        if tc.sens[0] == 1 && isd1 != 1 {
            remove_sd(&tc.st[0], 1, 1);
        }
        if tc.sens[0] != 1 && isd1 != nbsurf[0] {
            remove_sd(&tc.st[0], fin1, fin1);
        }
        if tc.sens[1] == 1 && isd2 != 1 {
            remove_sd(&tc.st[1], 1, 1);
        }
        if tc.sens[1] != 1 && isd2 != nbsurf[1] {
            remove_sd(&tc.st[1], fin2, fin2);
        }
        tc.isd = [
            index_of_surf_data(&tc.vtx, &tc.st[0])?.0,
            index_of_surf_data(&tc.vtx, &tc.st[1])?.0,
        ];
        Ok(true)
    }

    /// The corner by the statuses of the ends: a bevel by intersection (all same faces),
    /// a knee by intersection tried first (both on same), else a corner surface.
    fn two_corner_by_kind(&mut self, tc: &mut TwoCorner) -> Result<(), Failure> {
        let stat = [0, 1].map(|k| -> Result<State, Failure> {
            Ok(lock(&spine_of(&tc.st[k])?).status(tc.isfirst(k)))
        });
        let [s1, s2] = stat;
        let (stat1, stat2) = (s1?, s2?);
        let c1biseau = stat1 == State::AllSame;
        let c1rotule = stat1 == State::OnSame && stat2 == State::OnSame;
        let (cp1, cp2) = (tc.vertex(0, tc.ifaarc(0)), tc.vertex(1, tc.ifaarc(1)));
        if !cp1.is_on_arc() {
            return self.corner_by_plate(tc.index, 2);
        }
        let pivot = cp1.arc()?.clone();
        let yapiv = cp2.is_on_arc() && pivot.is_same(cp2.arc()?);
        let mut piv = Pivot {
            edge: pivot,
            yapiv,
            hpivot: Handle::default(),
            par: (0., 0.),
            sameparam: false,
        };
        if yapiv {
            piv.hpivot = Handle::new(BRepAdaptorCurve::with_edge(&piv.edge)?);
            piv.par = (cp1.parameter_on_arc()?, cp2.parameter_on_arc()?);
            let (t1, t2) = (piv.hpivot.value(piv.par.0)?, piv.hpivot.value(piv.par.1)?);
            piv.sameparam = t1.distance(&t2) <= self.base.tolapp3d;
        }
        let face_of = |k: usize| -> Result<Face, Failure> {
            topods::face(
                self.base
                    .ds
                    .shape(lock(&tc.sd[k]).index(tc.ifaarc(k)), true),
            )
        };
        let (ff1, ff2) = (face_of(0)?, face_of(1)?);
        if ff1.is_null() || ff2.is_null() {
            return self.corner_by_plate(tc.index, 2);
        }
        if yapiv && !self.base_pivot_on_faces(&piv.edge, &ff1, &ff2)? {
            return self.corner_by_plate(tc.index, 2);
        }
        if c1biseau {
            self.base.done = self.base.perform_two_cornerby_inter(tc.index)?;
            if !self.base.done {
                return self.corner_by_plate(tc.index, 2);
            }
        } else if c1rotule {
            self.knee_by_inter(tc)?;
        }
        if c1biseau || self.base.done {
            return Ok(());
        }
        let ctx = CornerFaces {
            faco: tc.res.face.clone(),
            ff: [ff1, ff2],
            stat: [stat1, stat2],
        };
        self.corner_surface(tc, &piv, &ctx)
    }

    /// The knee tried by intersection; the ends of the fillets restored when it fails.
    fn knee_by_inter(&mut self, tc: &TwoCorner) -> Result<(), Failure> {
        let saved = [0, 1].map(|k| {
            let g = lock(&tc.sd[k]);
            let f = tc.isfirst(k);
            (
                g.vertex(f, 1).clone(),
                g.vertex(f, 2).clone(),
                g.interference_on_s1().clone(),
                g.interference_on_s2().clone(),
            )
        });
        self.base.done = self.base.perform_two_cornerby_inter(tc.index)?;
        if self.base.done {
            return Ok(());
        }
        for (k, (v1, v2, i1, i2)) in saved.into_iter().enumerate() {
            let mut g = lock(&tc.sd[k]);
            let f = tc.isfirst(k);
            *g.change_vertex(f, 1) = v1;
            *g.change_vertex(f, 2) = v2;
            *g.change_interference_on_s1() = i1;
            *g.change_interference_on_s2() = i2;
        }
        self.base.done = false;
        Ok(())
    }

    /// Both faces at the end of the fillets are faces of the pivot.
    fn base_pivot_on_faces(&self, pivot: &Edge, ff1: &Face, ff2: &Face) -> Result<bool, Failure> {
        let (mut ok1, mut ok2) = (false, false);
        for f in self.base.my_ef_map.find_from_key(pivot.as_ref())?.iter() {
            if !ok1 && ff1.is_same(f) {
                ok1 = true;
            }
            if !ok2 && ff2.is_same(f) {
                ok2 = true;
            }
        }
        Ok(ok1 && ok2)
    }
}

/// The pivot of the corner: the edge where the fillets end on their faces at the arc.
pub(super) struct Pivot {
    pub edge: Edge,
    pub yapiv: bool,
    pub hpivot: Handle<BRepAdaptorCurve>,
    pub par: (f64, f64),
    pub sameparam: bool,
}

/// The faces of the corner: the common face, the faces at the arc, the statuses.
pub(super) struct CornerFaces {
    pub faco: Face,
    pub ff: [Face; 2],
    pub stat: [State; 2],
}

/// A face adaptor restricted to the face.
pub(super) fn face_adaptor(f: &Face) -> Result<Handle<BRepAdaptorSurface>, Failure> {
    Ok(Handle::new(BRepAdaptorSurface::with_face(f, true)?))
}
