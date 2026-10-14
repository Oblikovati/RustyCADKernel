// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepTool/TopOpeBRepTool_makeTransition.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepTool_makeTransition`: the states before and after a point of an edge
//! crossing a face (or an edge of the face, its restriction), from the tangents and
//! curvatures at the point or from the projections of near points.
use crate::top_ope_brep_tool::normal::nt;
use crate::top_ope_brep_tool::tool::{TOOL_CLOSING, TOOL_INTERNAL, TopOpeBRepToolTOOL};
use crate::top_ope_brep_tool::tool_tangent::tggeom_e;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::gp::{Direction3, Point2};

/// The transition maker; `let mut mt = TopOpeBRepToolMakeTransition::new();`.
#[derive(Clone, Default)]
pub struct TopOpeBRepToolMakeTransition {
    my_e: Edge,
    mypb: f64,
    mypa: f64,
    myp_e: f64,
    my_fs: Face,
    myuv: Point2,
    has_es: bool,
    my_es: Edge,
    myp_es: f64,
    is_t2d: bool,
    myfactor: f64,
}

mod fun;

use fun::*;

impl TopOpeBRepToolMakeTransition {
    /// `TopOpeBRepTool_makeTransition()`; `TopOpeBRepToolMakeTransition::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Initialize(E, pbef, paft, parE, FS, uv, factor)`: the non-degenerated edge with
    /// its parameter range and the crossing parameter, the face and the UV of the point;
    /// false for a factor out of [0, 1]; the transition is 2d when the edge lies on the
    /// face (`EdgeONFace`); `mt.initialize(&e, 0., 1., 0.5, &fs, uv, 0.2)?`.
    pub fn initialize(
        &mut self,
        e: &Edge,
        (pbef, paft, par_e): (f64, f64, f64),
        fs: &Face,
        uv: Point2,
        factor: f64,
    ) -> Result<bool, Failure> {
        if BRepTool::degenerated(e)? {
            return Ok(false);
        }
        (self.my_e, self.mypb, self.mypa, self.myp_e) = (e.clone(), pbef, paft, par_e);
        (self.my_fs, self.myuv, self.myfactor, self.has_es) = (fs.clone(), uv, factor, false);
        if !(0. ..=1.).contains(&factor) {
            return Ok(false);
        }
        let found = TopOpeBRepToolTOOL::edge_on_face(self.myp_e, &self.my_e, self.myuv, fs)?;
        self.is_t2d = found.unwrap_or(false);
        Ok(found.is_some())
    }

    /// `Setfactor(factor)`; `mt.setfactor(0.2)`.
    pub fn setfactor(&mut self, factor: f64) {
        self.myfactor = factor;
    }

    /// `Getfactor()`; `mt.getfactor()`.
    pub fn getfactor(&self) -> f64 {
        self.myfactor
    }

    /// `IsT2d()`; `mt.is_t2d()`.
    pub fn is_t2d(&self) -> bool {
        self.is_t2d
    }

    /// `SetRest(ES, parES)`: the restriction edge of the face at its parameter; false for
    /// a degenerated edge; `mt.set_rest(&es, 0.5)?`.
    pub fn set_rest(&mut self, es: &Edge, par_es: f64) -> Result<bool, Failure> {
        if BRepTool::degenerated(es)? {
            return Ok(false);
        }
        (self.has_es, self.my_es, self.myp_es) = (true, es.clone(), par_es);
        Ok(true)
    }

    /// `HasRest()`; `mt.has_rest()`.
    pub fn has_rest(&self) -> bool {
        self.has_es
    }

    /// The unit tangent of the edge at the crossing parameter.
    fn tg_e(&self) -> Result<Option<Direction3>, Failure> {
        match tggeom_e(self.myp_e, &self.my_e)? {
            Some(tmp) => Ok(Some(Direction3::from_xyz(*tmp.xyz())?)),
            None => Ok(None),
        }
    }

    /// The tangent states of two quadric edges tangent at the point (`MkT2donE`).
    fn mk_t2d_tangent(&self, xx_es: Direction3) -> Result<Option<(State, State)>, Failure> {
        let mut nt_fs = Direction3::Z;
        if !nt(self.myuv, &self.my_fs, &mut nt_fs)? {
            return Ok(None);
        }
        let (ok, curv_e) = TopOpeBRepToolTOOL::curv_e(&self.my_e, self.myp_e, nt_fs)?;
        if !ok {
            return Ok(None);
        }
        let (ok, curv_es) = TopOpeBRepToolTOOL::curv_e(&self.my_es, self.myp_es, nt_fs)?;
        if !ok {
            return Ok(None);
        }
        if !(TopOpeBRepToolTOOL::is_quad_edge(&self.my_e)?
            && TopOpeBRepToolTOOL::is_quad_edge(&self.my_es)?)
        {
            return Ok(None);
        }
        let mkt = fun_mk_t2dquad_kind(curv_e, curv_es);
        // Natively called with (mypb, mypa, mypE) as (par1, f1, l1).
        let e1 = (&self.my_e, self.mypb, (self.mypa, self.myp_e));
        let found = fun_mk_t2dquad(e1, (&self.my_es, self.myp_es), mkt, xx_es, self.myfactor)?;
        Ok(found.map(|sta| (sta, sta)))
    }

    /// `MkT2donE(stb, sta)`: for an edge on the face, the states before and after the
    /// point from the side of the restriction edge (IN without restriction, or for an
    /// INTERNAL or closing restriction), or from the curvatures of tangent quadric
    /// edges; `None` when they cannot be computed; `mt.mk_t2don_e()?`.
    pub fn mk_t2don_e(&self) -> Result<Option<(State, State)>, Failure> {
        if !self.is_t2d {
            return Ok(None);
        }
        if !self.has_es {
            return Ok(Some((State::In, State::In)));
        }
        let ori_es_fs = TopOpeBRepToolTOOL::oriin_sor(&self.my_es, &self.my_fs, true)?;
        if ori_es_fs == 0 {
            return Ok(None);
        }
        if ori_es_fs == TOOL_INTERNAL || ori_es_fs == TOOL_CLOSING {
            return Ok(Some((State::In, State::In)));
        }
        let Some(tg_e) = self.tg_e()? else {
            return Ok(None);
        };
        let Some(xx_es) = TopOpeBRepToolTOOL::xx(self.myuv, &self.my_fs, self.myp_es, &self.my_es)?
        else {
            return Ok(None);
        };
        let dot = tg_e.dot(xx_es);
        if dot.abs() >= fun_tolang() {
            return Ok(Some(if dot > 0. {
                (State::Out, State::In)
            } else {
                (State::In, State::Out)
            }));
        }
        self.mk_t2d_tangent(xx_es)
    }

    /// `MkT3dproj(stb, sta)`: the states of the near points before and after projected
    /// on the face; the states written natively even on failure;
    /// `let (ok, stb, sta) = mt.mk_t3dproj()?;`.
    pub fn mk_t3dproj(&self) -> Result<(bool, State, State), Failure> {
        let (mut stb, mut sta) = (State::Unknown, State::Unknown);
        let e = (&self.my_e, (self.mypb, self.mypa), self.myp_e);
        if !fun_staproj(e, self.myfactor, BEFORE, &self.my_fs, &mut stb)? {
            return Ok((false, stb, sta));
        }
        let oka = fun_staproj(e, self.myfactor, AFTER, &self.my_fs, &mut sta)?;
        Ok((oka, stb, sta))
    }

    /// The states of a quadric edge tangent to a quadric face (`MkT3onE`).
    fn mk_t3_tangent(
        &self,
        tg_e: Direction3,
        nt_fs: Direction3,
    ) -> Result<Option<(State, State)>, Failure> {
        let tg0 = nt_fs.crossed(tg_e)?;
        let (ok, cur_e) = TopOpeBRepToolTOOL::curv_e(&self.my_e, self.myp_e, tg0)?;
        if !ok {
            return Ok(None);
        }
        let (ok, cur_fs, _) = TopOpeBRepToolTOOL::curv_f(&self.my_fs, self.myuv, tg0, false)?;
        if !ok
            || !TopOpeBRepToolTOOL::is_quad_edge(&self.my_e)?
            || !TopOpeBRepToolTOOL::is_quad_face(&self.my_fs)?
        {
            return Ok(None);
        }
        let mkt = fun_mk_t2dquad_kind(cur_e, cur_fs);
        let e = (&self.my_e, (self.mypb, self.mypa), self.myp_e);
        let Some(sta) = fun_mk_t3dquad(
            e,
            (&self.my_fs, self.myuv),
            (tg_e, nt_fs),
            mkt,
            self.myfactor,
        )?
        else {
            return Ok(None);
        };
        if !self.has_es {
            return Ok(Some((sta, sta)));
        }
        let Some(xx_es) = TopOpeBRepToolTOOL::xx(self.myuv, &self.my_fs, self.myp_es, &self.my_es)?
        else {
            return Ok(None);
        };
        Ok(Some((
            fun_sta_with_es_state(tg_e, xx_es, BEFORE, sta),
            fun_sta_with_es_state(tg_e, xx_es, AFTER, sta),
        )))
    }

    /// `MkT3onE(stb, sta)`: for an edge crossing the face, the states from the sign of
    /// the edge tangent on the face normal, or from the curvatures of a quadric edge
    /// tangent to a quadric face (restricted by the restriction edge); `None` when they
    /// cannot be computed; `mt.mk_t3on_e()?`.
    pub fn mk_t3on_e(&self) -> Result<Option<(State, State)>, Failure> {
        if self.is_t2d {
            return Ok(None);
        }
        let Some(tg_e) = self.tg_e()? else {
            return Ok(None);
        };
        let mut nt_fs = Direction3::Z;
        if !nt(self.myuv, &self.my_fs, &mut nt_fs)? {
            return Ok(None);
        }
        let dot = tg_e.dot(nt_fs);
        if dot.abs() > fun_tolang() {
            return Ok(Some(if dot > 0. {
                (State::In, State::Out)
            } else {
                (State::Out, State::In)
            }));
        }
        self.mk_t3_tangent(tg_e, nt_fs)
    }

    /// One retry of the states still ON or UNKNOWN by projection (`MkTonE`).
    fn retry_projection(
        &self,
        xx_es: Direction3,
        tg_e: Direction3,
        (stb, sta): (&mut State, &mut State),
    ) -> Result<(), Failure> {
        let e = (&self.my_e, (self.mypb, self.mypa), self.myp_e);
        let kob = *stb == State::On || *stb == State::Unknown;
        let koa = *sta == State::On || *sta == State::Unknown;
        if kob && fun_staproj(e, self.myfactor, BEFORE, &self.my_fs, stb)? && self.has_es {
            *stb = fun_sta_with_es_state(tg_e, xx_es, BEFORE, *stb);
        }
        if koa && fun_staproj(e, self.myfactor, AFTER, &self.my_fs, sta)? && self.has_es {
            *sta = fun_sta_with_es_state(tg_e, xx_es, AFTER, *sta);
        }
        Ok(())
    }

    /// `MkTonE(stb, sta)`: the 2d states for an edge on the face; else the 3d states
    /// (by tangents, else by projection), then up to five projections of nearer points
    /// (the factor grown by a fifth of the rest each time, kept) while a state is ON or
    /// UNKNOWN; the states written natively even on failure;
    /// `let (ok, stb, sta) = mt.mk_ton_e()?;`.
    pub fn mk_ton_e(&mut self) -> Result<(bool, State, State), Failure> {
        if self.is_t2d {
            return Ok(match self.mk_t2don_e()? {
                Some((stb, sta)) => (true, stb, sta),
                None => (false, State::Unknown, State::Unknown),
            });
        }
        let (mut stb, mut sta) = match self.mk_t3on_e()? {
            Some(found) => found,
            None => {
                let (_, b, a) = self.mk_t3dproj()?;
                (b, a)
            }
        };
        let Some(tg_e) = self.tg_e()? else {
            return Ok((false, stb, sta));
        };
        let mut xx_es = Direction3::Z;
        if self.has_es {
            match TopOpeBRepToolTOOL::xx(self.myuv, &self.my_fs, self.myp_es, &self.my_es)? {
                Some(found) => xx_es = found,
                None => return Ok((false, stb, sta)),
            }
        }
        let delta = (1. - self.myfactor) / 5.;
        for _ in 1..=5 {
            let kob = stb == State::On || stb == State::Unknown;
            let koa = sta == State::On || sta == State::Unknown;
            if !koa && !kob {
                return Ok((true, stb, sta));
            }
            self.retry_projection(xx_es, tg_e, (&mut stb, &mut sta))?;
            self.myfactor += delta;
        }
        Ok((false, stb, sta))
    }
}
