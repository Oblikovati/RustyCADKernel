//! The particular cases of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_KPart.cxx): the
//! detection of the faces on faces, disjoint, tangent and solid on solid cases, their
//! merges, and the classification helpers.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{TopOpeBRepBuildGTool, TopOpeBRepBuildGTopo};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// The results of the particular cases (TopOpeBRepBuild_kpresu.hxx).
pub mod kpresu {
    /// Undefined result.
    pub const RESUNDEF: i32 = -100;
    /// A new shape of the type of the second.
    pub const RESNEWSHA2: i32 = -12;
    /// A new shape of the type of the first.
    pub const RESNEWSHA1: i32 = -11;
    /// A new compound.
    pub const RESNEWCOM: i32 = -3;
    /// A new solid.
    pub const RESNEWSOL: i32 = -2;
    /// A new shell.
    pub const RESNEWSHE: i32 = -1;
    /// An empty result.
    pub const RESNULL: i32 = 0;
    /// The shell of the tangent face of the first shape.
    pub const RESSHAPE1: i32 = 1;
    /// The shell of the tangent face of the second shape.
    pub const RESSHAPE2: i32 = 2;
    /// The shapes 1 and 2.
    pub const RESSHAPE12: i32 = 3;
    /// The tangent face of the first shape.
    pub const RESFACE1: i32 = 11;
    /// The tangent face of the second shape.
    pub const RESFACE2: i32 = 12;
    /// Undefined shell selection.
    pub const SHEUNDEF: i32 = -100;
    /// No shell taken nor classified.
    pub const SHEAUCU: i32 = -1;
    /// The current shell classified.
    pub const SHECLASCOUR: i32 = 1;
    /// All the other shells classified.
    pub const SHECLASAUTR: i32 = 2;
    /// All the shells classified.
    pub const SHECLASTOUS: i32 = 3;
    /// The current shell taken without classification.
    pub const SHEGARDCOUR: i32 = 4;
    /// All the other shells taken without classification.
    pub const SHEGARDAUTR: i32 = 5;
    /// All the shells taken without classification.
    pub const SHEGARDTOUS: i32 = 6;
}

/// The number of sub-shapes of a type not in an ancestor type.
fn count(s: &Shape, t: ShapeEnum, avoid: ShapeEnum) -> Result<usize, Failure> {
    Ok(Explorer::new(s, t, avoid)?.count())
}

impl TopOpeBRepBuildBuilder {
    /// `FindIsKPart()`: the particular case of the operation (3 faces on faces, 2
    /// disjoint, 1 tangent planar, 5 tangent planar or cylindrical, 4 solids on solids,
    /// 0 none); `b.find_is_k_part()?`.
    pub fn find_is_k_part(&mut self) -> Result<i32, Failure> {
        self.kp_clear_maps();
        self.my_is_k_part = if self.kp_isfafa()? != 0 {
            3
        } else if self.kp_isdisj()? != 0 {
            2
        } else if self.kp_iskole()? != 0 {
            1
        } else if self.kp_iskoletge()? != 0 {
            5
        } else if self.kp_issoso()? != 0 {
            4
        } else {
            0
        };
        Ok(Self::kp_return(self.my_is_k_part))
    }

    /// `IsKPart()`; `b.is_k_part()`.
    pub fn is_k_part(&self) -> i32 {
        self.my_is_k_part
    }

    /// `MergeKPart(TB1, TB2)` (virtual, the same in Builder1): the states set, then MergeKPart;
    /// `b.merge_k_part_states(State::Out, State::Out)?`.
    pub fn merge_k_part_states(&mut self, tb1: State, tb2: State) -> Result<(), Failure> {
        self.my_state1 = tb1;
        self.my_state2 = tb2;
        self.merge_k_part()
    }

    /// `MergeKPart()` (virtual): the merge of the particular case, then End;
    /// `b.merge_k_part()?`.
    pub fn merge_k_part(&mut self) -> Result<(), Failure> {
        if self.b1.is_some() {
            return self.b1_merge_k_part();
        }
        self.merge_k_part_base()
    }

    /// `TopOpeBRepBuild_Builder::MergeKPart()`, the base version.
    pub(crate) fn merge_k_part_base(&mut self) -> Result<(), Failure> {
        match self.my_is_k_part {
            1 => self.merge_k_part_iskole()?,
            5 => self.merge_k_part_iskoletge()?,
            2 => self.merge_k_part_isdisj()?,
            3 => self.merge_k_part_isfafa()?,
            4 => self.merge_k_part_issoso()?,
            _ => {}
        }
        self.end()
    }

    /// The same domain table of the operation (cut, permuted cut, common, fuse), None
    /// for an undefined operation.
    fn same_table(&self) -> Option<TopOpeBRepBuildGTopo> {
        let tf = ShapeEnum::Face;
        if self.opec12() {
            Some(TopOpeBRepBuildGTool::g_cut_same(tf, tf))
        } else if self.opec21() {
            Some(TopOpeBRepBuildGTool::g_cut_same(tf, tf).copy_permuted())
        } else if self.opecom() {
            Some(TopOpeBRepBuildGTool::g_com_same(tf, tf))
        } else if self.opefus() {
            Some(TopOpeBRepBuildGTool::g_fus_same(tf, tf))
        } else {
            None
        }
    }

    /// The merged list of a compound set to the merged list of one of its shapes.
    fn compound_merged(&mut self, s: &Shape, sta: State, sub: &Shape) -> Result<(), Failure> {
        if !s.is_null() && s.shape_type()? == ShapeEnum::Compound {
            self.change_merged(s, sta);
            *self.change_merged(s, sta) = self.change_merged(sub, sta).clone();
        }
        Ok(())
    }

    /// `MergeKPartisfafa()`: the faces of the same domain merged in 2d;
    /// `b.merge_k_part_isfafa()?`.
    pub fn merge_k_part_isfafa(&mut self) -> Result<(), Failure> {
        if self.my_is_k_part != 3 {
            return Ok(());
        }
        let (s1, s2) = (self.my_shape1.clone(), self.my_shape2.clone());
        let Some(f1) = Iterator::next(&mut Explorer::new(&s1, ShapeEnum::Face, ShapeEnum::Shape)?)
        else {
            return Ok(());
        };
        let Some(f2) = Iterator::next(&mut Explorer::new(&s2, ShapeEnum::Face, ShapeEnum::Shape)?)
        else {
            return Ok(());
        };
        let (mut lf1, mut lf2) = (List::default(), List::default());
        self.g_find_sam_dom_of(&f1, &mut lf1, &mut lf2)?;
        let Some(g) = self.same_table() else {
            return Ok(());
        };
        self.g_map_shapes(&s1, &s2)?;
        self.g_merge_faces(&lf1, &lf2, &g)?;
        self.compound_merged(&s1, self.my_state1, &f1)?;
        self.compound_merged(&s2, self.my_state2, &f2)
    }

    /// `MergeKPartissoso()`: the solids of the same domain merged;
    /// `b.merge_k_part_issoso()?`.
    pub fn merge_k_part_issoso(&mut self) -> Result<(), Failure> {
        if self.my_is_k_part != 4 {
            return Ok(());
        }
        let (s1, s2) = (self.my_shape1.clone(), self.my_shape2.clone());
        let mut so1 = Shape::default();
        if !s1.is_null() {
            let Some(s) =
                Iterator::next(&mut Explorer::new(&s1, ShapeEnum::Solid, ShapeEnum::Shape)?)
            else {
                return Ok(());
            };
            so1 = s;
        }
        let mut so2 = Shape::default();
        if !s2.is_null() {
            let Some(s) =
                Iterator::next(&mut Explorer::new(&s2, ShapeEnum::Solid, ShapeEnum::Shape)?)
            else {
                return Ok(());
            };
            so2 = s;
        }
        if so1.is_null() {
            return Ok(());
        }
        let (mut lso1, mut lso2) = (List::default(), List::default());
        self.g_find_sam_dom_of(&so1, &mut lso1, &mut lso2)?;
        let Some(g) = self.same_table() else {
            return Ok(());
        };
        self.g_map_shapes(&s1, &s2)?;
        self.g_merge_solids(&lso1, &lso2, &g)?;
        self.compound_merged(&s1, self.my_state1, &so1)?;
        self.compound_merged(&s2, self.my_state2, &so2)
    }

    /// `KPisdisj()`: both shapes in the structure and disjoint;
    /// `b.kp_isdisj()?`.
    pub fn kp_isdisj(&self) -> Result<i32, Failure> {
        let hds = self.hds()?;
        let ds = hds.ds();
        if !ds.has_shape(&self.my_shape1, true) || !ds.has_shape(&self.my_shape2, true) {
            return Ok(0);
        }
        drop(ds);
        Ok(i32::from(
            self.kp_isdisjsh(&self.my_shape1)? != 0 && self.kp_isdisjsh(&self.my_shape2)? != 0,
        ))
    }

    /// `KPisfafa()`: both shapes are faces of the same domain only; `b.kp_isfafa()?`.
    pub fn kp_isfafa(&self) -> Result<i32, Failure> {
        Ok(i32::from(
            self.kp_isfafash(&self.my_shape1)? != 0 && self.kp_isfafash(&self.my_shape2)? != 0,
        ))
    }

    /// `KPissoso()`: both shapes are solids of the same domain only; `b.kp_issoso()?`.
    pub fn kp_issoso(&self) -> Result<i32, Failure> {
        Ok(i32::from(
            self.kp_issososh(&self.my_shape1)? != 0 && self.kp_issososh(&self.my_shape2)? != 0,
        ))
    }

    /// `KPClearMaps()`; `b.kp_clear_maps()`.
    pub fn kp_clear_maps(&mut self) {
        self.my_kp_map_f1f2.clear(true);
    }

    /// `KPlhg(S, T)`: the count of the sub-shapes with geometry; `b.kp_lhg(&s, ShapeEnum::Face)?`.
    pub fn kp_lhg(&self, s: &Shape, t: ShapeEnum) -> Result<i32, Failure> {
        let mut l = List::default();
        self.kp_lhg_list(s, t, &mut l)
    }

    /// `KPlhg(S, T, L)`: the sub-shapes with geometry in `l`; `b.kp_lhg_list(&s, ShapeEnum::Face, &mut l)?`.
    pub fn kp_lhg_list(
        &self,
        s: &Shape,
        t: ShapeEnum,
        l: &mut List<Shape>,
    ) -> Result<i32, Failure> {
        l.clear();
        let hds = self.hds()?;
        for x in Explorer::new(s, t, ShapeEnum::Shape)? {
            if hds.has_geometry(&x) {
                l.append(x);
            }
        }
        Ok(l.len() as i32)
    }

    /// `KPlhsd(S, T)`: the count of the sub-shapes with same domain shapes;
    /// `b.kp_lhsd(&s, ShapeEnum::Face)?`.
    pub fn kp_lhsd(&self, s: &Shape, t: ShapeEnum) -> Result<i32, Failure> {
        let mut l = List::default();
        self.kp_lhsd_list(s, t, &mut l)
    }

    /// `KPlhsd(S, T, L)`: the sub-shapes with same domain shapes in `l`;
    /// `b.kp_lhsd_list(&s, ShapeEnum::Face, &mut l)?`.
    pub fn kp_lhsd_list(
        &self,
        s: &Shape,
        t: ShapeEnum,
        l: &mut List<Shape>,
    ) -> Result<i32, Failure> {
        l.clear();
        let hds = self.hds()?;
        for x in Explorer::new(s, t, ShapeEnum::Shape)? {
            if hds.has_same_domain(&x, true) {
                l.append(x);
            }
        }
        Ok(l.len() as i32)
    }

    /// `KPclasSS(S1, exLS1, S2)`: the state of `s1` in `s2` avoiding the shapes;
    /// `b.kp_clas_ss_list(&s1, &l, &s2)?`.
    pub fn kp_clas_ss_list(
        &mut self,
        s1: &Shape,
        ex_ls1: &List<Shape>,
        s2: &Shape,
    ) -> Result<State, Failure> {
        self.my_shape_classifier
            .state_shape_shape_avoiding_list(s1, ex_ls1, s2)
    }

    /// `KPclasSS(S1, exS1, S2)`: the state of `s1` in `s2` avoiding a shape;
    /// `b.kp_clas_ss_avoiding(&s1, &ex, &s2)?`.
    pub fn kp_clas_ss_avoiding(
        &mut self,
        s1: &Shape,
        ex_s1: &Shape,
        s2: &Shape,
    ) -> Result<State, Failure> {
        self.my_shape_classifier
            .state_shape_shape_avoiding(s1, ex_s1, s2)
    }

    /// `KPclasSS(S1, S2)`: the state of `s1` in `s2`; `b.kp_clas_ss(&s1, &s2)?`.
    pub fn kp_clas_ss(&mut self, s1: &Shape, s2: &Shape) -> Result<State, Failure> {
        self.kp_clas_ss_avoiding(s1, &Shape::default(), s2)
    }

    /// `KPSameDomain(L1, L2)`: the same domain closure of the two lists (as
    /// FindSameDomain); `b.kp_same_domain(&mut l1, &mut l2)?`.
    pub fn kp_same_domain(
        &self,
        l1: &mut List<Shape>,
        l2: &mut List<Shape>,
    ) -> Result<(), Failure> {
        self.find_same_domain(l1, l2)
    }

    /// `KPisdisjsh(Sarg)`: no solid, face or edge with geometry, no solid with same
    /// domain faces nor face with same domain edges; `b.kp_isdisjsh(&s)?`.
    pub fn kp_isdisjsh(&self, sarg: &Shape) -> Result<i32, Failure> {
        if sarg.is_null() {
            return Ok(0);
        }
        for t in [ShapeEnum::Solid, ShapeEnum::Face, ShapeEnum::Edge] {
            if self.kp_lhg(sarg, t)? != 0 {
                return Ok(0);
            }
        }
        let mut lshsd = List::default();
        for (t, sub) in [
            (ShapeEnum::Solid, ShapeEnum::Face),
            (ShapeEnum::Face, ShapeEnum::Edge),
        ] {
            if self.kp_lhsd_list(sarg, t, &mut lshsd)? != 0 {
                for s in lshsd.iter() {
                    if self.kp_lhsd(s, sub)? != 0 {
                        return Ok(0);
                    }
                }
            }
        }
        Ok(1)
    }

    /// `KPissososh(Sarg)`: only solids (not in a compsolid), all with same domain
    /// solids, and no free shell, face, edge or vertex; `b.kp_issososh(&s)?`.
    pub fn kp_issososh(&self, sarg: &Shape) -> Result<i32, Failure> {
        let nsol1 = count(sarg, ShapeEnum::Solid, ShapeEnum::Shape)?;
        let nsol2 = count(sarg, ShapeEnum::Solid, ShapeEnum::CompSolid)?;
        if nsol1 != 0 && nsol1 != nsol2 {
            return Ok(0);
        }
        if self.kp_lhsd(sarg, ShapeEnum::Solid)? as usize != nsol1 {
            return Ok(0);
        }
        for (t, a) in [
            (ShapeEnum::Shell, ShapeEnum::Solid),
            (ShapeEnum::Face, ShapeEnum::Shell),
            (ShapeEnum::Edge, ShapeEnum::Wire),
            (ShapeEnum::Vertex, ShapeEnum::Edge),
        ] {
            if count(sarg, t, a)? != 0 {
                return Ok(0);
            }
        }
        Ok(1)
    }

    /// `KPisfafash(Sarg)`: only faces (in shells), all with same domain faces, and no
    /// free wire, edge or vertex; `b.kp_isfafash(&s)?`.
    pub fn kp_isfafash(&self, sarg: &Shape) -> Result<i32, Failure> {
        let nfac1 = count(sarg, ShapeEnum::Face, ShapeEnum::Shape)?;
        let nfac2 = count(sarg, ShapeEnum::Face, ShapeEnum::Shell)?;
        if nfac1 != 0 && nfac1 != nfac2 {
            return Ok(0);
        }
        if self.kp_lhsd(sarg, ShapeEnum::Face)? as usize != nfac1 {
            return Ok(0);
        }
        for (t, a) in [
            (ShapeEnum::Wire, ShapeEnum::Face),
            (ShapeEnum::Edge, ShapeEnum::Wire),
            (ShapeEnum::Vertex, ShapeEnum::Edge),
        ] {
            if count(sarg, t, a)? != 0 {
                return Ok(0);
            }
        }
        Ok(1)
    }

    /// `KPls(S, T)`: the count of the sub-shapes of a type; `TopOpeBRepBuildBuilder::kp_ls(&s, ShapeEnum::Edge)?`.
    pub fn kp_ls(s: &Shape, t: ShapeEnum) -> Result<i32, Failure> {
        let mut l = List::default();
        Self::kp_ls_list(s, t, &mut l)
    }

    /// `KPls(S, T, L)`: the sub-shapes of a type in `l`;
    /// `TopOpeBRepBuildBuilder::kp_ls_list(&s, ShapeEnum::Edge, &mut l)?`.
    pub fn kp_ls_list(s: &Shape, t: ShapeEnum, l: &mut List<Shape>) -> Result<i32, Failure> {
        l.clear();
        for x in Explorer::new(s, t, ShapeEnum::Shape)? {
            l.append(x);
        }
        Ok(l.len() as i32)
    }

    /// `KPclassF(F1, F2)`: the state of the first edge of the forward face in the
    /// forward other face of the same domain (UNKNOWN without edge or face);
    /// `b.kp_class_f(&f1, &f2)?`.
    pub fn kp_class_f(&mut self, f1: &Shape, f2: &Shape) -> Result<State, Failure> {
        if f1.is_null() || f2.is_null() {
            return Ok(State::Unknown);
        }
        let (f1f, f2f) = (
            f1.oriented(Orientation::Forward),
            f2.oriented(Orientation::Forward),
        );
        let mut le1 = List::default();
        if Self::kp_ls_list(&f1f, ShapeEnum::Edge, &mut le1)? == 0 {
            return Ok(State::Unknown);
        }
        let e1 = le1.first().cloned().unwrap_or_default();
        self.my_shape_classifier.state_shape_shape(&e1, &f2f, 1)
    }

    /// `KPclassFF(F1, F2, St1, St2)`: the states of each face in the other;
    /// `let (s1, s2) = b.kp_class_ff(&f1, &f2)?;`.
    pub fn kp_class_ff(&mut self, f1: &Shape, f2: &Shape) -> Result<(State, State), Failure> {
        Ok((self.kp_class_f(f1, f2)?, self.kp_class_f(f2, f1)?))
    }

    /// `KPiskoleFF(F1, F2, St1, St2)`: the faces are IN/OUT of each other in opposite
    /// states, with the states; `let (ok, s1, s2) = b.kp_iskole_ff(&f1, &f2)?;`.
    pub fn kp_iskole_ff(
        &mut self,
        f1: &Shape,
        f2: &Shape,
    ) -> Result<(bool, State, State), Failure> {
        let (st1, st2) = self.kp_class_ff(f1, f2)?;
        let ok = |s: State| s == State::Out || s == State::In;
        Ok((ok(st1) && ok(st2) && st1 != st2, st1, st2))
    }

    /// `KPContains(S, L)`: as Contains; `TopOpeBRepBuildBuilder::kp_contains(&s, &l)`.
    pub fn kp_contains(s: &Shape, l: &List<Shape>) -> bool {
        Self::contains(s, l)
    }

    /// `KPreturn(b)`: `b`; `TopOpeBRepBuildBuilder::kp_return(1)`.
    pub fn kp_return(b: i32) -> i32 {
        b
    }
}
