//! The faces of the shell face sets (TopOpeBRepBuild_ffsfs.cxx): a face to split and
//! merge built with its same domain faces in the three configurations of the
//! operation, its split faces given their ancestor faces and fused.
use super::TopOpeBRepBuildBuilder;
use super::grid_ss::{funbuild_ancestorrankget, funbuild_ancestorrankprepare};
use crate::top_ope_brep_build::globals::{
    GLOBAL_CLASSIFYSPLITEDGE, GLOBAL_LFR1, GLOBAL_LFRTOPROCESS, GLOBAL_REVOWNSPLFACORI,
    GLOBAL_SPLIT_ANC,
};
use crate::top_ope_brep_build::{
    TopOpeBRepBuildFuseFace, TopOpeBRepBuildGTopo, TopOpeBRepBuildShapeSet,
    TopOpeBRepBuildShellFaceSet,
};
use crate::top_ope_brep_ds::TopOpeBRepDSConfig;
use crate::top_ope_brep_tool::{TopOpeBRepToolShapeClassifier, fun_tool_proj_ponf};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidExplorer;
use rustycad_tktopalgo::brep_top_adaptor::BRepTopAdaptorFClass2d;
use std::cell::{Cell, RefCell};

thread_local! {
    /// `STATIC_motheropedef`.
    static STATIC_MOTHEROPEDEF: Cell<bool> = const { Cell::new(false) };
    /// `STATIC_Gmotherope`.
    static STATIC_GMOTHEROPE: RefCell<TopOpeBRepBuildGTopo> = RefCell::new(TopOpeBRepBuildGTopo::new());
}

/// `FUN_setmotherope(G)`: the mother operation defined; `fun_setmotherope(&g)`.
pub fn fun_setmotherope(g: &TopOpeBRepBuildGTopo) {
    STATIC_GMOTHEROPE.with_borrow_mut(|m| *m = *g);
    STATIC_MOTHEROPEDEF.set(true);
}

/// `FUN_unsetmotherope()`; `fun_unsetmotherope()`.
pub fn fun_unsetmotherope() {
    STATIC_MOTHEROPEDEF.set(false);
}

/// `FUN_ismotheropedef()`; `fun_ismotheropedef()`.
pub fn fun_ismotheropedef() -> bool {
    STATIC_MOTHEROPEDEF.get()
}

/// `FUN_motherope()`: the last mother operation set; `fun_motherope()`.
pub fn fun_motherope() -> TopOpeBRepBuildGTopo {
    STATIC_GMOTHEROPE.with_borrow(|m| *m)
}

/// The point (found once) of a face inside an ancestor face classified ON.
fn on_state_in_face(
    fsp: &Shape,
    f: &Shape,
    p3d: &mut Option<Point3>,
) -> Result<Option<State>, Failure> {
    if p3d.is_none() {
        let mut p = Point3::default();
        if !BRepClass3dSolidExplorer::find_a_point_in_the_face_point(&topods::face(fsp)?, &mut p)? {
            return Ok(None);
        }
        *p3d = Some(p);
    }
    let ff = topods::face(f)?;
    let (mut p2d, mut dd) = (Point2::default(), 0.);
    if !fun_tool_proj_ponf(
        p3d.unwrap_or_default(),
        &ff,
        &mut p2d,
        &mut dd,
        ExtFlag::MinMax,
        ExtAlgo::Grad,
    )? {
        return Ok(None);
    }
    if dd > BRepTool::face_tolerance(&ff)? * 1.0e1 {
        return Ok(None);
    }
    Ok(Some(
        BRepTopAdaptorFClass2d::new(&ff, 1e-8)?.perform(p2d, true)?,
    ))
}

impl TopOpeBRepBuildBuilder {
    /// `FUN_getAncestorFsp(B, SC, LF, fsp, p3ddef, p3d)` (static): the index of the face
    /// of `lf` containing the split face (0 when none, or when the point of an ON face
    /// is not found).
    fn ancestor_in(
        &self,
        sc: &mut TopOpeBRepToolShapeClassifier,
        lf: &List<Shape>,
        fsp: &Shape,
        p3d: &mut Option<Point3>,
    ) -> Result<i32, Failure> {
        for f in lf.iter() {
            let mut st = sc.state_shape_shape(fsp, f, 1)?;
            if st == State::Unknown || st == State::Out {
                continue;
            }
            if st == State::On {
                match on_state_in_face(fsp, f, p3d)? {
                    Some(s) => st = s,
                    None => return Ok(0),
                }
            }
            if st == State::In {
                return Ok(self.hds()?.shape_index(f, true));
            }
        }
        Ok(0)
    }

    /// `FUN_getAncestorFsp(B, SC, LF1, LF2, fsp)` (static): the ancestor face of the
    /// split face in the list of its rank (in both when unknown, 0 for two).
    fn ancestor_of(
        &self,
        sc: &mut TopOpeBRepToolShapeClassifier,
        lf1: &List<Shape>,
        lf2: &List<Shape>,
        fsp: &Shape,
    ) -> Result<i32, Failure> {
        let (of1, of2) = funbuild_ancestorrankget(fsp)?;
        let rkfsp = if of1 && !of2 {
            1
        } else if of2 && !of1 {
            2
        } else {
            0
        };
        let unk = rkfsp == 0;
        let (rkf1, rkf2) = {
            let bds = self.hds()?.ds();
            (
                bds.ancestor_rank(lf1.first().unwrap_or(fsp)),
                bds.ancestor_rank(lf2.first().unwrap_or(fsp)),
            )
        };
        let mut p3d = None;
        let (ison1, ison2) = (rkf1 == rkfsp, rkf2 == rkfsp);
        let mut ianc1 = 0;
        if ison1 || unk {
            ianc1 = self.ancestor_in(sc, lf1, fsp, &mut p3d)?;
        }
        if ison1 {
            return Ok(ianc1);
        }
        let mut ianc2 = 0;
        if ison2 || unk {
            ianc2 = self.ancestor_in(sc, lf2, fsp, &mut p3d)?;
        }
        if ison2 {
            return Ok(ianc2);
        }
        Ok(match (ianc1, ianc2) {
            (0, i) | (i, 0) => i,
            _ => 0,
        })
    }

    /// `FUN_getAncestorFsp(B, SC, LF1, LF2, FOR, SplitAnc)` (static): the ancestor faces
    /// of the IN and OUT split faces of `for_` recorded in `GLOBAL_SplitAnc`.
    fn record_split_ancestors(
        &mut self,
        lf1: &List<Shape>,
        lf2: &List<Shape>,
        for_: &Shape,
    ) -> Result<(), Failure> {
        if GLOBAL_SPLIT_ANC.with_borrow(|m| m.is_none()) {
            return Ok(());
        }
        let mut spfor = List::default();
        for sta in [State::In, State::Out] {
            if self.is_split(for_, sta) {
                for s in self.splits(for_, sta).iter() {
                    spfor.append(s.clone());
                }
            }
        }
        let mut sc = std::mem::replace(
            &mut self.my_shape_classifier,
            TopOpeBRepToolShapeClassifier::new(),
        );
        let r = self.bind_split_ancestors(&mut sc, lf1, lf2, &spfor);
        self.my_shape_classifier = sc;
        r
    }

    /// Each split face not recorded yet bound to its ancestor (when found).
    fn bind_split_ancestors(
        &self,
        sc: &mut TopOpeBRepToolShapeClassifier,
        lf1: &List<Shape>,
        lf2: &List<Shape>,
        spfor: &List<Shape>,
    ) -> Result<(), Failure> {
        for fsp in spfor.iter() {
            if GLOBAL_SPLIT_ANC.with_borrow(|m| m.as_ref().is_some_and(|m| m.is_bound(fsp))) {
                continue;
            }
            let ianc = self.ancestor_of(sc, lf1, lf2, fsp)?;
            if ianc != 0 {
                GLOBAL_SPLIT_ANC.with_borrow_mut(|m| {
                    m.get_or_insert_with(DataMap::default)
                        .bind(fsp.clone(), ianc)
                });
            }
        }
        Ok(())
    }

    /// One configuration of the faces of the same domain: their faces made, the split
    /// faces oriented by rank and given ancestors, the face split into the set.
    #[allow(clippy::too_many_arguments)]
    fn same_domain_case(
        &mut self,
        lf1: &List<Shape>,
        lf2: &List<Shape>,
        lso2: &List<Shape>,
        gm: &TopOpeBRepBuildGTopo,
        c2: TopOpeBRepDSConfig,
        for_: &Shape,
        hsd: bool,
    ) -> Result<(), Failure> {
        GLOBAL_CLASSIFYSPLITEDGE.set(true);
        let r = self.g_fill_faces_wes_make_faces(lf1, lf2, lso2, gm);
        GLOBAL_CLASSIFYSPLITEDGE.set(false);
        r?;
        GLOBAL_REVOWNSPLFACORI.set(true);
        funbuild_ancestorrankprepare(self, lf1, lf2, TopOpeBRepDSConfig::SameOriented, c2)?;
        if hsd {
            self.record_split_ancestors(lf1, lf2, for_)?;
        }
        Ok(())
    }

    /// In a common, the IN splits of the same domain faces cleared, those of the face
    /// kept (copied back) for reference.
    fn keep_reference_splits(&mut self, for_: &Shape, lf1: &List<Shape>, lf2: &List<Shape>) {
        if !self.is_split(for_, State::In) {
            return;
        }
        let spfor = self.splits(for_, State::In).clone();
        for f in lf1.iter().chain(lf2.iter()) {
            if self.is_split(f, State::In) {
                self.change_split(f, State::In).clear();
            }
        }
        let mut c = spfor;
        self.change_split(for_, State::In).append_list(&mut c);
    }

    /// The table of a different orientation case: the ON parts of the state `tb` moved
    /// to IN.
    fn diff_table(
        gin: &TopOpeBRepBuildGTopo,
        tb: State,
        first: bool,
    ) -> Result<TopOpeBRepBuildGTopo, Failure> {
        let mut gm = *gin;
        if first {
            gm.change_value(tb, State::On, false)?;
            gm.change_value(State::In, State::On, true)?;
        } else {
            gm.change_value(State::On, tb, false)?;
            gm.change_value(State::On, State::In, true)?;
        }
        gm.change_config(
            TopOpeBRepDSConfig::SameOriented,
            TopOpeBRepDSConfig::DiffOriented,
        );
        Ok(gm)
    }

    /// The merge of a face to split and merge (its reference or a face of the first
    /// shape whose reference is not merged): its faces made with the same domain faces
    /// in the three configurations, the faces of the first one fused into the set;
    /// false when the fusion is not done.
    fn merge_face_sfs(
        &mut self,
        for_: &Shape,
        lso2: &List<Shape>,
        gin: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<bool, Failure> {
        let (tb1, tb2) = gin.states_on()?;
        let hsd = self.hds()?.has_same_domain(for_, true);
        GLOBAL_LFRTOPROCESS.set(true);
        GLOBAL_LFR1.with_borrow_mut(|l| l.get_or_insert_with(List::default).clear());
        GLOBAL_SPLIT_ANC.with_borrow_mut(|m| m.get_or_insert_with(DataMap::default).clear(true));
        let ff = for_.oriented(Orientation::Forward);
        let (mut lfso, mut lfdo) = (List::default(), List::default());
        self.g_find_sam_dom_sodo_of(&ff, &mut lfso, &mut lfdo)?;
        let rank_f = self.g_shape_rank(&ff);
        let rank_x = match rank_f {
            0 => 0,
            1 => 2,
            _ => 1,
        };
        let (mut lfso1, mut lfso2, mut lfdo2) = (List::default(), List::default(), List::default());
        self.g_find_same_rank(&lfso, rank_f, &mut lfso1);
        self.g_find_same_rank(&lfso, rank_x, &mut lfso2);
        self.g_find_same_rank(&lfdo, rank_x, &mut lfdo2);
        fun_unsetmotherope();
        let mut gm = *gin;
        gm.change_config(
            TopOpeBRepDSConfig::SameOriented,
            TopOpeBRepDSConfig::SameOriented,
        );
        if !lfso1.is_empty() && !lfso2.is_empty() {
            self.same_domain_case(
                &lfso1,
                &lfso2,
                lso2,
                &gm,
                TopOpeBRepDSConfig::SameOriented,
                for_,
                hsd,
            )?;
            GLOBAL_LFR1.with_borrow_mut(|l| l.get_or_insert_with(List::default).clear());
            self.g_split_face_sfs(for_, lso2, &gm, sfs)?;
            GLOBAL_LFRTOPROCESS.set(false);
            GLOBAL_REVOWNSPLFACORI.set(false);
        }
        let dodo = !lfso1.is_empty() && !lfdo2.is_empty();
        let gm = Self::diff_table(gin, tb2, true)?;
        fun_setmotherope(&gm);
        if dodo {
            self.same_domain_case(
                &lfso1,
                &lfdo2,
                lso2,
                &gm,
                TopOpeBRepDSConfig::DiffOriented,
                for_,
                hsd,
            )?;
            if self.opecom() {
                self.keep_reference_splits(for_, &lfso1, &lfdo2);
            }
            self.g_split_face_sfs(for_, lso2, &gm, sfs)?;
            GLOBAL_REVOWNSPLFACORI.set(false);
        }
        if !self.opecom() {
            let gm = Self::diff_table(gin, tb1, false)?;
            fun_setmotherope(&gm);
            if dodo {
                self.same_domain_case(
                    &lfso1,
                    &lfdo2,
                    lso2,
                    &gm,
                    TopOpeBRepDSConfig::DiffOriented,
                    for_,
                    hsd,
                )?;
                self.g_split_face_sfs(for_, lso2, &gm, sfs)?;
                GLOBAL_REVOWNSPLFACORI.set(false);
            }
        }
        fun_unsetmotherope();
        GLOBAL_SPLIT_ANC.with_borrow_mut(|m| {
            m.get_or_insert_with(DataMap::<Shape, i32, ShapeMapHasher>::default)
                .clear(true)
        });
        let lfr1 = GLOBAL_LFR1.with_borrow(|l| l.clone().unwrap_or_default());
        let mut fufa = TopOpeBRepBuildFuseFace::new();
        fufa.init(&List::default(), &lfr1, 1);
        fufa.perform_face()?;
        if !fufa.is_done() {
            return Ok(false);
        }
        for f in fufa.l_fuse_face().iter() {
            sfs.add_start_element(f)?;
        }
        Ok(true)
    }

    /// `GFillFaceSFS(FOR, LSO2, Gin, SFS)`: a face to split and merge merged with its
    /// same domain faces (when it is to make the merge), a face to split only split, or
    /// its splits added; the edges to avoid cleared; `b.g_fill_face_sfs(&f, &l, &g, &mut sfs)?`.
    pub fn g_fill_face_sfs(
        &mut self,
        for_: &Shape,
        lso2: &List<Shape>,
        gin: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        let (tb1, _) = gin.states_on()?;
        let tosplit = self.g_to_split(for_, tb1)?;
        let tomerge = self.g_to_merge(for_)?;
        let rk_for = self.hds()?.ds().ancestor_rank(for_);
        GLOBAL_LFRTOPROCESS.set(false);
        if tosplit && tomerge {
            let fref = {
                let hds = self.hds()?;
                hds.shape(hds.same_domain_reference(for_), true)
            };
            let makemerge = for_.is_same(&fref) || (!self.g_to_merge(&fref)? && rk_for == 1);
            if makemerge && !self.merge_face_sfs(for_, lso2, gin, sfs)? {
                return Ok(());
            }
        } else if tosplit && !tomerge {
            self.g_split_face(for_, gin, lso2)?;
            self.g_split_face_sfs(for_, lso2, gin, sfs)?;
        } else if !tosplit && !tomerge {
            self.g_split_face_sfs(for_, lso2, gin, sfs)?;
        }
        self.my_edge_avoid.clear();
        Ok(())
    }
}
