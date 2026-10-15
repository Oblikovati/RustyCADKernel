//! The general helpers of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_Grid.cxx,
//! TopOpeBRepBuild_keep.cxx and the release forms of TopOpeBRepBuild_Griddump.cxx):
//! the split and merge tests, the same domain lists, the ranks and the kept shapes.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildEdgeBuilder, TopOpeBRepBuildFaceBuilder, TopOpeBRepBuildGTopo,
    TopOpeBRepBuildShapeSet, TopOpeBRepBuildSolidBuilder,
};
use crate::top_ope_brep_ds::TopOpeBRepDSDumper;
use crate::top_ope_brep_tool::TopOpeBRepToolShapeExplorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tkmath::gp::Point3;

impl TopOpeBRepBuildBuilder {
    /// `GToSplit(S, TB)`: not split yet and a face of the structure, with geometry or
    /// with same domain shapes; `b.g_to_split(&f, State::In)?`.
    pub fn g_to_split(&self, s: &Shape, tb: State) -> Result<bool, Failure> {
        let hds = self.hds()?;
        let facshap = hds.has_shape(s, true) && s.shape_type()? == ShapeEnum::Face;
        Ok(!self.is_split(s, tb)
            && (facshap || hds.has_geometry(s) || hds.has_same_domain(s, true)))
    }

    /// `GToMerge(S)`: not merged IN and with same domain shapes; `b.g_to_merge(&f)?`.
    pub fn g_to_merge(&self, s: &Shape) -> Result<bool, Failure> {
        Ok(!self.is_merged(s, State::In) && self.hds()?.has_same_domain(s, true))
    }

    /// `GTakeCommonOfSame(G)`: the ON states are OUT/OUT or IN/IN;
    /// `TopOpeBRepBuildBuilder::g_take_common_of_same(&g)?`.
    pub fn g_take_common_of_same(g: &TopOpeBRepBuildGTopo) -> Result<bool, Failure> {
        let (t1, t2) = g.states_on()?;
        Ok(matches!(
            (t1, t2),
            (State::Out, State::Out) | (State::In, State::In)
        ))
    }

    /// `GTakeCommonOfDiff(G)`: the ON states are OUT/IN or IN/OUT;
    /// `TopOpeBRepBuildBuilder::g_take_common_of_diff(&g)?`.
    pub fn g_take_common_of_diff(g: &TopOpeBRepBuildGTopo) -> Result<bool, Failure> {
        let (t1, t2) = g.states_on()?;
        Ok(matches!(
            (t1, t2),
            (State::Out, State::In) | (State::In, State::Out)
        ))
    }

    /// `GFindSamDom(S, L1, L2)`: the same domain closure of `s`;
    /// `b.g_find_sam_dom_of(&f, &mut l1, &mut l2)?`.
    pub fn g_find_sam_dom_of(
        &self,
        s: &Shape,
        l1: &mut List<Shape>,
        l2: &mut List<Shape>,
    ) -> Result<(), Failure> {
        l1.clear();
        l2.clear();
        l1.append(s.clone());
        self.g_find_sam_dom(l1, l2)
    }

    /// `GFindSamDom(L1, L2)`: the same domain closure of the two lists (as
    /// FindSameDomain); `b.g_find_sam_dom(&mut l1, &mut l2)?`.
    pub fn g_find_sam_dom(
        &self,
        l1: &mut List<Shape>,
        l2: &mut List<Shape>,
    ) -> Result<(), Failure> {
        self.find_same_domain(l1, l2)
    }

    /// `GFindSamDomSODO(S, LSO, LDO)`: the same domain closure of `s` split by the
    /// orientation of `s`; `b.g_find_sam_dom_sodo_of(&f, &mut lso, &mut ldo)?`.
    pub fn g_find_sam_dom_sodo_of(
        &self,
        s: &Shape,
        lso: &mut List<Shape>,
        ldo: &mut List<Shape>,
    ) -> Result<(), Failure> {
        lso.clear();
        ldo.clear();
        lso.append(s.clone());
        self.g_find_sam_dom_sodo(lso, ldo)
    }

    /// `GFindSamDomSODO(LSO, LDO)`: the same domain closure split into the shapes of the
    /// orientation of the first shape of `lso` (taken before the closure) and the
    /// others; `b.g_find_sam_dom_sodo(&mut lso, &mut ldo)?`.
    pub fn g_find_sam_dom_sodo(
        &self,
        lso: &mut List<Shape>,
        ldo: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let Some(sref) = lso.first().cloned() else {
            return Ok(());
        };
        let hds = self.hds()?;
        let oref = hds.same_domain_orientation(&sref);
        self.g_find_sam_dom(lso, ldo)?;
        let (mut llso, mut lldo) = (List::default(), List::default());
        for s in lso.iter().chain(ldo.iter()) {
            let o = hds.same_domain_orientation(s);
            if o == oref && !Self::g_contains(s, &llso) {
                llso.append(s.clone());
            } else if o != oref && !Self::g_contains(s, &lldo) {
                lldo.append(s.clone());
            }
        }
        (*lso, *ldo) = (llso, lldo);
        Ok(())
    }

    /// `GMapShapes(S1, S2)`: as MapShapes; `b.g_map_shapes(&s1, &s2)?`.
    pub fn g_map_shapes(&mut self, s1: &Shape, s2: &Shape) -> Result<(), Failure> {
        self.map_shapes(s1, s2)
    }

    /// `GClearMaps()`; `b.g_clear_maps()`.
    pub fn g_clear_maps(&mut self) {
        self.clear_maps();
    }

    /// `GFindSameRank(L1, Rank, L2)`: as FindSameRank; `b.g_find_same_rank(&l1, 1, &mut l2)`.
    pub fn g_find_same_rank(&self, l1: &List<Shape>, rank: i32, l2: &mut List<Shape>) {
        for s in l1.iter() {
            if self.g_shape_rank(s) == rank && !Self::g_contains(s, l2) {
                l2.append(s.clone());
            }
        }
    }

    /// `GShapeRank(S)`: 1 or 2 for a sub-shape of the first or second shape, else 0;
    /// `b.g_shape_rank(&s)`.
    pub fn g_shape_rank(&self, s: &Shape) -> i32 {
        if self.g_is_shape_of(s, 1) {
            1
        } else if self.g_is_shape_of(s, 2) {
            2
        } else {
            0
        }
    }

    /// `GIsShapeOf(S, I)`: as IsShapeOf, false for a null shape; `b.g_is_shape_of(&s, 1)`.
    pub fn g_is_shape_of(&self, s: &Shape, i: i32) -> bool {
        !s.is_null() && self.is_shape_of(s, i)
    }

    /// `GContains(S, L)`: as Contains; `TopOpeBRepBuildBuilder::g_contains(&s, &l)`.
    pub fn g_contains(s: &Shape, l: &List<Shape>) -> bool {
        Self::contains(s, l)
    }

    /// `GCopyList(Lin, I1, I2, Lou)`: the shapes of ranks `i1` to `i2` (from 1) of
    /// `lin` appended to `lou`; `TopOpeBRepBuildBuilder::g_copy_list_range(&l, 1, 2, &mut lou)`.
    pub fn g_copy_list_range(lin: &List<Shape>, i1: i32, i2: i32, lou: &mut List<Shape>) {
        for (i, el) in (1..).zip(lin.iter()) {
            if i >= i1 && i <= i2 {
                lou.append(el.clone());
            }
        }
    }

    /// `GCopyList(Lin, Lou)`: the shapes of `lin` appended to `lou`;
    /// `TopOpeBRepBuildBuilder::g_copy_list(&l, &mut lou)`.
    pub fn g_copy_list(lin: &List<Shape>, lou: &mut List<Shape>) {
        Self::g_copy_list_range(lin, 1, lin.len() as i32, lou);
    }

    /// `GKeepShape(S, LSclass, TB)`: `GKeepShape1` without the position;
    /// `b.g_keep_shape(&f, &l, State::In)?`.
    pub fn g_keep_shape(
        &mut self,
        s: &Shape,
        lsclass: &List<Shape>,
        tb: State,
    ) -> Result<bool, Failure> {
        Ok(self.g_keep_shape1(s, lsclass, tb)?.0)
    }

    /// `GKeepShape1(S, LSclass, TB, pos)`: kept when not classified (no shape to
    /// classify with, or a face out of the structure with classification off) or
    /// positioned on `tb`; the position (UNKNOWN when not classified);
    /// `let (keep, pos) = b.g_keep_shape1(&f, &l, State::In)?;`.
    pub fn g_keep_shape1(
        &mut self,
        s: &Shape,
        lsclass: &List<Shape>,
        tb: State,
    ) -> Result<(bool, State), Failure> {
        let mut toclassify = true;
        if s.shape_type()? == ShapeEnum::Face
            && !self.hds()?.has_shape(s, true)
            && self.my_classify_def
        {
            toclassify = self.my_classify_val;
        }
        if !(toclassify && !lsclass.is_empty()) {
            return Ok((true, State::Unknown));
        }
        let pos = self.shape_position(s, lsclass)?;
        Ok((pos == tb, pos))
    }

    /// `GKeepShapes(S, LSclass, TB, Lin, Lou)`: the shapes of `lin` positioned on `tb`
    /// against `lsclass` (all when empty) appended to `lou`;
    /// `b.g_keep_shapes(&s, &lc, State::In, &lin, &mut lou)?`.
    pub fn g_keep_shapes(
        &mut self,
        _s: &Shape,
        lsclass: &List<Shape>,
        tb: State,
        lin: &List<Shape>,
        lou: &mut List<Shape>,
    ) -> Result<(), Failure> {
        for sl in lin.iter() {
            let keep = lsclass.is_empty() || self.shape_position(sl, lsclass)? == tb;
            if keep {
                lou.append(sl.clone());
            }
        }
        Ok(())
    }

    /// `GdumpSAMDOM(L, str)`: the shapes with their reference and orientation printed;
    /// `b.gdump_samdom(&l, "")?`.
    pub fn gdump_samdom(&self, l: &List<Shape>, astr: &str) -> Result<(), Failure> {
        let dumper = TopOpeBRepDSDumper::new(&self.my_data_structure);
        println!("{}", dumper.sprint_shape_ref_ori_list(l, astr)?);
        Ok(())
    }

    /// `GdumpLS(L)`: nothing in a release build; `b.gdump_ls(&l)`.
    pub fn gdump_ls(&self, _l: &List<Shape>) {}

    /// `GdumpPNT(P)`: nothing in a release build; `TopOpeBRepBuildBuilder::gdump_pnt(&p)`.
    pub fn gdump_pnt(_p: &Point3) {}

    /// `GdumpORIPARPNT(o, p, Pnt)`: nothing in a release build;
    /// `TopOpeBRepBuildBuilder::gdump_oriparpnt(Orientation::Forward, 0., &p)`.
    pub fn gdump_oriparpnt(_o: Orientation, _p: f64, _pnt: &Point3) {}

    /// `GdumpSHA(S, str)`: nothing in a release build; `b.gdump_sha(&s, "")`.
    pub fn gdump_sha(&self, _s: &Shape, _str: &str) {}

    /// `GdumpSHAORI(S, str)`: nothing in a release build; `b.gdump_shaori(&s, "")`.
    pub fn gdump_shaori(&self, _s: &Shape, _str: &str) {}

    /// `GdumpSHAORIGEO(S, str)`: nothing in a release build; `b.gdump_shaorigeo(&s, "")`.
    pub fn gdump_shaorigeo(&self, _s: &Shape, _str: &str) {}

    /// `GdumpSHASTA(iS, T, a, b)`: nothing in a release build; `b.gdump_shasta_index(1, State::In, "", "")`.
    pub fn gdump_shasta_index(&self, _is: i32, _t: State, _a: &str, _b: &str) {}

    /// `GdumpSHASTA(S, T, a, b)`: nothing in a release build; `b.gdump_shasta(&s, State::In, "", "")`.
    pub fn gdump_shasta(&self, _s: &Shape, _t: State, _a: &str, _b: &str) {}

    /// `GdumpSHASTA(iS, T, SS, a, b, c)`: nothing in a release build;
    /// `b.gdump_shasta_set(1, State::In, &ss, "", "", "")`.
    pub fn gdump_shasta_set(
        &self,
        _is: i32,
        _t: State,
        _ss: &dyn TopOpeBRepBuildShapeSet,
        _a: &str,
        _b: &str,
        _c: &str,
    ) {
    }

    /// `GdumpEDG(S, str)`: nothing in a release build; `b.gdump_edg(&e, "")`.
    pub fn gdump_edg(&self, _s: &Shape, _str: &str) {}

    /// `GdumpEDGVER(E, V, str)`: nothing in a release build; `b.gdump_edgver(&e, &v, "")`.
    pub fn gdump_edgver(&self, _e: &Shape, _v: &Shape, _str: &str) {}

    /// `GdumpEXP(E)`: nothing in a release build; `b.gdump_exp(&ex)`.
    pub fn gdump_exp(&self, _e: &TopOpeBRepToolShapeExplorer) {}

    /// `GdumpSOBU(SB)`: nothing in a release build; `b.gdump_sobu(&mut sb)`.
    pub fn gdump_sobu(&self, _sb: &mut TopOpeBRepBuildSolidBuilder) {}

    /// `GdumpFABU(FB)`: nothing in a release build; `b.gdump_fabu(&mut fb)`.
    pub fn gdump_fabu(&self, _fb: &mut TopOpeBRepBuildFaceBuilder) {}

    /// `GdumpEDBU(EB)`: nothing in a release build; `b.gdump_edbu(&mut eb)`.
    pub fn gdump_edbu(&self, _eb: &mut TopOpeBRepBuildEdgeBuilder) {}

    /// `GtraceSPS(iS)`: false in a release build; `b.gtrace_sps_index(1)`.
    pub fn gtrace_sps_index(&self, _is: i32) -> bool {
        false
    }

    /// `GtraceSPS(iS, jS)`: false; `b.gtrace_sps_pair(1, 2)`.
    pub fn gtrace_sps_pair(&self, _is: i32, _js: i32) -> bool {
        false
    }

    /// `GtraceSPS(S)`: false in a release build; `b.gtrace_sps(&s)`.
    pub fn gtrace_sps(&self, _s: &Shape) -> bool {
        false
    }

    /// `GtraceSPS(S, IS)`: false with the index 0; `let (b, is) = b.gtrace_sps_with_index(&s);`.
    pub fn gtrace_sps_with_index(&self, _s: &Shape) -> (bool, i32) {
        (false, 0)
    }

    /// `GdumpSHASETreset()`: nothing in a release build; `b.gdump_shaset_reset()`.
    pub fn gdump_shaset_reset(&mut self) {}

    /// `GdumpSHASETindex()`: 0 in a release build; `b.gdump_shaset_index()`.
    pub fn gdump_shaset_index(&mut self) -> i32 {
        0
    }

    /// `PrintGeo(S)`: nothing in a release build; `TopOpeBRepBuildBuilder::print_geo(&s)`.
    pub fn print_geo(_s: &Shape) {}

    /// `PrintSur(F)`: nothing in a release build; `TopOpeBRepBuildBuilder::print_sur(&f)`.
    pub fn print_sur(_f: &Face) {}

    /// `PrintCur(E)`: nothing in a release build; `TopOpeBRepBuildBuilder::print_cur(&e)`.
    pub fn print_cur(_e: &Edge) {}

    /// `PrintPnt(V)`: nothing in a release build; `TopOpeBRepBuildBuilder::print_pnt(&v)`.
    pub fn print_pnt(_v: &Vertex) {}

    /// `PrintOri(S)`: nothing in a release build; `TopOpeBRepBuildBuilder::print_ori(&s)`.
    pub fn print_ori(_s: &Shape) {}

    /// `StringState(S)`: empty in a release build; `TopOpeBRepBuildBuilder::string_state(State::In)`.
    pub fn string_state(_s: State) -> String {
        String::new()
    }

    /// `GcheckNBOUNDS(E)`: false in a release build; `TopOpeBRepBuildBuilder::gcheck_nbounds(&e)`.
    pub fn gcheck_nbounds(_e: &Shape) -> bool {
        false
    }
}
