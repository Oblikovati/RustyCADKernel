//! The section of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_Section.cxx): the split of
//! the section edges ON, IN and OUT, their same domain parts shared, their p-curves on
//! the faces of their same domain edges and on periodic faces.
use super::TopOpeBRepBuildBuilder;
use super::build::null_hds;
use crate::top_ope_brep_build::TopOpeBRepBuildShapeListOfShape;
use crate::top_ope_brep_ds::fdscnx_edge_connexity_same_shape;
use crate::top_ope_brep_tool::TopOpeBRepToolTOOL;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve as GeomAPIProjectPointOnCurve;
use rustycad_tkmath::gp::Point3;

/// `FUN_PinC(P, C, pmin, pmax, tol)` (static): the point projects on the curve range
/// within `tol`.
fn fun_pin_c(
    p: Point3,
    c: &Handle<dyn GeomCurve>,
    pmin: f64,
    pmax: f64,
    tol: f64,
) -> Result<bool, Failure> {
    let mydm = GeomAPIProjectPointOnCurve::with_range(p, c, pmin, pmax)?;
    if !mydm.extrema()?.is_done() || mydm.nb_points() == 0 {
        return Ok(false);
    }
    Ok(mydm.lower_distance()? <= tol)
}

/// `FUN_PinE(P, E)` (static): the point is on the edge within its tolerance.
fn fun_pin_e(p: Point3, e: &Edge) -> Result<bool, Failure> {
    let (ce, f, l) = BRepTool::curve(e)?;
    fun_pin_c(p, &ce, f, l, BRepTool::edge_tolerance(e)?)
}

/// `FUN_EstaEE(E, sta, EE)` (static): an inner point of `e` is in `ee` (IN) or not
/// (OUT); a degenerated edge is not IN; Standard_ProgramError for another state.
fn fun_esta_ee(e: &Edge, sta: State, ee: &Edge) -> Result<bool, Failure> {
    if BRepTool::degenerated(e)? {
        return Ok(sta != State::In);
    }
    let (ce, f, l) = BRepTool::curve(e)?;
    let t = 0.417789;
    let p = ce.get().ok_or_else(null_hds)?.value((1. - t) * f + t * l)?;
    let einee = fun_pin_e(p, ee)?;
    match sta {
        State::In => Ok(einee),
        State::Out => Ok(!einee),
        _ => Err(Failure::with_type(
            "Standard_ProgramError",
            "TopOpeBRepBuild FUN_EstaEE on invalid state",
            "TopOpeBRepBuild_Builder",
        )),
    }
}

/// `FUN_selLEE(LE, E, sta, LEsta)` (static): the edges of `le` on the state relative
/// to `e` moved to `lesta`.
fn fun_sel_lee(
    le: &mut List<Shape>,
    e: &Edge,
    sta: State,
    lesta: &mut List<Shape>,
) -> Result<(), Failure> {
    let mut kept = List::default();
    for ecur in le.iter() {
        if fun_esta_ee(&topods::edge(ecur)?, sta, e)? {
            lesta.append(ecur.clone());
        } else {
            kept.append(ecur.clone());
        }
    }
    *le = kept;
    Ok(())
}

/// `FUN_FindEinSLOS(E, loslos)`: a shape of the list is the same as `e`.
pub(crate) fn fun_find_ein_slos(e: &Shape, loslos: &List<TopOpeBRepBuildShapeListOfShape>) -> bool {
    loslos.iter().any(|s| s.shape().is_same(e))
}

/// The edges IN each same domain edge of the splits ON, by section edge.
type SlosMap = DataMap<Shape, List<TopOpeBRepBuildShapeListOfShape>, ShapeMapHasher>;

impl TopOpeBRepBuildBuilder {
    /// The non-null section edges of the structure.
    pub(super) fn section_edges_of(&self) -> Result<Vec<Edge>, Failure> {
        let bds = self.hds()?.ds();
        let mut v = Vec::new();
        for i in 1..=bds.nb_section_edges() {
            let e = bds.section_edge(i, true)?;
            if !e.is_null() {
                v.push(e);
            }
        }
        Ok(v)
    }

    /// The splits ON of `e1` in `e2` (resplit), recorded by both edges when neither is
    /// recorded by the other.
    fn record_in_splits(
        mein: &mut SlosMap,
        e1: &Shape,
        e2: &Shape,
        le1ine2: &List<Shape>,
    ) -> Result<(), Failure> {
        for e in [e1, e2] {
            if !mein.is_bound(e) {
                mein.bind(e.clone(), List::default());
            }
        }
        if fun_find_ein_slos(e2, mein.find(e1)?) || fun_find_ein_slos(e1, mein.find(e2)?) {
            return Ok(());
        }
        let mut lr = List::default();
        for e in le1ine2.iter() {
            let mut newle = List::default();
            if TopOpeBRepToolTOOL::split_e(&topods::edge(e)?, &mut newle)? {
                lr.append_list(&mut newle);
            } else {
                lr.append(e.clone());
            }
        }
        mein.change_find(e1)?
            .append(TopOpeBRepBuildShapeListOfShape::with_list(e2, &lr));
        mein.change_find(e2)?
            .append(TopOpeBRepBuildShapeListOfShape::with_list(e1, &lr));
        Ok(())
    }

    /// The pair (E1, E2) of a section edge and a same domain edge ordered by their
    /// splits ON (the edge of the splits first, by rank and reference), with the splits.
    fn ordered_pair<'a>(
        e: &'a Shape,
        esd: &'a Shape,
        lespon: &'a List<Shape>,
        lesdspon: &'a List<Shape>,
        take_e: bool,
    ) -> Option<(&'a Shape, &'a Shape, &'a List<Shape>)> {
        match (lespon.is_empty(), lesdspon.is_empty()) {
            (false, false) => Some(if take_e {
                (e, esd, lespon)
            } else {
                (esd, e, lesdspon)
            }),
            (false, true) => Some((e, esd, lespon)),
            (true, false) => Some((esd, e, lesdspon)),
            (true, true) => None,
        }
    }

    /// Step 2 of SplitSectionEdges for one section edge: its splits ON not IN its same
    /// domain edges (MEOUT) and the splits of each pair IN the other (MEIN).
    fn section_edge_sd_splits(
        &self,
        e: &Edge,
        mein: &mut SlosMap,
        meout: &mut DataMap<Shape, List<Shape>, ShapeMapHasher>,
    ) -> Result<(), Failure> {
        let hds = self.hds()?;
        let es = e.as_ref();
        let (ie, re, lesd, iref) = {
            let bds = hds.ds();
            (
                hds.shape_index(es, true),
                bds.ancestor_rank(es),
                bds.shape_same_domain(es).clone(),
                bds.same_domain_ref(es),
            )
        };
        if !self.is_split(es, State::On) || lesd.is_empty() {
            return Ok(());
        }
        let lespon = self.splits(es, State::On).clone();
        let mut leoutlesd = List::default();
        Self::g_copy_list(&lespon, &mut leoutlesd);
        for esd in lesd.iter() {
            let iesd = hds.shape_index(esd, true);
            let lesdspon = self.splits(esd, State::On).clone();
            let mut dummylos = List::default();
            fun_sel_lee(
                &mut leoutlesd,
                &topods::edge(esd)?,
                State::In,
                &mut dummylos,
            )?;
            let take_e = (re == 1 && iesd != iref) || ie == iref;
            let Some((e1, e2, le1)) = Self::ordered_pair(es, esd, &lespon, &lesdspon, take_e)
            else {
                continue;
            };
            let mut le1loc = List::default();
            Self::g_copy_list(le1, &mut le1loc);
            let mut le1ine2 = List::default();
            fun_sel_lee(&mut le1loc, &topods::edge(e2)?, State::In, &mut le1ine2)?;
            if !le1ine2.is_empty() {
                Self::record_in_splits(mein, e1, e2, &le1ine2)?;
            }
        }
        if !meout.is_bound(es) {
            meout.bind(es.clone(), List::default());
        }
        Self::g_copy_list(&leoutlesd, meout.change_find(es)?);
        Ok(())
    }

    /// Step 3 of SplitSectionEdges: the splits ON of the section edges with same domain
    /// edges set to their MEOUT splits and their MEIN resplits.
    fn reset_section_splits(
        &mut self,
        edges: &[Edge],
        mein: &SlosMap,
        meout: &DataMap<Shape, List<Shape>, ShapeMapHasher>,
    ) -> Result<(), Failure> {
        for e in edges {
            let es = e.as_ref();
            let has_sd = !self.hds()?.ds().shape_same_domain(es).is_empty();
            if !self.is_split(es, State::On)
                || !has_sd
                || (!meout.is_bound(es) && !mein.is_bound(es))
            {
                continue;
            }
            let lespon = self.change_split(es, State::On);
            lespon.clear();
            if let Ok(leout) = meout.find(es) {
                Self::g_copy_list(leout, lespon);
            }
            if let Ok(loslos) = mein.find(es) {
                for los in loslos.iter() {
                    Self::g_copy_list(los.list(), lespon);
                }
            }
        }
        Ok(())
    }

    /// The same domain edge of the edge in the face of a rank (its first one of the rank
    /// of the face in the face).
    fn esd_in_face(&self, lesd: &List<Shape>, f: &Shape) -> Result<Option<Shape>, Failure> {
        let bds = self.hds()?.ds();
        let rf = bds.ancestor_rank(f);
        for esd in lesd.iter() {
            if bds.ancestor_rank(esd) == rf
                && Explorer::new(f, ShapeEnum::Edge, ShapeEnum::Shape)?.any(|ee| ee.is_equal(esd))
            {
                return Ok(Some(esd.clone()));
            }
        }
        Ok(None)
    }

    /// Step 4 of SplitSectionEdges: the splits ON of the reference section edges get a
    /// p-curve on each face of their same domain edges.
    fn section_splits_pcurves(&mut self, edges: &[Edge]) -> Result<(), Failure> {
        let hds = self.my_data_structure.clone();
        let bb = BRepBuilder::default();
        for e in edges {
            let es = e.as_ref();
            let (lesd, ie, re) = {
                let bds = hds.get().ok_or_else(null_hds)?.ds();
                (
                    bds.shape_same_domain(es).clone(),
                    bds.shape_index(es, true),
                    bds.same_domain_ref(es),
                )
            };
            if lesd.is_empty() || ie != re {
                continue;
            }
            let mut lf = List::default();
            for esd in lesd.iter() {
                Self::g_copy_list(&fdscnx_edge_connexity_same_shape(esd, &hds)?, &mut lf);
            }
            let lon = self.change_split(es, State::On).clone();
            let tol_e = BRepTool::edge_tolerance(e)?;
            for f in lf.iter() {
                let ff = topods::face(f)?;
                let esd_f = self
                    .esd_in_face(&lesd, f)?
                    .map(|s| topods::edge(&s))
                    .transpose()?
                    .unwrap_or_default();
                for eon in lon.iter() {
                    let eone = topods::edge(eon)?;
                    if crate::top_ope_brep_tool::fc2d_has_curve_on_surface(&eone, &ff)? {
                        continue;
                    }
                    let (pc, _, _, tolpc) =
                        crate::top_ope_brep_tool::fc2d_curve_on_surface_on_edge(
                            &eone, &ff, &esd_f, true,
                        )?;
                    if pc.is_null() {
                        return Err(Failure::with_type(
                            "Standard_ProgramError",
                            "TopOpeBRepBuild_Builder::SSE null PC on F",
                            "TopOpeBRepBuild_Builder",
                        ));
                    }
                    bb.update_edge_pcurve_on_face(&eone, &pc, &ff, tol_e.max(tolpc))?;
                }
            }
        }
        Ok(())
    }

    /// `SplitSectionEdges()`: each section edge split, the splits ON of the same domain
    /// edges shared, given p-curves on the faces of their same domain edges, checked on
    /// periodic faces and moved onto the seam side of iso section edges of periodic
    /// faces; `b.split_section_edges()?`.
    pub fn split_section_edges(&mut self) -> Result<(), Failure> {
        if self.my_split_section_edges_done {
            return Ok(());
        }
        let edges = self.section_edges_of()?;
        for e in &edges {
            self.split_section_edge(e.as_ref())?;
        }
        let (mut mein, mut meout) = (SlosMap::default(), DataMap::default());
        for e in &edges {
            self.section_edge_sd_splits(e, &mut mein, &mut meout)?;
        }
        self.reset_section_splits(&edges, &mein, &meout)?;
        self.section_splits_pcurves(&edges)?;
        self.check_periodic_section_pcurves()?;
        self.move_iso_section_splits()?;
        self.my_split_section_edges_done = true;
        Ok(())
    }

    /// The shape/shape FACE interferences of the faces of the structure, with the face.
    fn face_ssi(
        &self,
    ) -> Result<
        Vec<(
            Shape,
            crate::top_ope_brep_ds::export::KindIndices,
            crate::top_ope_brep_ds::TopOpeBRepDSTransition,
            bool,
        )>,
        Failure,
    > {
        let hds = self.hds()?;
        let bds = hds.ds();
        let mut v = Vec::new();
        for i in 1..=bds.nb_shapes() {
            let f = bds.shape(i, true).clone();
            if f.shape_type()? != ShapeEnum::Face {
                continue;
            }
            for it in bds.shape_interferences(&f, true).iter() {
                let ssi: Handle<crate::top_ope_brep_ds::TopOpeBRepDSShapeShapeInterference> =
                    it.clone().cast();
                if let Some(s) = ssi.get() {
                    let d = crate::top_ope_brep_ds::export::fds_data(it);
                    if d.2 == crate::top_ope_brep_ds::TopOpeBRepDSKind::Face {
                        v.push((f.clone(), d, s.transition(), s.gbound()));
                    }
                }
            }
        }
        Ok(v)
    }

    /// Step 5 of SplitSectionEdges: the section edges between FACE transitions on a
    /// periodic (cylinder, cone, sphere) face have a 3d curve and a p-curve on it.
    fn check_periodic_section_pcurves(&self) -> Result<(), Failure> {
        let hds = self.hds()?;
        for (_, (_, gi, _, si), tfe, _) in self.face_ssi()? {
            if tfe.shape_after() != ShapeEnum::Face || tfe.shape_before() != ShapeEnum::Face {
                continue;
            }
            let fs = hds.shape(si, true);
            if !super::section_iso::fun_periodic_s(&fs)? {
                continue;
            }
            let eg = topods::edge(&hds.shape(gi, true))?;
            if !hds.ds().is_section_edge(&eg, true)
                || crate::top_ope_brep_tool::fc2d_has_curve_on_surface(&eg, &topods::face(&fs)?)?
            {
                continue;
            }
            if !crate::top_ope_brep_tool::fc2d_has_c3d(&eg)? {
                return Err(Failure::with_type(
                    "Standard_ProgramError",
                    "TopOpeBRepBuild_Builder::SSE EG without C3D",
                    "TopOpeBRepBuild_Builder",
                ));
            }
            let (pc, _, _, _) =
                crate::top_ope_brep_tool::fc2d_curve_on_surface(&eg, &topods::face(&fs)?, true)?;
            if pc.is_null() {
                return Err(Failure::with_type(
                    "Standard_ProgramError",
                    "TopOpeBRepBuild_Builder::SSE EG without PC on FS",
                    "TopOpeBRepBuild_Builder",
                ));
            }
        }
        Ok(())
    }

    /// Step 6 of SplitSectionEdges: on each periodic face, the splits ON of the iso
    /// section edges (not bound geometries) moved onto the period side of the edge.
    fn move_iso_section_splits(&mut self) -> Result<(), Failure> {
        let hds = self.my_data_structure.clone();
        let h = hds.get().ok_or_else(null_hds)?;
        for (f, (_, gi, _, _), _, gb) in self.face_ssi()? {
            let ff = topods::face(&f)?;
            let Some((uper, vper)) = super::section_iso::fun_periodic(&ff)? else {
                continue;
            };
            if gb {
                continue;
            }
            let eg = topods::edge(&h.shape(gi, true))?;
            if !h.ds().is_section_edge(&eg, true) {
                continue;
            }
            let newlon = self.iso_moved_splits(&eg, &ff, (uper, vper))?;
            if let Some(newlon) = newlon {
                let nlon = self.change_split(eg.as_ref(), State::On);
                nlon.clear();
                for e in newlon.iter() {
                    nlon.append(e.clone());
                }
            }
        }
        Ok(())
    }
}
