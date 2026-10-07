//! HLRBRep_Data.cxx: RejectedPoint, EdgeState and AdjustParameter, the interference
//! of one intersection point.
use super::update::{sl_props, toler10};
use super::{Body, Data};
use crate::hlr_brep::{Curve, EdgeData, EdgeFaceTool};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, State};
use rustycad_tkgeomalgo::int_res2d::{
    IntersectionPoint, Position, Situation, Transition, TypeTrans,
};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Vector2};

// `THE_PARAMETER_CUT` of AdjustParameter.
const PARAMETER_CUT: f64 = 1.0e-1;

/// AdjustParameter(E, h, p, t): the start (`h`) or end bound of the edge, moved
/// inwards by a tenth of the range at a vertical end; returns `(p, t)`.
fn adjust_parameter(e: &EdgeData, h: bool) -> (f64, f32) {
    let (p1, t1, p2, t2) = e.status().bounds();
    if h {
        let p = if e.ver_at_sta() {
            p1 + (p2 - p1) * PARAMETER_CUT
        } else {
            p1
        };
        (p, t1)
    } else {
        let p = if e.ver_at_end() {
            p2 - (p2 - p1) * PARAMETER_CUT
        } else {
            p2
        };
        (p, t2)
    }
}

// FORWARD and REVERSED swap; the other orientations stay.
fn flip(o: Orientation) -> Orientation {
    match o {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
        other => other,
    }
}

/// The transition on the face edge (`Orie`), or None when it is undecided.
fn face_transition(tr1: &Transition, fe_ori: Orientation) -> Result<Option<Orientation>, Failure> {
    let rev = fe_ori == Orientation::Reversed;
    let pick = |a: Orientation, b: Orientation| Some(if rev { a } else { b });
    Ok(match tr1.transition_type() {
        TypeTrans::In => pick(Orientation::Reversed, Orientation::Forward),
        TypeTrans::Out => pick(Orientation::Forward, Orientation::Reversed),
        TypeTrans::Touch => match tr1.situation()? {
            Situation::Inside => pick(Orientation::External, Orientation::Internal),
            Situation::Outside => pick(Orientation::Internal, Orientation::External),
            Situation::Unknown => None,
        },
        TypeTrans::Undecided => None,
    })
}

/// Whether the tangent at the adjusted end `p` turns back from the tangent at the
/// original end `psav` (the second derivative where the tangent vanishes) on a
/// vertical end (`douteux`).
fn turns_back(c: &Curve, psav: f64, p: f64) -> Result<bool, Failure> {
    let (mut pt, mut tg_sav, mut nm_sav) =
        (Point2::default(), Vector2::default(), Vector2::default());
    c.d2(psav, &mut pt, &mut tg_sav, &mut nm_sav)?;
    if tg_sav.square_magnitude() <= precision::angular() {
        tg_sav = nm_sav;
    }
    let mut tg = Vector2::default();
    c.d1(p, &mut pt, &mut tg)?;
    Ok(tg.xy().dot(tg_sav.xy()) < 0.0)
}

impl Data {
    /// EdgeState(p1, p2): the states of the current edge before and after its point
    /// of parameter `p1` against the hiding face at the face edge parameter `p2`;
    /// `let (before, after) = ds.edge_state(0.5, 0.25)?;`.
    pub fn edge_state(&self, p1: f64, p2: f64) -> Result<(State, State), Failure> {
        self.body(|b| b.edge_state(p1, p2))
    }
}

impl Body<'_> {
    pub(super) fn edge_state(&self, p1: f64, p2: f64) -> Result<(State, State), Failure> {
        let s = self.face_geom()?;
        let fe_geom = self.edge(self.w.fe_geom)?.geometry();
        let Some((pu, pv)) = EdgeFaceTool::uv_point(p2, fe_geom, s)? else {
            return Ok((State::Out, State::Out));
        };
        let mut props = sl_props(s)?;
        props.set_parameters(pu, pv)?;
        if !props.is_normal_defined()? {
            return Ok((State::Out, State::Out));
        }
        let mut nrm_face = props.normal()?;
        let (pbid, tng_edge) = self.edge(self.w.le)?.geometry().d1_3d(p1)?;
        let mut v = self.sight(pbid)?;
        v.transform(self.w.proj.inverted_transformation());
        if nrm_face.dot(v) > 0. {
            nrm_face.reverse();
        }
        let scal = if tng_edge.square_magnitude() > 1.0e-10 {
            nrm_face.dot(Direction3::from_vector(tng_edge)?)
        } else {
            0.
        };
        let lim = toler10(self.w.toler);
        Ok(if scal > lim {
            (State::In, State::Out)
        } else if scal < -lim {
            (State::Out, State::In)
        } else {
            (State::On, State::On)
        })
    }

    // The sight direction at `p` in the view frame: towards the eye in perspective,
    // else -Z.
    pub(super) fn sight(&self, p: Point3) -> Result<Direction3, Failure> {
        let proj = &*self.w.proj;
        if !proj.perspective() {
            return Direction3::new(0., 0., -1.);
        }
        let mut p2d = Point2::default();
        proj.project(p, &mut p2d);
        Direction3::new(p2d.x(), p2d.y(), -proj.focus()?)
    }

    /// RejectedPoint(PInter, BoundOri, NumSeg): rejects an intersection above the
    /// face or with an undecided transition, else builds the interference.
    pub(super) fn rejected_point(
        &mut self,
        p_inter: &IntersectionPoint,
        bound_ori: Orientation,
        num_seg: i32,
    ) -> Result<bool, Failure> {
        let tol_z = self.w.big_size * 0.00001;
        let (le, fe) = (self.w.le, self.w.fe);
        let (lg, fg) = (
            self.edge(le)?.geometry(),
            self.edge(self.w.fe_geom)?.geometry(),
        );
        let mut p1 = lg.parameter_3d(p_inter.param_on_first())?;
        let mut p2 = fg.parameter_3d(p_inter.param_on_second())?;
        let mut dz = lg.z(p1)? - fg.z(p2)?;
        let mut inverted = false;
        if le == fe && dz >= tol_z {
            // An auto intersection can be inverted.
            inverted = true;
            std::mem::swap(&mut p1, &mut p2);
            dz = -dz;
        }
        if dz >= tol_z {
            self.w.above_intf = true;
            return Ok(true);
        }
        self.w.above_intf = false;
        let st = if dz <= -tol_z { State::In } else { State::On };
        let (tr1, tr2) = if inverted {
            (
                p_inter.transition_of_second(),
                p_inter.transition_of_first(),
            )
        } else {
            (
                p_inter.transition_of_first(),
                p_inter.transition_of_second(),
            )
        };
        if self.w.i_face_test && self.face_test_rejects(st, &tr1, &tr2)? {
            return Ok(true);
        }
        let Some(mut orie) = face_transition(&tr1, self.w.fe_ori)? else {
            return Ok(true);
        };
        if self.w.i_face_back {
            orie = top_abs::complement(orie); // change the transition
        }
        let ori = match tr1.position_on_curve() {
            Position::Head => Orientation::Forward,
            Position::Middle => Orientation::Internal,
            Position::End => Orientation::Reversed,
        };
        let mut ind = 0;
        let mut or2 = Orientation::Internal;
        if tr2.position_on_curve() != Position::Middle {
            // The transition on the face edge, at a vertical end.
            if self.w.same_vertex {
                return Ok(true);
            }
            let head = tr2.position_on_curve() == Position::Head;
            let fed = self.edge(self.w.fe_data)?;
            ind = if head { fed.v_sta() } else { fed.v_end() };
            or2 = if head {
                Orientation::Forward
            } else {
                Orientation::Reversed
            };
            let psav = p2;
            p2 = adjust_parameter(fed, head).0;
            let douteux = if head {
                fed.ver_at_sta()
            } else {
                fed.ver_at_end()
            };
            let fg = self.edge(self.w.fe_geom)?.geometry();
            if douteux {
                if turns_back(fg, psav, p2)? {
                    orie = flip(orie);
                }
            } else {
                let (mut pt, mut tg) = (Point2::default(), Vector2::default());
                fg.d1(p2, &mut pt, &mut tg)?;
            }
            self.w.intf.change_boundary().set_2d(fe, p2);
        }
        if ori != Orientation::Internal {
            // The transition on the current edge, at a vertical end.
            let head = ori == Orientation::Forward;
            let led = self.edge(le)?;
            let psav = p1;
            p1 = adjust_parameter(led, head).0;
            let douteux = if head {
                led.ver_at_sta()
            } else {
                led.ver_at_end()
            };
            if douteux && turns_back(led.geometry(), psav, p1)? {
                orie = flip(orie);
            }
        }
        if st == State::On {
            let (stbef, staft) = self.edge_state(p1, p2)?;
            self.w.intf.change_boundary().set_state_3d(stbef, staft);
        }
        let decal = if self.w.fe_internal {
            2
        } else if st == State::In && ori == Orientation::Forward && orie == Orientation::Forward {
            0
        } else {
            1
        };
        let le_tol = self.w.le_tol;
        let inter = self.w.intf.change_intersection();
        inter.set_orientation(ori);
        inter.set_level(decal);
        inter.set_seg_index(num_seg);
        inter.set_index(ind);
        inter.set_parameter(p1);
        inter.set_tolerance(le_tol);
        inter.set_state(st);
        self.w.intf.set_orientation(or2);
        self.w.intf.set_transition(orie);
        self.w.intf.set_boundary_transition(bound_ori);
        self.w.intf.change_boundary().set_2d(fe, p2);
        Ok(false)
    }

    // The tests of a face hiding its own edges: an auto intersection inside makes
    // the edge not simple; an intersection at a shared vertex (or on the face) is
    // rejected; an intersection inside makes the face not simple.
    fn face_test_rejects(
        &mut self,
        st: State,
        tr1: &Transition,
        tr2: &Transition,
    ) -> Result<bool, Failure> {
        if self.w.le == self.w.fe {
            if st == State::In {
                let le = self.w.le;
                self.edge_mut(le)?.set_simple(false);
            }
        } else if self.w.same_vertex
            && (st == State::On
                || tr1.position_on_curve() != Position::Middle
                || tr2.position_on_curve() != Position::Middle)
        {
            return Ok(true);
        }
        if st == State::In {
            self.w.i_face_smpl = false;
        }
        Ok(false)
    }
}
