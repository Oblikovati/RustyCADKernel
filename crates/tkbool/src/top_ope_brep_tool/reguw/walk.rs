//! TopOpeBRepTool_REGUW.cxx: the walk along the UV loops of a wire: the first edge of
//! a block, the next edge at the current vertex (the nearest by matter angle) and the
//! splitting of the wire into closed loops.
use super::TopOpeBRepToolREGUW;
use crate::top_ope_brep_tool::geometry::{fun_tool_line2d, fun_tool_quad2d};
use crate::top_ope_brep_tool::tool::{
    TOOL_CLOSING, TOOL_FORWARD, TOOL_REVERSED, TopOpeBRepToolTOOL,
};
use crate::top_ope_brep_tool::topology::fun_tool_make_wire;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Direction2, Point2};
use std::f64::consts::PI;

/// Standard_NullObject of a null p-curve.
fn null_pcurve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "TopOpeBRepTool_REGUW: null p-curve; expected a 2d curve",
        "TopOpeBRepTool_REGUW",
    )
}

/// `FUN_nextdata(iStep, e, pc, v, p2d, tg)` (static): the vertex at the end of the
/// edge in the walking sense, its UV point and the oriented UV tangent there (taken at
/// 0.2345 of the way back on a non-line conic p-curve).
fn fun_nextdata(
    i_step: i32,
    e: &Edge,
    pc: &Handle<dyn Geom2dCurve>,
) -> Result<(Vertex, Point2, Direction2), Failure> {
    let iv1e1 = if i_step == 1 {
        TOOL_FORWARD
    } else {
        TOOL_REVERSED
    };
    let v = TopOpeBRepToolTOOL::vertex(iv1e1, e)?;
    let par1 = TopOpeBRepToolTOOL::par_e(iv1e1, e)?;
    let approx = fun_tool_quad2d(pc)? && !fun_tool_line2d(pc)?;
    let curve = pc.get().ok_or_else(null_pcurve)?;
    let (p2d, tg2d) = if approx {
        let iv0e1 = if i_step == 1 {
            TOOL_REVERSED
        } else {
            TOOL_FORWARD
        };
        let par0 = TopOpeBRepToolTOOL::par_e(iv0e1, e)?;
        let x: f64 = 0.2345;
        let (_, tg2d) = curve.d1((1. - x) * par1 + x * par0)?;
        (curve.value(par1)?, tg2d)
    } else {
        curve.d1(par1)?
    };
    let mut tg = Direction2::from_xy(tg2d.xy())?;
    if e.orientation() == Orientation::Reversed {
        tg.reverse();
    }
    Ok((v, p2d, tg))
}

impl TopOpeBRepToolREGUW {
    /// The code of the start vertex of the edges in the walking sense.
    fn iv0e1(&self) -> i32 {
        if self.i_step == 1 {
            TOOL_REVERSED
        } else {
            TOOL_FORWARD
        }
    }

    /// The edges starting at the vertex (then the closing ones) and the count of the
    /// starting ones.
    fn start_edges(&self, v: &Shape) -> Result<(i32, List<Shape>), Failure> {
        let co = self.mymapv_eds.find_from_key(v)?;
        let (nea, lea) = co.item(self.iv0e1())?;
        let (_, leb) = co.item(TOOL_CLOSING)?;
        let mut le = lea;
        for e in leb.iter() {
            le.append(e.clone());
        }
        Ok((nea, le))
    }

    /// The edge removed from the connexities of its vertices, then the walk data at its
    /// end.
    fn take_edge(&mut self) -> Result<bool, Failure> {
        let myed = self.myed.clone();
        for vcur in Explorer::new(&myed, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            self.mymapv_eds.change_from_key(&vcur)?.remove_item(&myed);
        }
        Ok(true)
    }

    /// The vertices no longer multiple removed from the multiple ones.
    fn purge_multiple(&mut self) -> Result<(), Failure> {
        let mut kept: List<Shape> = List::default();
        for vmu in self.my_list_vmultiple.iter() {
            if self.mymapv_eds.find_from_key(vmu)?.is_multiple() {
                kept.append(vmu.clone());
            } else {
                self.mymapvmultiple.remove(vmu);
            }
        }
        self.my_list_vmultiple = kept;
        Ok(())
    }

    /// The start vertex of a block: the first multiple vertex, else the first vertex
    /// with a starting or closing edge.
    fn block_start(&self) -> Result<Vertex, Failure> {
        if let Some(first) = self.my_list_vmultiple.first() {
            return topods::vertex(first);
        }
        for i in 1..=self.mymapv_eds.extent() {
            let v = self.mymapv_eds.find_key(i)?;
            let co = self.mymapv_eds.find_from_index(i)?;
            let (nea, _) = co.item(self.iv0e1())?;
            let (neb, _) = co.item(TOOL_CLOSING)?;
            if nea + neb != 0 {
                return topods::vertex(v);
            }
        }
        Ok(Vertex::default())
    }

    /// `InitBlock()`: the first edge of a new loop from its start vertex, removed from
    /// the connexities, with the UV start point and the walk data at its end; false
    /// when no edge is left; Standard_Failure before `Init`; `r.init_block()?`.
    pub fn init_block(&mut self) -> Result<bool, Failure> {
        self.check_init()?;
        self.myv0 = Vertex::default();
        self.myp2d0 = Point2::new(1.0e7, 1.0e7);
        self.myed = Edge::default();
        self.purge_multiple()?;
        self.myv0 = self.block_start()?;
        if self.myv0.is_null() {
            return Ok(false);
        }
        let (nea, le) = self.start_edges(&self.myv0.clone())?;
        if le.is_empty() {
            return Ok(false);
        }
        if nea > 0 {
            self.myed = topods::edge(le.first().ok_or_else(null_pcurve)?)?;
        } else {
            self.first_closing_edge(&le)?;
        }
        if self.myed.is_null() {
            return Ok(false);
        }
        self.take_edge()?;
        let Some(c2df) = self.my_corriso.uv_rep(&self.myed) else {
            return Ok(false);
        };
        let (pc, _, _, _) = c2df.pc();
        let par0 = TopOpeBRepToolTOOL::par_e(self.iv0e1(), &self.myed)?;
        self.myp2d0 = pc.get().ok_or_else(null_pcurve)?.value(par0)?;
        (self.myv, self.myp2d, self.mytg2d) = fun_nextdata(self.i_step, &self.myed, &pc)?;
        Ok(true)
    }

    /// The edge of a block starting on a closing vertex: natively the closing test is
    /// made on `myed` (still null), which is never closing, so the first edge is taken.
    fn first_closing_edge(&mut self, le: &List<Shape>) -> Result<(), Failure> {
        let cs = self.my_corriso.s().clone();
        for eb in le.iter() {
            let isc_e = TopOpeBRepToolTOOL::is_closing_e_in(&self.myed, &cs, self.fref())?;
            if !isc_e {
                self.myed = topods::edge(eb)?;
                break;
            }
        }
        Ok(())
    }

    /// The matter angle from the current tangent to the oriented tangent of the edge at
    /// its start vertex.
    fn walk_angle(&self, e: &Edge) -> Result<Option<f64>, Failure> {
        let fac: f64 = 0.45678;
        let Some(c2d) = self.my_corriso.uv_rep(e) else {
            return Ok(None);
        };
        let mut tg2d = TopOpeBRepToolTOOL::try_tg2d_app(self.iv0e1(), e, &c2d, fac)?;
        if e.orientation() == Orientation::Reversed {
            tg2d.reverse();
        }
        let tg = rustycad_tkmath::gp::Vector2::from(self.mytg2d);
        Ok(Some(if self.i_step == 1 {
            TopOpeBRepToolTOOL::matter_2d(tg, tg2d)?
        } else {
            2. * PI - TopOpeBRepToolTOOL::matter_2d(tg2d, tg)?
        }))
    }

    /// `NearestE(loe, efound)`: the edge of the list making the smallest matter angle
    /// with the current tangent; `None` when two angles are equal (Precision::Angular())
    /// or an edge has no UV representation; Standard_Failure before `Init` and
    /// Standard_NoSuchObject for an empty list; `r.nearest_e(&l)?`.
    pub fn nearest_e(&self, loe: &List<Shape>) -> Result<Option<Edge>, Failure> {
        self.check_init()?;
        let tola = precision::angular();
        let mut it = loe.iter();
        let first = it.next().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "TopOpeBRepTool_REGUW::NearestE: empty edge list; expected edges",
                "TopOpeBRepTool_REGUW",
            )
        })?;
        let mut efound = topods::edge(first)?;
        let Some(mut angfound) = self.walk_angle(&efound)? else {
            return Ok(None);
        };
        for es in it {
            let ei = topods::edge(es)?;
            if ei.is_same(&self.myed) {
                continue;
            }
            let Some(angi) = self.walk_angle(&ei)? else {
                return Ok(None);
            };
            if (angi - angfound).abs() < tola {
                return Ok(None);
            }
            if angi < angfound {
                (efound, angfound) = (ei, angi);
            }
        }
        Ok(Some(efound))
    }

    /// The edges at the current vertex starting at the current UV point (other than the
    /// current edge); `None` when one has no UV representation.
    fn next_candidates(&self) -> Result<Option<List<Shape>>, Failure> {
        let (_, le) = self.start_edges(&self.myv.clone().into_shape())?;
        let mut kept: List<Shape> = List::default();
        for es in le.iter() {
            let e = topods::edge(es)?;
            if e.is_same(&self.myed) {
                continue;
            }
            let Some(c2df) = self.my_corriso.uv_rep(es) else {
                return Ok(None);
            };
            let (pc, _, _, _) = c2df.pc();
            let par = TopOpeBRepToolTOOL::par_e(self.iv0e1(), &e)?;
            let p2de = pc.get().ok_or_else(null_pcurve)?.value(par)?;
            if p2de.is_equal(self.myp2d, self.mytol2d) {
                kept.append(es.clone());
            }
        }
        Ok(Some(kept))
    }

    /// `NextinBlock()`: the next edge of the loop at the current vertex (the nearest by
    /// matter angle among several), removed from the connexities, with the walk data at
    /// its end; false when there is none; Standard_Failure before `Init`;
    /// `r.nextin_block()?`.
    pub fn nextin_block(&mut self) -> Result<bool, Failure> {
        self.check_init()?;
        let Some(le) = self.next_candidates()? else {
            return Ok(false);
        };
        if le.is_empty() {
            return Ok(false);
        }
        if le.len() == 1 {
            self.myed = topods::edge(le.first().ok_or_else(null_pcurve)?)?;
        } else {
            let Some(efound) = self.nearest_e(&le)? else {
                return Ok(false);
            };
            self.myed = efound;
        }
        self.take_edge()?;
        let c2df = self.my_corriso.uv_rep(&self.myed).unwrap_or_default();
        let (pc, _, _, _) = c2df.pc();
        (self.myv, self.myp2d, self.mytg2d) = fun_nextdata(self.i_step, &self.myed, &pc)?;
        Ok(true)
    }

    /// The INTERNAL and EXTERNAL edges at the vertices of the loop moved into it.
    fn gather_internal(&mut self, lo_ecur: &mut List<Shape>) -> Result<(), Failure> {
        let current: Vec<Shape> = lo_ecur.iter().cloned().collect();
        for e in &current {
            for v in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                let le = self
                    .mymapv_eds
                    .change_from_key(&v)?
                    .change_item(crate::top_ope_brep_tool::TOOL_INTERNAL)?;
                let mut kept: List<Shape> = List::default();
                for ee in le.iter() {
                    if matches!(
                        ee.orientation(),
                        Orientation::Internal | Orientation::External
                    ) {
                        lo_ecur.append(ee.clone());
                    } else {
                        kept.append(ee.clone());
                    }
                }
                *le = kept;
            }
        }
        Ok(())
    }

    /// The count of the FORWARD and REVERSED edges of the UV corrector.
    fn bounded_edge_count(&self) -> i32 {
        let mut n_e = self.my_corriso.eds().extent();
        for e in self.my_corriso.eds().iter() {
            if matches!(
                e.orientation(),
                Orientation::Internal | Orientation::External
            ) {
                n_e -= 1;
            }
        }
        n_e
    }

    /// `REGU(istep, Scur, Splits)`: the walk of the edges into closed UV loops (FORWARD
    /// sense at step 1, REVERSED at step 2); `Splits` is left empty when the shape is
    /// one valid loop; false when the walk fails; Standard_Failure before `Init`;
    /// `r.regu_step(1, &w, &mut l)?`.
    pub fn regu_step(
        &mut self,
        istep: i32,
        _scur: &Shape,
        splits: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        self.check_init()?;
        self.i_step = istep;
        splits.clear();
        self.mytol2d = 1.0e-5;
        self.isinit0 = true;
        let (mut lo_ecur, mut lo_w): (List<Shape>, List<Shape>) =
            (List::default(), List::default());
        let mut nite = 0;
        let n_e = self.bounded_edge_count();
        while nite <= n_e {
            let fini = nite == n_e;
            if self.isinit0 && !fini {
                if !self.init_block()? {
                    return Ok(false);
                }
                lo_ecur.append(self.myed.clone().into_shape());
                nite += 1;
                self.isinit0 = false;
                continue;
            }
            if self.myp2d.is_equal(self.myp2d0, self.mytol2d) {
                self.gather_internal(&mut lo_ecur)?;
                let fini = nite == n_e;
                if fini && lo_w.is_empty() && !self.hasnewsplits {
                    return Ok(true);
                }
                lo_w.append(fun_tool_make_wire(&lo_ecur)?.into_shape());
                self.isinit0 = true;
                lo_ecur.clear();
                if fini {
                    for w in lo_w.iter() {
                        splits.append(w.clone());
                    }
                    return Ok(true);
                }
                continue;
            }
            if !self.nextin_block()? {
                return Ok(false);
            }
            lo_ecur.append(self.myed.clone().into_shape());
            nite += 1;
        }
        Ok(true)
    }

    /// `REGU()`: the wire split at its multiple vertices walking FORWARD, then each
    /// split walked REVERSED when it still has multiple vertices; the splits stored for
    /// the shape (none when nothing is to do); Standard_Failure before `Init`;
    /// `r.regu()?`.
    pub fn regu(&mut self) -> Result<bool, Failure> {
        self.check_init()?;
        let toregu = !self.my_list_vmultiple.is_empty() || self.hasnewsplits;
        let mut splits: List<Shape> = List::default();
        let s = self.my_s.clone();
        if !toregu {
            self.my_ow_nw.bind(s, splits);
            return Ok(true);
        }
        let mut lo_s: List<Shape> = List::default();
        if !self.regu_step(1, &s, &mut lo_s)? {
            return Ok(false);
        }
        if lo_s.is_empty() {
            lo_s.append(s.clone());
        }
        for scur in lo_s.iter() {
            self.init_step(scur)?;
            self.map_s()?;
            if self.my_list_vmultiple.is_empty() {
                splits.append(scur.clone());
                continue;
            }
            let mut sp: List<Shape> = List::default();
            if !self.regu_step(2, scur, &mut sp)? {
                return Ok(false);
            }
            if sp.is_empty() {
                sp.append(scur.clone());
            }
            for w in sp.iter() {
                splits.append(w.clone());
            }
        }
        self.my_ow_nw.bind(s, splits);
        Ok(true)
    }
}
