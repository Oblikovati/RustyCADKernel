//! The regularizations of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_FREGU.cxx and
//! TopOpeBRepBuild_SREGU.cxx): the new faces (solids) split by their regularized wires
//! (shells), the splits of the edges (faces) of the same domain updated.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{TopOpeBRepBuildShellToSolid, TopOpeBRepBuildWireToFace};
use crate::top_ope_brep_tool::{ShapeListMap, regularize_shells, regularize_wires};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

impl TopOpeBRepBuildBuilder {
    /// The shapes of the same domain closure of `s`.
    fn same_domain_closure(&self, s: &Shape) -> Result<List<Shape>, Failure> {
        let (mut l1, mut l2) = (List::default(), List::default());
        self.g_find_sam_dom_of(s, &mut l1, &mut l2)?;
        l1.append_list(&mut l2);
        Ok(l1)
    }

    /// The state of the operation of the shape of a rank (the first for rank 1).
    fn rank_state(&self, s: &Shape) -> State {
        if self.g_shape_rank(s) == 1 {
            self.my_state1
        } else {
            self.my_state2
        }
    }

    /// The splits of `e` on the state with the memorized splits replaced by their own.
    fn replace_memo_splits(&mut self, e: &Shape, sta: State) {
        let lsp = self.change_split(e, sta).clone();
        let mut newl = List::default();
        for sp in lsp.iter() {
            if !self.my_memo_split.contains(sp) {
                newl.append(sp.clone());
            } else {
                let l = self.splits(sp, sta).clone();
                Self::g_copy_list(&l, &mut newl);
            }
        }
        let target = self.change_split(e, sta);
        target.clear();
        Self::g_copy_list(&newl, target);
    }

    /// `RegularizeFaces(FF, lnewFace, LOF)`: the regularized faces of the new faces,
    /// the splits of the edges of the faces of the same domain of `ff` updated with the
    /// memorized splits; `b.regularize_faces(&f, &lnf, &mut lof)?`.
    pub fn regularize_faces(
        &mut self,
        ff: &Shape,
        lnew_face: &List<Shape>,
        lof: &mut List<Shape>,
    ) -> Result<(), Failure> {
        lof.clear();
        self.my_memo_split.clear(true);
        for new_face in lnew_face.iter() {
            let mut l = List::default();
            self.regularize_face(ff, new_face, &mut l)?;
            lof.append_list(&mut l);
        }
        if self.my_memo_split.is_empty() {
            return Ok(());
        }
        for fsd_ff in self.same_domain_closure(ff)?.iter() {
            for e in Explorer::new(fsd_ff, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let staeope = self.rank_state(&e);
                for stae in [staeope, State::On] {
                    if self.is_split(&e, stae) {
                        self.replace_memo_splits(&e, stae);
                    }
                }
            }
        }
        Ok(())
    }

    /// The splits of the split shape `s` on the state, iterated live (as natively): each
    /// one of `kept` with resplits gets them appended and is memorized.
    fn apply_resplits(
        &mut self,
        s: &Shape,
        sta: State,
        kept: &Map<Shape, ShapeMapHasher>,
        resplits: &ShapeListMap,
    ) {
        let mut k = 0;
        while let Some(sp) = self.change_split(s, sta).iter().nth(k).cloned() {
            k += 1;
            if !kept.contains(&sp) {
                continue;
            }
            if let Ok(lre) = resplits.find(&sp) {
                let lre = lre.clone();
                self.my_memo_split.add(sp.clone());
                Self::g_copy_list(&lre, self.change_split(&sp, sta));
            }
        }
    }

    /// The new faces of the wires of the regularized wires (the old wire when kept).
    fn faces_of_wires(new_face: &Shape, ownw: &ShapeListMap) -> Result<List<Shape>, Failure> {
        let mut wtof = TopOpeBRepBuildWireToFace::new();
        for (ow, lw) in ownw.items() {
            if lw.is_empty() {
                wtof.add_wire(&topods::wire(ow)?);
            }
            for w in lw.iter() {
                wtof.add_wire(&topods::wire(w)?);
            }
        }
        let mut newfaces = List::default();
        wtof.make_faces(&topods::face(new_face)?, &mut newfaces)?;
        Ok(newfaces)
    }

    /// `RegularizeFace(FF, newFace, LOF)`: the faces of the regularized wires of the new
    /// face (the new face when not regularized), the splits of the edges of the faces of
    /// the same domain of `ff` in the new face extended by their resplits;
    /// `b.regularize_face(&f, &nf, &mut lof)?`.
    pub fn regularize_face(
        &mut self,
        ff: &Shape,
        anew_face: &Shape,
        lof: &mut List<Shape>,
    ) -> Result<(), Failure> {
        lof.clear();
        let new_face = topods::face(anew_face)?;
        let mut ownw = ShapeListMap::default();
        self.my_e_splits.clear(true);
        if !regularize_wires(&new_face, &mut ownw, &mut self.my_e_splits)? {
            lof.append(anew_face.clone());
            return Ok(());
        }
        let newfaces = Self::faces_of_wires(anew_face, &ownw)?;
        if newfaces.is_empty() {
            lof.append(anew_face.clone());
            return Ok(());
        }
        for f in newfaces.iter() {
            lof.append(f.clone());
        }
        let mut menf: Map<Shape, ShapeMapHasher> = Map::default();
        for e in Explorer::new(anew_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            menf.add(e);
        }
        let resplits = self.my_e_splits.clone();
        for fsd_ff in self.same_domain_closure(ff)?.iter() {
            let sta = self.rank_state(fsd_ff);
            for e in Explorer::new(fsd_ff, ShapeEnum::Edge, ShapeEnum::Shape)? {
                for stae in [sta, State::On] {
                    self.apply_resplits(&e, stae, &menf, &resplits);
                }
            }
        }
        Ok(())
    }

    /// `RegularizeSolids(SO, lnewSolid, LOSO)`: the regularized solids of the new
    /// solids, the splits of the faces of the solids of the same domain of `so` updated
    /// with the memorized splits; `b.regularize_solids(&so, &lns, &mut loso)?`.
    pub fn regularize_solids(
        &mut self,
        so: &Shape,
        lnew_solid: &List<Shape>,
        loso: &mut List<Shape>,
    ) -> Result<(), Failure> {
        loso.clear();
        self.my_memo_split.clear(true);
        for new_solid in lnew_solid.iter() {
            let mut l = List::default();
            self.regularize_solid(so, new_solid, &mut l)?;
            loso.append_list(&mut l);
        }
        if self.my_memo_split.is_empty() {
            return Ok(());
        }
        for sosd in self.same_domain_closure(so)?.iter() {
            for f in Explorer::new(sosd, ShapeEnum::Face, ShapeEnum::Shape)? {
                let staf = self.rank_state(&f);
                if self.is_split(&f, staf) {
                    self.replace_memo_splits_changed(&f, staf);
                }
            }
        }
        Ok(())
    }

    /// The splits of `f` with the memorized splits replaced by their split lists (the
    /// mutable ones, bound when missing, as natively for the solids).
    fn replace_memo_splits_changed(&mut self, f: &Shape, sta: State) {
        let lsp = self.change_split(f, sta).clone();
        let mut newl = List::default();
        for sp in lsp.iter() {
            if !self.my_memo_split.contains(sp) {
                newl.append(sp.clone());
            } else {
                let l = self.change_split(sp, sta).clone();
                Self::g_copy_list(&l, &mut newl);
            }
        }
        let target = self.change_split(f, sta);
        target.clear();
        Self::g_copy_list(&newl, target);
    }

    /// `RegularizeSolid(SS, newSolid, LOSO)`: the solids of the regularized shells of
    /// the new solid (the new solid when not regularized), the splits of the faces of
    /// the solids of the same domain of `ss` in the new solid extended by their
    /// resplits; `b.regularize_solid(&so, &ns, &mut loso)?`.
    pub fn regularize_solid(
        &mut self,
        ss: &Shape,
        anew_solid: &Shape,
        loso: &mut List<Shape>,
    ) -> Result<(), Failure> {
        loso.clear();
        let new_solid = topods::solid(anew_solid)?;
        let mut osns = ShapeListMap::default();
        self.my_f_splits.clear(true);
        if !regularize_shells(&new_solid, &mut osns, &mut self.my_f_splits)? {
            loso.append(anew_solid.clone());
            return Ok(());
        }
        let mut stos = TopOpeBRepBuildShellToSolid::new();
        for (_, lns) in osns.items() {
            for sh in lns.iter() {
                stos.add_shell(&topods::shell(sh)?);
            }
        }
        let mut new_solids = List::default();
        stos.make_solids(&new_solid, &mut new_solids)?;
        if new_solids.is_empty() {
            loso.append(anew_solid.clone());
            return Ok(());
        }
        for s in new_solids.iter() {
            loso.append(s.clone());
        }
        let mut mfns: Map<Shape, ShapeMapHasher> = Map::default();
        for f in Explorer::new(anew_solid, ShapeEnum::Face, ShapeEnum::Shape)? {
            mfns.add(f);
        }
        let resplits = self.my_f_splits.clone();
        for ssd in self.same_domain_closure(ss)?.iter() {
            let sta = self.rank_state(ssd);
            for f in Explorer::new(ssd, ShapeEnum::Face, ShapeEnum::Shape)? {
                self.apply_resplits(&f, sta, &mfns, &resplits);
            }
        }
        Ok(())
    }
}
