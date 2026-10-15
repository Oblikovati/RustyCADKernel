//! TopOpeBRepTool_TOOL.cxx: the orientation of a sub-shape in a shape, closing
//! vertices and edges, the ordered vertices of an edge, the position of a parameter
//! on an edge and the split of an edge at its internal vertices.
use super::{
    TOOL_CLOSING, TOOL_EXTERNAL, TOOL_FORWARD, TOOL_INTERNAL, TOOL_REVERSED, TopOpeBRepToolTOOL,
};
use crate::top_ope_brep_tool::shape_tool::TopOpeBRepToolShapeTool;
use crate::top_ope_brep_tool::tool_tangent::tggeom_e;
use crate::top_ope_brep_tool::topology::{fun_ds_copy_edge, fun_ds_parameter};
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, ShapeIterator, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Vector3;

/// The orientation code of an orientation (FORWARD 1 to EXTERNAL 4).
fn orientation_code(o: Orientation) -> i32 {
    match o {
        Orientation::Forward => TOOL_FORWARD,
        Orientation::Reversed => TOOL_REVERSED,
        Orientation::Internal => TOOL_INTERNAL,
        Orientation::External => TOOL_EXTERNAL,
    }
}

impl TopOpeBRepToolTOOL {
    /// The closing check of `OriinSor`: `Some(code)` when the answer is known before the
    /// explorer (0 for a wrong sub-shape type, CLOSING for the closing sub-shape).
    fn closing_code(sub: &Shape, s: &Shape) -> Result<Option<i32>, Failure> {
        match s.shape_type()? {
            ShapeEnum::Edge => {
                if sub.shape_type()? != ShapeEnum::Vertex {
                    return Ok(Some(0));
                }
                let vclo = Self::closed_e(&topods::edge(s)?)?;
                Ok(vclo.filter(|v| sub.is_same(v)).map(|_| TOOL_CLOSING))
            }
            ShapeEnum::Face => {
                if sub.shape_type()? != ShapeEnum::Edge {
                    return Ok(Some(0));
                }
                let f = topods::face(s)?;
                let closing = Self::closed_s(&f)? && Self::is_closing_e(&topods::edge(sub)?, &f)?;
                Ok(closing.then_some(TOOL_CLOSING))
            }
            _ => Ok(None),
        }
    }

    /// `OriinSor(sub, S, checkclo)`: the orientation code (1 to 4) of the first explored
    /// sub-shape of `S` same as `sub`, 0 when there is none; with `checkclo`, CLOSING (5)
    /// for the closing vertex of an edge or the closing edge of a closed face, 0 for a
    /// sub-shape of the wrong type; `TopOpeBRepToolTOOL::oriin_sor(&v, &e, true)?`.
    pub fn oriin_sor(sub: &Shape, s: &Shape, checkclo: bool) -> Result<i32, Failure> {
        if checkclo && let Some(code) = Self::closing_code(sub, s)? {
            return Ok(code);
        }
        for ssub in Explorer::new(s, sub.shape_type()?, ShapeEnum::Shape)? {
            if ssub.is_same(sub) {
                return Ok(orientation_code(ssub.orientation()));
            }
        }
        Ok(0)
    }

    /// `OriinSorclosed(sub, S)`: FORWARD (1) or REVERSED (2) for the first direct child
    /// of `S` equal to `sub` with that orientation, 0 otherwise (and for a sub-shape of
    /// the wrong type); `TopOpeBRepToolTOOL::oriin_sorclosed(&e, &f)?`.
    pub fn oriin_sorclosed(sub: &Shape, s: &Shape) -> Result<i32, Failure> {
        let wanted = match s.shape_type()? {
            ShapeEnum::Edge => Some(ShapeEnum::Vertex),
            ShapeEnum::Face => Some(ShapeEnum::Edge),
            _ => None,
        };
        if wanted.is_some_and(|w| sub.shape_type().ok() != Some(w)) {
            return Ok(0);
        }
        for ssub in ShapeIterator::new(s, true, true) {
            if !ssub.is_equal(sub) {
                continue;
            }
            match ssub.orientation() {
                Orientation::Forward => return Ok(TOOL_FORWARD),
                Orientation::Reversed => return Ok(TOOL_REVERSED),
                _ => {}
            }
        }
        Ok(0)
    }

    /// `ClosedE(E, vclo)`: the closing vertex of a non-degenerated edge (the first
    /// non-INTERNAL vertex met again); `TopOpeBRepToolTOOL::closed_e(&e)?`.
    pub fn closed_e(e: &Edge) -> Result<Option<Vertex>, Failure> {
        if BRepTool::degenerated(e)? {
            return Ok(None);
        }
        let mut vv = Shape::default();
        for v in Explorer::new(e, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            if v.orientation() == Orientation::Internal {
                continue;
            }
            if vv.is_null() {
                vv = v;
            } else if v.is_same(&vv) {
                return Ok(Some(topods::vertex(&vv)?));
            }
        }
        Ok(None)
    }

    /// `ClosedS(F)`: whether the basis surface of the face is closed and periodic in U
    /// or V; `TopOpeBRepToolTOOL::closed_s(&f)?`.
    pub fn closed_s(f: &Face) -> Result<bool, Failure> {
        let s = TopOpeBRepToolShapeTool::basis_surface_of_face(f)?;
        let Some(surface) = s.get() else {
            return Ok(false);
        };
        let uclosed = surface.is_u_closed()? && surface.is_u_periodic();
        let vclosed = surface.is_v_closed()? && surface.is_v_periodic();
        Ok(uclosed || vclosed)
    }

    /// The count of the explored edges of `s` same as `e`.
    fn occurrences(e: &Edge, s: &Shape) -> Result<i32, Failure> {
        let mut nbocc = 0;
        for c in Explorer::new(s, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if c.is_same(e) {
                nbocc += 1;
            }
        }
        Ok(nbocc)
    }

    /// `IsClosingE(E, F)`: whether the edge occurs twice in the face and is closed on
    /// it; `TopOpeBRepToolTOOL::is_closing_e(&e, &f)?`.
    pub fn is_closing_e(e: &Edge, f: &Face) -> Result<bool, Failure> {
        if Self::occurrences(e, f)? != 2 {
            return Ok(false);
        }
        BRepTool::is_closed_on_face(e, f)
    }

    /// `IsClosingE(E, W, F)`: whether the edge occurs twice in `W` and is closed on the
    /// face; `TopOpeBRepToolTOOL::is_closing_e_in(&e, &w, &f)?`.
    pub fn is_closing_e_in(e: &Edge, w: &Shape, f: &Face) -> Result<bool, Failure> {
        if Self::occurrences(e, w)? != 2 {
            return Ok(false);
        }
        BRepTool::is_closed_on_face(e, f)
    }

    /// `Vertices(E, Vces)`: the vertices of the edge, the one of smaller parameter first
    /// for a FORWARD edge (second for a REVERSED one);
    /// `let [v1, v2] = TopOpeBRepToolTOOL::vertices(&e)?;`.
    pub fn vertices(e: &Edge) -> Result<[Vertex; 2], Failure> {
        let ori_e = e.orientation();
        let (v1, v2) = topexp::vertices(e, false);
        let par1 = BRepTool::parameter(&v1, e)?;
        let par2 = BRepTool::parameter(&v2, e)?;
        let mut sma_first = par1 < par2;
        if ori_e == Orientation::Reversed {
            sma_first = !sma_first;
        }
        Ok(if sma_first { [v1, v2] } else { [v2, v1] })
    }

    /// `Vertex(Iv, E)`: the vertex `iv` (1 or 2) of `Vertices`;
    /// `TopOpeBRepToolTOOL::vertex(1, &e)?`.
    pub fn vertex(iv: i32, e: &Edge) -> Result<Vertex, Failure> {
        let [first, second] = Self::vertices(e)?;
        match iv {
            1 => Ok(first),
            2 => Ok(second),
            _ => Err(Failure::with_type(
                "Standard_OutOfRange",
                &format!("TopOpeBRepTool_TOOL::Vertex: index {iv}; expected 1 or 2"),
                "TopOpeBRepTool_TOOL",
            )),
        }
    }

    /// `ParE(Iv, E)`: the parameter of the vertex `iv`; `TopOpeBRepToolTOOL::par_e(1, &e)?`.
    pub fn par_e(iv: i32, e: &Edge) -> Result<f64, Failure> {
        BRepTool::parameter(&Self::vertex(iv, e)?, e)
    }

    /// `OnBoundary(par, e)`: CLOSING (5) on a bound of a closed curve, FORWARD (1) on the
    /// first bound, REVERSED (2) on the last one (within the curve resolution of its
    /// tolerance), INTERNAL (3) inside the range, EXTERNAL (4) outside;
    /// `TopOpeBRepToolTOOL::on_boundary(0.5, &e)?`.
    pub fn on_boundary(par: f64, e: &Edge) -> Result<i32, Failure> {
        let bc = BRepAdaptorCurve::with_edge(e)?;
        let closed = bc.is_closed()?;
        let (first, last) = (bc.first_parameter()?, bc.last_parameter()?);
        let tolp = bc.resolution(bc.tolerance()?)?;
        let onf = (par - first).abs() < tolp;
        let onl = (par - last).abs() < tolp;
        if (onf || onl) && closed {
            return Ok(TOOL_CLOSING);
        }
        if onf {
            return Ok(TOOL_FORWARD);
        }
        if onl {
            return Ok(TOOL_REVERSED);
        }
        if first < par && par < last {
            return Ok(TOOL_INTERNAL);
        }
        Ok(TOOL_EXTERNAL)
    }

    /// `TgINSIDE(v, E, Tg, OvinE)`: the unit tangent of the edge oriented FORWARD at the
    /// vertex, reversed when the vertex is REVERSED in it; `ovin_e` gets the orientation
    /// code of a vertex of the edge (even when the tangent fails, as natively); `None`
    /// for a vertex not of the edge or without tangent;
    /// `TopOpeBRepToolTOOL::tg_inside(&v, &e, &mut ov)?`.
    pub fn tg_inside(v: &Vertex, e: &Edge, ovin_e: &mut i32) -> Result<Option<Vector3>, Failure> {
        let efor = topods::edge(&e.oriented(Orientation::Forward))?;
        let ov_e = Self::oriin_sor(v, &efor, true)?;
        if ov_e == 0 {
            return Ok(None);
        }
        *ovin_e = ov_e;
        let iv = match ov_e {
            TOOL_CLOSING => TOOL_FORWARD,
            TOOL_FORWARD | TOOL_REVERSED => ov_e,
            _ => 0,
        };
        let par_e = if iv == 0 {
            BRepTool::parameter(v, e)?
        } else {
            Self::par_e(iv, &efor)?
        };
        let Some(mut tg) = tggeom_e(par_e, &efor)? else {
            return Ok(None);
        };
        if ov_e == TOOL_REVERSED {
            tg.reverse();
        }
        Ok(Some(tg))
    }

    /// `FUN_tool_sortVonE` (static): the vertices sorted by their parameters on the
    /// edge; vertices of equal parameters keep the last one (an indexed map of the
    /// parameters natively).
    fn sort_v_on_e(lov: &List<Shape>, e: &Edge) -> Result<Vec<Shape>, Failure> {
        let mut pars: Vec<(f64, Shape)> = Vec::new();
        for v in lov.iter() {
            let par = BRepTool::parameter(&topods::vertex(v)?, e)?;
            match pars.iter_mut().find(|(p, _)| *p == par) {
                Some(slot) => slot.1 = v.clone(),
                None => pars.push((par, v.clone())),
            }
        }
        pars.sort_by(|a, b| a.0.total_cmp(&b.0));
        Ok(pars.into_iter().map(|(_, v)| v).collect())
    }

    /// The edge from `v0` (FORWARD) to `v` (REVERSED): a copy of `efor` without
    /// sub-shapes with the vertices at their parameters.
    fn split_piece(efor: &Edge, v0: &Shape, v: &Shape) -> Result<Shape, Failure> {
        let par0 = BRepTool::parameter(&topods::vertex(v0)?, efor)?;
        let par = BRepTool::parameter(&topods::vertex(v)?, efor)?;
        let mut ed = Shape::default();
        fun_ds_copy_edge(efor, &mut ed)?;
        let bb = BRepBuilder::default();
        let v0f = v0.oriented(Orientation::Forward);
        bb.add(&mut ed, &v0f)?;
        fun_ds_parameter(&ed, &v0f, par0)?;
        let vr = v.oriented(Orientation::Reversed);
        bb.add(&mut ed, &vr)?;
        fun_ds_parameter(&ed, &vr, par)?;
        Ok(ed)
    }

    /// `SplitE(Eanc, Splits)`: an edge with more than two vertices split at its
    /// vertices sorted by parameter, the pieces appended with the orientation of the
    /// edge; false for two vertices or less; `TopOpeBRepToolTOOL::split_e(&e, &mut l)?`.
    pub fn split_e(eanc: &Edge, splits: &mut List<Shape>) -> Result<bool, Failure> {
        let o_eanc = eanc.orientation();
        let efor = topods::edge(&eanc.oriented(Orientation::Forward))?;
        let mut lov: List<Shape> = List::default();
        for v in Explorer::new(&efor, ShapeEnum::Vertex, ShapeEnum::Shape)? {
            lov.append(v);
        }
        if lov.len() <= 2 {
            return Ok(false);
        }
        let sorted = Self::sort_v_on_e(&lov, &efor)?;
        let Some((first, rest)) = sorted.split_first() else {
            return Ok(false);
        };
        let mut v0 = first.clone();
        for v in rest {
            let ed = Self::split_piece(&efor, &v0, v)?;
            splits.append(ed.oriented(o_eanc));
            // Natively `v` is oriented REVERSED in place before `v0 = v`: the next
            // parameter of `v0` is read with that orientation.
            v0 = v.oriented(Orientation::Reversed);
        }
        Ok(true)
    }
}
