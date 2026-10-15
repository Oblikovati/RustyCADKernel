//! The solid merges of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_GridSS.cxx): the shell
//! face sets of the solids filled with their kept shells and faces, the split faces
//! oriented by their ancestors, and the merged faces of the same domain.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::globals::{
    GLOBAL_LFR1, GLOBAL_LFRTOPROCESS, GLOBAL_REVOWNSPLFACORI, GLOBAL_SPLIT_ANC,
};
use crate::top_ope_brep_build::{
    TopOpeBRepBuildGTool, TopOpeBRepBuildGTopo, TopOpeBRepBuildShapeSet,
    TopOpeBRepBuildShellFaceSet, TopOpeBRepBuildSolidBuilder,
};
use crate::top_ope_brep_ds::{TopOpeBRepDSConfig, TopOpeBRepDSSurfaceIterator};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeExplorer;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{IndexedMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};
use std::cell::{Cell, RefCell};

thread_local! {
    /// `stabuild_IMELF1`, `stabuild_IMELF2`: the edges of the faces of the lists (never
    /// cleared, as natively).
    static STABUILD_IMELF: RefCell<[IndexedMap<Shape, ShapeMapHasher>; 2]> = RefCell::new(Default::default());
    /// `stabuild_IDMEALF1`, `stabuild_IDMEALF2`: the split edges of those edges with
    /// their ancestors (never cleared).
    static STABUILD_IDMEALF: RefCell<[IndexedDataMapOfShapeListOfShape; 2]> = RefCell::new(Default::default());
    /// `stabuild_IMEF`: the edges of the faces asked (never cleared).
    static STABUILD_IMEF: RefCell<IndexedMap<Shape, ShapeMapHasher>> = RefCell::new(IndexedMap::default());
    /// `static_CONF1`, `static_CONF2`.
    static STATIC_CONF: Cell<(TopOpeBRepDSConfig, TopOpeBRepDSConfig)> = const { Cell::new((TopOpeBRepDSConfig::UnshGeometry, TopOpeBRepDSConfig::UnshGeometry)) };
}

/// `FUNBUILD_MAPANCSPLSHAPES(B, S, STATE, IDM)` (static): the splits of `s` on the
/// state with `s` among their ancestors.
fn map_anc_spl_shapes(
    b: &TopOpeBRepBuildBuilder,
    s: &Shape,
    state: State,
    idm: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    if !b.is_split(s, state) {
        return Ok(());
    }
    for sps in b.splits(s, state).iter() {
        if !idm.contains(sps) {
            idm.add(sps.clone(), List::default());
        }
        idm.change_from_key(sps)?.append(s.clone());
    }
    Ok(())
}

/// `FUNBUILD_ANCESTORRANKPREPARE(B, LF1, LF2, CONF1, CONF2)`: the edges of the faces of
/// the two lists and their split edges IN and OUT mapped (added to the previous ones);
/// `funbuild_ancestorrankprepare(&b, &lf1, &lf2, c1, c2)?`.
pub fn funbuild_ancestorrankprepare(
    b: &TopOpeBRepBuildBuilder,
    lf1: &List<Shape>,
    lf2: &List<Shape>,
    conf1: TopOpeBRepDSConfig,
    conf2: TopOpeBRepDSConfig,
) -> Result<(), Failure> {
    STATIC_CONF.set((conf1, conf2));
    STABUILD_IMELF.with_borrow_mut(|imelf| -> Result<(), Failure> {
        for (k, l) in [lf1, lf2].into_iter().enumerate() {
            for f in l.iter() {
                topexp::map_shapes_of_type(f, ShapeEnum::Edge, &mut imelf[k])?;
            }
        }
        STABUILD_IDMEALF.with_borrow_mut(|idm| -> Result<(), Failure> {
            for k in 0..2 {
                for i in 1..=imelf[k].extent() {
                    let e = imelf[k].find_key(i)?.clone();
                    map_anc_spl_shapes(b, &e, State::In, &mut idm[k])?;
                    map_anc_spl_shapes(b, &e, State::Out, &mut idm[k])?;
                }
            }
            Ok(())
        })
    })
}

/// `FUNBUILD_ANCESTORRANKGET(B, f, of1, of2)`: whether an edge of the mapped edges
/// (with those of `f`) is a split edge of the first (second) list;
/// `let (of1, of2) = funbuild_ancestorrankget(&f)?;`.
pub fn funbuild_ancestorrankget(f: &Shape) -> Result<(bool, bool), Failure> {
    STABUILD_IMEF.with_borrow_mut(|imef| -> Result<(bool, bool), Failure> {
        topexp::map_shapes_of_type(f, ShapeEnum::Edge, imef)?;
        STABUILD_IDMEALF.with_borrow(|idm| {
            let of = |k: usize| {
                (1..=imef.extent()).any(|i| imef.find_key(i).is_ok_and(|e| idm[k].contains(e)))
            };
            Ok((of(0), of(1)))
        })
    })
}

/// `FUNBUILD_ORIENTLOFS(B, TB1, TB2, LOFS)`: each face of a split of one list only
/// reversed as the parts of its list; `funbuild_orientlofs(State::In, State::Out, &mut l)?`.
pub fn funbuild_orientlofs(tb1: State, tb2: State, lofs: &mut List<Shape>) -> Result<(), Failure> {
    for f in lofs.iter_mut() {
        let (of1, of2) = funbuild_ancestorrankget(f)?;
        let r12 = TopOpeBRepBuildBuilder::reverse(tb1, tb2);
        let r21 = TopOpeBRepBuildBuilder::reverse(tb2, tb1);
        let rf = if of1 && !of2 {
            r12
        } else if of2 && !of1 {
            r21
        } else {
            false
        };
        let o = TopOpeBRepBuildBuilder::orient(f.orientation(), rf);
        f.set_orientation(o);
    }
    Ok(())
}

impl TopOpeBRepBuildBuilder {
    /// `GMergeSolids(LSO1, LSO2, G1)`: the solids of the shell face set of the two lists
    /// as the merged solids of those not merged yet; `b.g_merge_solids(&l1, &l2, &g)?`.
    pub fn g_merge_solids(
        &mut self,
        lso1: &List<Shape>,
        lso2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
    ) -> Result<(), Failure> {
        let Some(so1) = lso1.first().cloned() else {
            return Ok(());
        };
        let (tb1, tb2) = g1.states_on()?;
        self.my_solid_reference = topods::solid(&so1)?;
        let mut sfs = TopOpeBRepBuildShellFaceSet::with_solid(&so1)?;
        self.g_fill_solids_sfs(lso1, lso2, g1, &mut sfs)?;
        let so1f = so1.oriented(Orientation::Forward);
        let mut sobu = TopOpeBRepBuildSolidBuilder::new();
        sobu.init_solid_builder(&mut sfs, true)?;
        let mut lsom = List::default();
        self.gsobu_make_solids(&so1f, &mut sobu, &mut lsom)?;
        for (l, tb) in [(lso1, tb1), (lso2, tb2)] {
            for s in l.iter() {
                if !self.is_merged(s, tb) {
                    *self.change_merged(s, tb) = lsom.clone();
                }
            }
        }
        Ok(())
    }

    /// `GFillSolidsSFS(LS1, LS2, G1, SFS)`: the solids of each list not merged yet
    /// filled (the table permuted for the second), with the ON patches of a common or
    /// fuse; `b.g_fill_solids_sfs(&l1, &l2, &g, &mut sfs)?`.
    pub fn g_fill_solids_sfs(
        &mut self,
        ls1: &List<Shape>,
        ls2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        let Some(s1) = ls1.first() else {
            return Ok(());
        };
        let (tb1, tb2) = g1.states_on()?;
        self.my_process_on = self.opecom() || self.opefus();
        if self.my_process_on {
            self.my_on_faces_map.clear(true);
        }
        self.my_solid_reference = topods::solid(s1)?;
        for (g, l, lo, tb) in [(*g1, ls1, ls2, tb1), (g1.copy_permuted(), ls2, ls1, tb2)] {
            for s in l.iter() {
                if !self.is_merged(s, tb) {
                    self.g_fill_solid_sfs(s, lo, &g, sfs)?;
                }
            }
        }
        if self.my_process_on {
            self.add_on_patches_sfs(g1, sfs)?;
            self.my_process_on = false;
        }
        Ok(())
    }

    /// `GFillSolidSFS(SO1, LSO2, G1, SFS)` (virtual): the shells of the forward solid out
    /// of the structure added when kept, the others filled by their faces;
    /// `b.g_fill_solid_sfs(&so, &l, &g, &mut sfs)?`.
    pub fn g_fill_solid_sfs(
        &mut self,
        so1: &Shape,
        lso2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        if self.b1.is_some() {
            return self.b1_g_fill_solid_sfs(so1, lso2, g1, sfs);
        }
        self.g_fill_solid_sfs_base(so1, lso2, g1, sfs)
    }

    /// `TopOpeBRepBuild_Builder::GFillSolidSFS`, the base version.
    pub(crate) fn g_fill_solid_sfs_base(
        &mut self,
        so1: &Shape,
        lso2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        let (tb1, _) = g1.states_on()?;
        let rev_ori1 = g1.is_to_reverse1()?;
        let sof = so1.oriented(Orientation::Forward);
        self.my_solid_to_fill = topods::solid(&sof)?;
        let mut ex_shell =
            TopOpeBRepToolShapeExplorer::new(&sof, ShapeEnum::Shell, ShapeEnum::Shape)?;
        while ex_shell.more() {
            let sh = ex_shell.current().clone();
            if !self.hds()?.has_shape(&sh, true) {
                if self.g_keep_shape(&sh, lso2, tb1)? {
                    sfs.add_shape(&sh.oriented(Self::orient(sh.orientation(), rev_ori1)))?;
                }
            } else {
                self.g_fill_shell_sfs(&sh, lso2, g1, sfs)?;
            }
            ex_shell.next()?;
        }
        Ok(())
    }

    /// `GFillSurfaceTopologySFS(SO1, G1, SFS)`: nothing but the states and types read,
    /// as natively; `b.g_fill_surface_topology_sfs(&so, &g, &mut sfs)?`.
    pub fn g_fill_surface_topology_sfs(
        &self,
        _so1: &Shape,
        g1: &TopOpeBRepBuildGTopo,
        _sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        g1.states_on()?;
        g1.type_();
        Ok(())
    }

    /// `GFillSurfaceTopologySFS(SSit, G1, SFS)`: the new faces of the current surface,
    /// oriented on the state (complemented for DIFFORIENTED), as elements;
    /// `b.g_fill_surface_topology_sfs_at(&it, &g, &mut sfs)?`.
    pub fn g_fill_surface_topology_sfs_at(
        &self,
        ssit: &TopOpeBRepDSSurfaceIterator,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        let (tb1, _) = g1.states_on()?;
        let tb = match (g1.config1(), tb1) {
            (TopOpeBRepDSConfig::DiffOriented, State::Out) => State::In,
            (TopOpeBRepDSConfig::DiffOriented, State::In) => State::Out,
            _ => tb1,
        };
        for f in self.new_faces(ssit.current()?)?.iter() {
            sfs.add_element(&f.oriented(ssit.orientation(tb)?))?;
        }
        Ok(())
    }

    /// `GFillShellSFS(SH, LSO2, G1, SFS)` (virtual): the faces of the shell with same
    /// domain faces filled first, then the others; `b.g_fill_shell_sfs(&sh, &l, &g, &mut sfs)?`.
    pub fn g_fill_shell_sfs(
        &mut self,
        sh: &Shape,
        lso2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        if self.b1.is_some() {
            return self.b1_g_fill_shell_sfs(sh, lso2, g1, sfs);
        }
        self.g_fill_shell_sfs_base(sh, lso2, g1, sfs)
    }

    /// `TopOpeBRepBuild_Builder::GFillShellSFS`, the base version.
    pub(crate) fn g_fill_shell_sfs_base(
        &mut self,
        sh: &Shape,
        lso2: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        g1.states_on()?;
        for with_sd in [true, false] {
            let mut ex_face =
                TopOpeBRepToolShapeExplorer::new(sh, ShapeEnum::Face, ShapeEnum::Shape)?;
            while ex_face.more() {
                let f = ex_face.current().clone();
                if self.hds()?.has_same_domain(&f, true) == with_sd {
                    self.g_fill_face_sfs(&f, lso2, g1, sfs)?;
                }
                ex_face.next()?;
            }
        }
        Ok(())
    }

    /// The orientation of a split face of an ancestor (`GLOBAL_SplitAnc`) by the
    /// operation and the geometry of the same domain reference of the ancestor.
    fn ancestor_orientation(
        &self,
        ianc: i32,
        neworif: Orientation,
    ) -> Result<Orientation, Failure> {
        let bds = self.hds()?.ds();
        let (rk_anc, o_anc) = (
            bds.ancestor_rank_index(ianc),
            bds.shape(ianc, true).orientation(),
        );
        let mut o = neworif;
        if self.opecom() || self.opefus() {
            o = o_anc;
        } else if self.opec12() || self.opec21() {
            let mut tb_anc = State::Unknown;
            if self.opec12() {
                tb_anc = if rk_anc == 1 { State::Out } else { State::In };
            }
            if self.opec21() {
                tb_anc = if rk_anc == 2 { State::Out } else { State::In };
            }
            o = if tb_anc == State::Out {
                o_anc
            } else {
                top_abs::complement(o_anc)
            };
        }
        let iref_anc = bds.same_domain_ref_index(ianc);
        if iref_anc != ianc {
            let mut samegeom = bds.same_domain_ori_index(ianc) == TopOpeBRepDSConfig::SameOriented;
            if o_anc != bds.shape(iref_anc, true).orientation() {
                samegeom = !samegeom;
            }
            if !samegeom {
                o = top_abs::complement(o);
            }
        }
        Ok(o)
    }

    /// `GSplitFaceSFS(FOR, LSclass, G1, SFS)`: the split faces of the face on the state
    /// (oriented by their ancestors when known) as start elements (or kept in
    /// `GLOBAL_lfr1`), else the face itself, when kept, as an element;
    /// `b.g_split_face_sfs(&f, &l, &g, &mut sfs)?`.
    pub fn g_split_face_sfs(
        &mut self,
        for_: &Shape,
        lsclass: &List<Shape>,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        let (tb1, tb2) = g1.states_on()?;
        let neworif0 = Self::orient(for_.orientation(), g1.is_to_reverse1()?);
        if !self.is_split(for_, tb1) {
            let hds = self.hds()?;
            let mut add = true;
            if hds.has_shape(for_, true) && !hds.has_geometry(for_) {
                add = self.g_keep_shape(for_, lsclass, tb1)?;
            }
            if add {
                sfs.add_element(&for_.oriented(neworif0))?;
            }
            return Ok(());
        }
        if GLOBAL_REVOWNSPLFACORI.get() {
            funbuild_orientlofs(tb1, tb2, self.change_split(for_, tb1))?;
        }
        let mut neworif = neworif0;
        for new_f in self.splits(for_, tb1).clone().iter() {
            let anc =
                GLOBAL_SPLIT_ANC.with_borrow(|m| m.as_ref().map(|m| m.find(new_f).ok().copied()));
            if let Some(Some(ianc)) = anc {
                neworif = self.ancestor_orientation(ianc, neworif)?;
            }
            let new_f = new_f.oriented(neworif);
            if GLOBAL_LFRTOPROCESS.get() {
                GLOBAL_LFR1.with_borrow_mut(|l| {
                    l.get_or_insert_with(List::default).append(new_f);
                });
            } else {
                sfs.add_start_element(&new_f)?;
            }
        }
        Ok(())
    }

    /// `GMergeFaceSFS(FOR, G1, SFS)`: for a face to merge in a common of same (or
    /// different) orientations, the faces of the same domain lists merged in 2d, the
    /// merged faces of the reference face added as start elements;
    /// `b.g_merge_face_sfs(&f, &g, &mut sfs)?`.
    pub fn g_merge_face_sfs(
        &mut self,
        for_: &Shape,
        g1: &TopOpeBRepBuildGTopo,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        if !self.g_to_merge(for_)? {
            return Ok(());
        }
        g1.states_on()?;
        let neworif = Self::orient(for_.orientation(), g1.is_to_reverse1()?);
        let ff = for_.oriented(Orientation::Forward);
        let makecomsam = Self::g_take_common_of_same(g1)?;
        let makecomdif = Self::g_take_common_of_diff(g1)?;
        if !makecomsam && !makecomdif {
            return Ok(());
        }
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
        let lf2 = if makecomsam { &lfso2 } else { &lfdo2 };
        if lfso1.is_empty() || lf2.is_empty() {
            return Ok(());
        }
        let mut gf = TopOpeBRepBuildGTool::g_com_unsh(ShapeEnum::Face, ShapeEnum::Face);
        let c2 = if makecomsam {
            TopOpeBRepDSConfig::SameOriented
        } else {
            TopOpeBRepDSConfig::DiffOriented
        };
        gf.change_config(TopOpeBRepDSConfig::SameOriented, c2);
        let lf2 = lf2.clone();
        self.g_merge_faces(&lfso1, &lf2, &gf)?;
        let hds = self.hds()?;
        let fref = hds.shape(hds.same_domain_reference(for_), true);
        if for_.is_same(&fref) {
            for new_f in self.merged(for_, State::In).clone().iter() {
                sfs.add_start_element(&new_f.oriented(neworif))?;
            }
        }
        Ok(())
    }
}
