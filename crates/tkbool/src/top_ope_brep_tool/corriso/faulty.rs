//! TopOpeBRepTool_CORRISO.cxx: the edges with unconnected UV ends, the faulty closing
//! edges and the edges out of the UV bounds of the reference face.
use super::{EdgeFlagMap, TopOpeBRepToolCORRISO};
use crate::top_ope_brep_tool::c2df::TopOpeBRepToolC2DF;
use crate::top_ope_brep_tool::tool::TopOpeBRepToolTOOL;
use crate::top_ope_brep_tool::tool_iso::uviso_c2df;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::gp::{Direction2, Point2};

/// `FUN_getx(E, c2df, uiso, par)` (static): the coordinate along an iso of the UV
/// point at the parameter.
fn fun_getx(c2df: &TopOpeBRepToolC2DF, uiso: bool, par: f64) -> Result<f64, Failure> {
    let uv = TopOpeBRepToolTOOL::uvf(par, c2df)?;
    Ok(if uiso { uv.y() } else { uv.x() })
}

impl TopOpeBRepToolCORRISO {
    /// The UV point of the vertex `iv` of an edge on its representation.
    fn uv_of_vertex(
        &self,
        iv: i32,
        e: &Edge,
        c2df: &TopOpeBRepToolC2DF,
    ) -> Result<Point2, Failure> {
        TopOpeBRepToolTOOL::uvf(TopOpeBRepToolTOOL::par_e(iv, e)?, c2df)
    }

    /// Whether the end `iv_e` of `e_ref` (at `uv_v_e`) is connected in UV to the other
    /// end of an edge at its vertex (`EdgeWithFaultyUV`).
    fn end_connected(
        &self,
        e_ref: &Edge,
        iv_e: i32,
        uv_v_e: Point2,
        tols: (f64, f64, f64),
    ) -> Result<Option<bool>, Failure> {
        let (mut tttol, tttuv_f, tttuv_e) = tols;
        let v_e = TopOpeBRepToolTOOL::vertex(iv_e, e_ref)?;
        let tttuvv_e = self.tol_uv(BRepTool::vertex_tolerance(&v_e)?)?;
        let Some(loe) = self.my_v_eds.seek(&v_e) else {
            return Ok(None);
        };
        for es in loe.iter() {
            let e = topods::edge(es)?;
            if e.is_same(e_ref)
                || matches!(
                    e.orientation(),
                    Orientation::Internal | Orientation::External
                )
            {
                continue;
            }
            let Some(a_c2df) = self.my_e_rep2d.seek(es) else {
                return Ok(None);
            };
            let ves = TopOpeBRepToolTOOL::vertices(&e)?;
            for (k, ve) in ves.iter().enumerate() {
                let ive = k as i32 + 1;
                if !ve.is_same(&v_e) {
                    continue;
                }
                let uv_ve = self.uv_of_vertex(ive, &e, a_c2df)?;
                if ive == iv_e {
                    continue;
                }
                let tttuvve = self.tol_uv(BRepTool::vertex_tolerance(ve)?)?;
                tttol = std_max(
                    tttol,
                    std_max(
                        tttuv_f,
                        std_max(tttuv_e, std_max(tttuv_e, std_max(tttuvve, tttuvv_e))),
                    ),
                );
                if uv_v_e.is_equal(uv_ve, tttol) {
                    return Ok(Some(true));
                }
            }
        }
        Ok(Some(false))
    }

    /// `EdgeWithFaultyUV(E, Ivfaulty)`: for a FORWARD or REVERSED open edge, the end not
    /// connected in UV to the other end of an edge at its vertex (1 or 2, 3 for both, 0
    /// for none); `c.edge_with_faulty_uv(&e)?`.
    pub fn edge_with_faulty_uv(&self, e: &Edge) -> Result<(bool, i32), Failure> {
        let tttol: f64 = 1.0e-8;
        let tttuv_f = self.tol_uv(BRepTool::face_tolerance(&self.my_fref)?)?;
        let tttuv_e = self.tol_uv(BRepTool::edge_tolerance(e)?)?;
        if matches!(
            e.orientation(),
            Orientation::Internal | Orientation::External
        ) {
            return Ok((false, 0));
        }
        let v_es = TopOpeBRepToolTOOL::vertices(e)?;
        if v_es[0].is_same(&v_es[1]) {
            return Ok((false, 0));
        }
        let (mut nfyv, mut ivfaulty) = (0, 0);
        for iv_e in 1..=2 {
            let Some(c2df) = self.uv_rep(e) else {
                return Ok((false, ivfaulty));
            };
            let uv_v_e = self.uv_of_vertex(iv_e, e, &c2df)?;
            let Some(ok) = self.end_connected(e, iv_e, uv_v_e, (tttol, tttuv_f, tttuv_e))? else {
                return Ok((false, ivfaulty));
            };
            if !ok {
                nfyv += 1;
                ivfaulty = iv_e;
            }
        }
        if nfyv == 2 {
            ivfaulty = 3;
        }
        Ok((ivfaulty != 0, ivfaulty))
    }

    /// `EdgesWithFaultyUV(EdsToCheck, nfybounds, FyEds, stopatfirst)`: the edges with one
    /// (`nfybounds` 1), two (2) or any (3) unconnected UV end and their faulty ends;
    /// `c.edges_with_faulty_uv(&eds, 3, &mut fy, false)?`.
    pub fn edges_with_faulty_uv(
        &self,
        eds_to_check: &List<Shape>,
        nfybounds: i32,
        fy_eds: &mut EdgeFlagMap,
        stopatfirst: bool,
    ) -> Result<bool, Failure> {
        fy_eds.clear(false);
        for echk in eds_to_check.iter() {
            let (faulty, ivfaulty) = self.edge_with_faulty_uv(&topods::edge(echk)?)?;
            if !faulty {
                continue;
            }
            let nfyv = if ivfaulty == 3 { 2 } else { 1 };
            let found = match nfybounds {
                1 | 2 => nfyv == nfybounds,
                3 => nfyv > 0,
                _ => false,
            };
            if found {
                fy_eds.bind(echk.clone(), ivfaulty);
                if stopatfirst {
                    return Ok(true);
                }
            }
        }
        Ok(!fy_eds.is_empty())
    }

    /// `EdgeWithFaultyUV(EdsToCheck, nfybounds, fyE, Ifaulty)`: the first such edge and
    /// its faulty ends; `c.first_edge_with_faulty_uv(&eds, 3)?`.
    pub fn first_edge_with_faulty_uv(
        &self,
        eds_to_check: &List<Shape>,
        nfybounds: i32,
    ) -> Result<Option<(Shape, i32)>, Failure> {
        let mut fy_eds = EdgeFlagMap::default();
        if !self.edges_with_faulty_uv(eds_to_check, nfybounds, &mut fy_eds, true)? {
            return Ok(None);
        }
        Ok(fy_eds.items().next().map(|(k, v)| (k.clone(), *v)))
    }

    /// The tolerance of the UV distance between the first end of a closing edge and the
    /// last end of its other occurrence, and that distance; `None` without a UV
    /// representation.
    fn closing_ends(&self, c_e: &Edge) -> Result<Option<(f64, f64)>, Failure> {
        let tttuvc_e = self.tol_uv(BRepTool::edge_tolerance(c_e)?)?;
        let Some(c_e2d) = self.uv_rep(c_e) else {
            return Ok(None);
        };
        let oc_e = topods::edge(&c_e.oriented(top_abs::complement(c_e.orientation())))?;
        let tttuvoc_e = self.tol_uv(BRepTool::edge_tolerance(&oc_e)?)?;
        let Some(oc_e2d) = self.uv_rep(&oc_e) else {
            return Ok(None);
        };
        let uvvce1 = self.uv_of_vertex(1, c_e, &c_e2d)?;
        let uvvoce2 = self.uv_of_vertex(2, &oc_e, &oc_e2d)?;
        Ok(Some((
            uvvce1.distance(uvvoce2),
            std_max(tttuvc_e, tttuvoc_e),
        )))
    }

    /// The single faulty closing edge of `PurgeFyClosingE` (`ivf == 3`, cto016G*);
    /// `Err(())` when the purge returns false at once.
    fn purge_single(
        &self,
        fyceds: &EdgeFlagMap,
        mapcl: &IndexedMap<Shape>,
        fy_cl_eds: &mut List<Shape>,
    ) -> Result<Result<(), ()>, Failure> {
        let mut fyeds = EdgeFlagMap::default();
        self.edges_with_faulty_uv(&self.my_eds, 3, &mut fyeds, false)?;
        let nfy = fyeds.extent();
        let Some((ce_s, &ivf)) = fyceds.items().next() else {
            return Ok(Ok(()));
        };
        let c_e = topods::edge(ce_s)?;
        let oo_ce = top_abs::complement(c_e.orientation());
        if mapcl.contains(&c_e.oriented(oo_ce)) {
            let Some((d, tol)) = self.closing_ends(&c_e)? else {
                return Ok(Err(()));
            };
            if d < tol && nfy != 1 {
                return Ok(Err(()));
            }
        }
        if ivf == 3 {
            fy_cl_eds.append(ce_s.clone());
        }
        Ok(Ok(()))
    }

    /// The faulty closing edges on one iso line (`ivf` 1 or 2, cto016E*); `Err(())` when
    /// the purge returns false at once.
    fn purge_several(
        &self,
        fyceds: &EdgeFlagMap,
        tols: (f64, f64),
        fy_cl_eds: &mut List<Shape>,
    ) -> Result<Result<(), ()>, Failure> {
        let (tolu, tolv) = tols;
        let mut first: Option<(bool, bool, Point2)> = None;
        let (mut xinf, mut xsup) = (1.0e7_f64, -1.0e7_f64);
        let (mut infdef, mut supdef) = (false, false);
        for (ce_s, &ivf) in fyceds.items() {
            let c_e = topods::edge(ce_s)?;
            let Some(c2df) = self.uv_rep(ce_s) else {
                return Ok(Err(()));
            };
            let (mut d2dx, mut o2dx) = (Direction2::X, Point2::default());
            let (uvisox, isoux, isovx) = uviso_c2df(&c2df, &mut d2dx, &mut o2dx)?;
            if !uvisox {
                return Ok(Err(()));
            }
            let (isou, isov, o2d) = match first {
                None => {
                    first = Some((isoux, isovx, o2dx));
                    (isoux, isovx, o2dx)
                }
                Some((isou, isov, o2d)) => {
                    if !((isou && isoux) || (isov && isovx)) {
                        return Ok(Err(()));
                    }
                    let mut onsamline = false;
                    if isou && isoux {
                        onsamline = (o2d.x() - o2dx.x()).abs() < tolu;
                    }
                    if isov && isovx {
                        onsamline = (o2d.y() - o2dx.y()).abs() < tolv;
                    }
                    if !onsamline {
                        return Ok(Err(()));
                    }
                    (isou, isov, o2d)
                }
            };
            let _ = (isov, o2d);
            for i in 1..=2 {
                let xi = fun_getx(&c2df, isou, TopOpeBRepToolTOOL::par_e(i, &c_e)?)?;
                let vifaulty = ivf == i || ivf == 3;
                if xi < xinf {
                    (xinf, infdef) = (xi, vifaulty);
                }
                if xi > xsup {
                    (xsup, supdef) = (xi, vifaulty);
                }
            }
            fy_cl_eds.append(ce_s.clone());
        }
        if !(infdef && supdef) {
            fy_cl_eds.clear();
        }
        Ok(Ok(()))
    }

    /// Whether the end `ivce` of the closing edge is connected in UV (or one period
    /// apart) to the other end of a non-closing edge at its vertex; `None` without
    /// connexity or representation.
    fn closing_end_connected(
        &self,
        c_e: &Edge,
        ivce: i32,
        mapcl: &IndexedMap<Shape>,
        tols: (f64, f64, f64),
    ) -> Result<Option<bool>, Failure> {
        let (mut tttol, tttuv_f, tttuvc_e) = tols;
        let vce = TopOpeBRepToolTOOL::vertex(ivce, c_e)?;
        let Some(loe) = self.connexity(&vce) else {
            return Ok(None);
        };
        let Some(c_e2d) = self.uv_rep(c_e) else {
            return Ok(None);
        };
        let uvvce = self.uv_of_vertex(ivce, c_e, &c_e2d)?;
        let tttuvvce = self.tol_uv(BRepTool::vertex_tolerance(&vce)?)?;
        for es in loe.iter() {
            if mapcl.contains(es) {
                continue;
            }
            let e = topods::edge(es)?;
            let Some(e2d) = self.uv_rep(es) else {
                return Ok(None);
            };
            let tttuv_e = self.tol_uv(BRepTool::edge_tolerance(&e)?)?;
            let v_e = TopOpeBRepToolTOOL::vertices(&e)?;
            for (k, ve) in v_e.iter().enumerate() {
                let ive = k as i32 + 1;
                if !ve.is_same(&vce) {
                    continue;
                }
                let uvve = self.uv_of_vertex(ive, &e, &e2d)?;
                if ive == ivce {
                    continue;
                }
                let tttuvve = self.tol_uv(BRepTool::vertex_tolerance(ve)?)?;
                tttol = std_max(
                    tttol,
                    std_max(
                        tttuv_f,
                        std_max(tttuv_e, std_max(tttuvc_e, std_max(tttuvve, tttuvvce))),
                    ),
                );
                let mut sameuv = uvve.distance(uvvce) < tttol;
                if self.my_uclosed {
                    sameuv = sameuv || ((uvve.x() - uvvce.x()).abs() - self.my_uper).abs() < tttol;
                }
                if self.my_vclosed {
                    sameuv = sameuv || ((uvve.y() - uvvce.y()).abs() - self.my_vper).abs() < tttol;
                }
                if sameuv {
                    return Ok(Some(true));
                }
                break;
            }
        }
        Ok(Some(false))
    }

    /// `PurgeFyClosingE(ClEds, fyClEds)`: the faulty closing edges to remove among the
    /// closing edges given: a single faulty one, the faulty ones of an iso line whose
    /// UV extremes are both faulty, else the closing edges lying on their other
    /// occurrence in UV whose ends connect to no other edge;
    /// `c.purge_fy_closing_e(&cleds, &mut fy)?`.
    pub fn purge_fy_closing_e(
        &self,
        cl_eds: &List<Shape>,
        fy_cl_eds: &mut List<Shape>,
    ) -> Result<bool, Failure> {
        fy_cl_eds.clear();
        let tttol: f64 = 1.0e-8;
        let tttol_s = BRepTool::face_tolerance(&self.my_fref)?;
        let (tolu, tolv) = (self.tol(1, tttol_s)?, self.tol(2, tttol_s)?);
        let tttuv_f = std_max(tolu, tolv);
        let mut mapcl: IndexedMap<Shape> = IndexedMap::default();
        for c in cl_eds.iter() {
            mapcl.add(c.clone());
        }
        let mut fyceds = EdgeFlagMap::default();
        if !self.edges_with_faulty_uv(cl_eds, 3, &mut fyceds, false)? {
            return Ok(false);
        }
        let step = if fyceds.extent() == 1 {
            self.purge_single(&fyceds, &mapcl, fy_cl_eds)?
        } else {
            self.purge_several(&fyceds, (tolu, tolv), fy_cl_eds)?
        };
        if step.is_err() {
            return Ok(false);
        }
        if !fy_cl_eds.is_empty() {
            return Ok(true);
        }
        for ce_s in cl_eds.iter() {
            let c_e = topods::edge(ce_s)?;
            let tttuvc_e = self.tol_uv(BRepTool::edge_tolerance(&c_e)?)?;
            if self.uv_rep(ce_s).is_none() {
                return Ok(false);
            }
            let oc_e = c_e.oriented(top_abs::complement(c_e.orientation()));
            if !mapcl.contains(&oc_e) {
                continue;
            }
            let Some((d, tol)) = self.closing_ends(&c_e)? else {
                return Ok(false);
            };
            if d >= tol {
                continue;
            }
            let mut nvc_eok = 0;
            for ivce in 1..=2 {
                match self.closing_end_connected(&c_e, ivce, &mapcl, (tttol, tttuv_f, tttuvc_e))? {
                    None => return Ok(false),
                    Some(true) => nvc_eok += 1,
                    Some(false) => {}
                }
            }
            if nvc_eok == 0 {
                fy_cl_eds.append(ce_s.clone());
            }
        }
        Ok(!fy_cl_eds.is_empty())
    }
}
