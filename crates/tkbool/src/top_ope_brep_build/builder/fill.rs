//! The fills of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_Builder.cxx): the shapes of
//! the same domain, the positions of shapes, the split or kept sub-shapes added to
//! the shape sets and the paves of the edges.
use super::TopOpeBRepBuildBuilder;
use super::perform::substitute_coincident_edges;
use super::split::STATIC_SOLIDINDEX;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildPave, TopOpeBRepBuildPaveSet, TopOpeBRepBuildShapeSet,
    TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::{
    TopOpeBRepDSConfig, TopOpeBRepDSDataStructure, TopOpeBRepDSPointIterator,
};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeExplorer;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use rustycad_tktopalgo::brep_class3d::BRepClass3dSolidClassifier;

/// `FUN_touched(BDS, EOR)` (static): a vertex of the edge is in the data structure.
fn fun_touched(bds: &TopOpeBRepDSDataStructure, eor: &Shape) -> Result<bool, Failure> {
    let (vf, vl) = topexp::vertices(&topods::edge(eor)?, false);
    Ok(bds.has_shape(vf.as_ref(), true) || bds.has_shape(vl.as_ref(), true))
}

impl TopOpeBRepBuildBuilder {
    /// The shapes of the same domain of the first `n` shapes of `from` prepended to `to`
    /// when missing; the count of the shapes added.
    fn add_same_domain(
        &self,
        from: &List<Shape>,
        n: usize,
        to: &mut List<Shape>,
    ) -> Result<usize, Failure> {
        let hds = self.hds()?;
        let mut added = 0;
        for s1 in from.iter().take(n) {
            for s2 in hds.same_domain(s1).iter() {
                if !Self::contains(s2, to) {
                    to.prepend(s2.clone());
                    added += 1;
                }
            }
        }
        Ok(added)
    }

    /// `FindSameDomain(L1, L2)`: the closure of the same domain shapes of the two lists,
    /// each new shape prepended to the other list; `b.find_same_domain(&mut l1, &mut l2)?`.
    pub fn find_same_domain(
        &self,
        l1: &mut List<Shape>,
        l2: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let (mut nl1, mut nl2) = (l1.len(), l2.len());
        while nl1 > 0 || nl2 > 0 {
            nl2 += self.add_same_domain(l1, nl1, l2)?;
            nl1 = 0;
            nl1 += self.add_same_domain(l2, nl2, l1)?;
            nl2 = 0;
        }
        Ok(())
    }

    /// `FindSameDomainSameOrientation(L1, L2)`: the same domain closure split into the
    /// shapes of the orientation of the first shape of `l1` and the others;
    /// `b.find_same_domain_same_orientation(&mut l1, &mut l2)?`.
    pub fn find_same_domain_same_orientation(
        &self,
        l1: &mut List<Shape>,
        l2: &mut List<Shape>,
    ) -> Result<(), Failure> {
        self.find_same_domain(l1, l2)?;
        let Some(sref) = l1.first().cloned() else {
            return Ok(());
        };
        let hds = self.hds()?;
        let oref = hds.same_domain_orientation(&sref);
        let (mut ll1, mut ll2) = (List::default(), List::default());
        for s in l1.iter().chain(l2.iter()) {
            let o = hds.same_domain_orientation(s);
            if o == oref && !Self::contains(s, &ll1) {
                ll1.append(s.clone());
            } else if o != oref && !Self::contains(s, &ll2) {
                ll2.append(s.clone());
            }
        }
        (*l1, *l2) = (ll1, ll2);
        Ok(())
    }

    /// `ShapePosition(S, LS)`: the first state of `s` against a shape of `ls` other than
    /// OUT and UNKNOWN (the edges to avoid for a face), UNKNOWN when the last is unknown,
    /// else OUT; `b.shape_position(&f, &l)?`.
    pub fn shape_position(&mut self, s: &Shape, ls: &List<Shape>) -> Result<State, Failure> {
        let mut state = State::Unknown;
        let plos = if s.shape_type()? == ShapeEnum::Face {
            self.my_edge_avoid.clone()
        } else {
            self.my_empty_shape_list.clone()
        };
        for sls in ls.iter() {
            state = self
                .my_shape_classifier
                .state_shape_shape_avoiding_list(s, &plos, sls)?;
            if state != State::Out && state != State::Unknown {
                return Ok(state);
            }
        }
        Ok(if state == State::Unknown {
            state
        } else {
            State::Out
        })
    }

    /// `KeepShape(S1, LS2, ToBuild1)`: no shape in `ls2` or `s1` positioned on the state
    /// against them; `b.keep_shape(&s, &l, State::In)?`.
    pub fn keep_shape(
        &mut self,
        s1: &Shape,
        ls2: &List<Shape>,
        to_build1: State,
    ) -> Result<bool, Failure> {
        if ls2.is_empty() {
            return Ok(true);
        }
        Ok(self.shape_position(s1, ls2)? == to_build1)
    }

    /// The splits of a split shape added as start elements (the ON splits of an edge
    /// without IN split, kept by the faces in 2d), but for a consumed face edge.
    fn add_split_parts(
        &mut self,
        a_shape: &Shape,
        to_build1: State,
        newori: Orientation,
        a_set: &mut dyn TopOpeBRepBuildShapeSet,
        consumed: bool,
    ) -> Result<(), Failure> {
        let ls = self.splits(a_shape, to_build1).clone();
        let is_lson =
            a_shape.shape_type()? == ShapeEnum::Edge && to_build1 == State::In && ls.is_empty();
        let parts = if is_lson {
            self.splits(a_shape, State::On).clone()
        } else {
            ls
        };
        for part in parts.iter() {
            if consumed {
                continue;
            }
            let mut new_shape =
                substitute_coincident_edges(part, &self.my_coincident_edges, &self.my_build_tool)?;
            self.my_build_tool.set_orientation(&mut new_shape, newori);
            let mut add = true;
            if is_lson && !self.my_list_of_face.is_empty() {
                let lof = self.my_list_of_face.clone();
                add = self.keep_shape(&new_shape, &lof, to_build1)?;
            }
            if add {
                a_set.add_start_element(&new_shape)?;
            }
        }
        Ok(())
    }

    /// Whether an edge not split, without geometry, is kept: by the faces in 2d, else by
    /// the state of a point of its 3d curve in the other solid.
    fn keep_unsplit_edge(&mut self, a_shape: &Shape, to_build1: State) -> Result<bool, Failure> {
        if !self.my_list_of_face.is_empty() {
            let lof = self.my_list_of_face.clone();
            return self.keep_shape(a_shape, &lof, to_build1);
        }
        let sol = if STATIC_SOLIDINDEX.get() == 1 {
            &self.my_shape2
        } else {
            &self.my_shape1
        };
        if sol.is_null() {
            return Ok(true);
        }
        let (c3d, first, last) = BRepTool::curve(&topods::edge(a_shape)?)?;
        let Some(c3d) = c3d.get() else {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                "SplitShapes no 3D curve on edge",
                "TopOpeBRepBuild_Builder",
            ));
        };
        let tt = 0.127956477;
        let par = (1. - tt) * first + tt * last;
        let p3d = c3d.value(par)?;
        let scl = BRepClass3dSolidClassifier::with_point(sol, &p3d, precision::confusion())?;
        Ok(scl.state() == to_build1)
    }

    /// Whether an unsplit shape is added: an edge in the structure without geometry, or
    /// touched by a vertex, is tested by `keep_unsplit_edge`.
    fn add_unsplit(&mut self, a_shape: &Shape, to_build1: State) -> Result<bool, Failure> {
        let hds = self.hds()?;
        let isedge = a_shape.shape_type()? == ShapeEnum::Edge;
        let (hs, hg) = (hds.has_shape(a_shape, true), hds.has_geometry(a_shape));
        let mut testkeep = isedge && hs && !hg;
        let mut istouched = isedge && !hs && !hg;
        if istouched {
            istouched = fun_touched(&hds.ds(), a_shape)?;
        }
        testkeep = testkeep || istouched;
        if testkeep {
            self.keep_unsplit_edge(a_shape, to_build1)
        } else {
            Ok(true)
        }
    }

    /// `SplitShapes(Ex, ToBuild1, ToBuild2, aSet, RevOri)`: each explored solid, shell,
    /// face or edge split, its splits added as start elements, or the shape itself,
    /// when kept, as an element (oriented, with its coincident edges substituted);
    /// `b.split_shapes(&mut ex, State::In, State::Out, &mut sfs, false)?`.
    pub fn split_shapes(
        &mut self,
        ex: &mut TopOpeBRepToolShapeExplorer,
        to_build1: State,
        to_build2: State,
        a_set: &mut dyn TopOpeBRepBuildShapeSet,
        rev_ori: bool,
    ) -> Result<(), Failure> {
        while ex.more() {
            let mut a_shape = ex.current().clone();
            let t = a_shape.shape_type()?;
            let consumed = t == ShapeEnum::Edge && self.my_consumed_face_edges.is_bound(&a_shape);
            let newori = Self::orient(self.my_build_tool.orientation(&a_shape), rev_ori);
            match t {
                ShapeEnum::Solid | ShapeEnum::Shell => {
                    self.split_solid(&a_shape, to_build1, to_build2)?
                }
                ShapeEnum::Face => self.split_face(&a_shape, to_build1, to_build2)?,
                ShapeEnum::Edge => self.split_edge(&a_shape, to_build1, to_build2)?,
                _ => {
                    ex.next()?;
                    continue;
                }
            }
            if self.is_split(&a_shape, to_build1) {
                self.add_split_parts(&a_shape, to_build1, newori, a_set, consumed)?;
            } else if self.add_unsplit(&a_shape, to_build1)? && !consumed {
                a_shape = substitute_coincident_edges(
                    &a_shape,
                    &self.my_coincident_edges,
                    &self.my_build_tool,
                )?;
                self.my_build_tool.set_orientation(&mut a_shape, newori);
                a_set.add_element(&a_shape)?;
            }
            ex.next()?;
        }
        Ok(())
    }

    /// `FillShape(S1, ToBuild1, LS2, ToBuild2, aSet, RevOri)`: the wires (of a face) or
    /// shells (of a solid) of the forward shape not in the structure added when kept,
    /// the others explored into their edges or faces by `SplitShapes` (reversed for a
    /// shape of the same domain DIFFORIENTED); `b.fill_shape(&f, State::In, &l, State::Out, &mut wes, false)?`.
    pub fn fill_shape(
        &mut self,
        s1: &Shape,
        to_build1: State,
        ls2: &List<Shape>,
        to_build2: State,
        a_set: &mut dyn TopOpeBRepBuildShapeSet,
        in_rev_ori: bool,
    ) -> Result<(), Failure> {
        let mut rev_ori = in_rev_ori;
        let (t1, t11) = match s1.shape_type()? {
            ShapeEnum::Face => (ShapeEnum::Wire, ShapeEnum::Edge),
            ShapeEnum::Solid | ShapeEnum::Shell => (ShapeEnum::Shell, ShapeEnum::Face),
            _ => (ShapeEnum::Compound, ShapeEnum::Compound),
        };
        let hds = self.my_data_structure.clone();
        let hds = hds.get().ok_or_else(|| {
            Failure::new("TopOpeBRepBuild_Builder::FillShape: null data structure")
        })?;
        if hds.has_same_domain(s1, true)
            && hds.same_domain_orientation(s1) == TopOpeBRepDSConfig::DiffOriented
        {
            rev_ori = !rev_ori;
        }
        let mut a_shape = s1.clone();
        self.my_build_tool
            .set_orientation(&mut a_shape, Orientation::Forward);
        let mut ex1 = TopOpeBRepToolShapeExplorer::new(&a_shape, t1, ShapeEnum::Shape)?;
        while ex1.more() {
            let mut a_sub_shape = ex1.current().clone();
            if !hds.has_shape(&a_sub_shape, true) {
                if self.keep_shape(&a_sub_shape, ls2, to_build1)? {
                    let newori =
                        Self::orient(self.my_build_tool.orientation(&a_sub_shape), rev_ori);
                    a_sub_shape = substitute_coincident_edges(
                        &a_sub_shape,
                        &self.my_coincident_edges,
                        &self.my_build_tool,
                    )?;
                    self.my_build_tool.set_orientation(&mut a_sub_shape, newori);
                    a_set.add_shape(&a_sub_shape)?;
                }
            } else {
                let mut ex11 =
                    TopOpeBRepToolShapeExplorer::new(&a_sub_shape, t11, ShapeEnum::Shape)?;
                self.split_shapes(&mut ex11, to_build1, to_build2, a_set, rev_ori)?;
            }
            ex1.next()?;
        }
        Ok(())
    }

    /// `FillFace(F1, ToBuild1, LF2, ToBuild2, WES, RevOri)`: `FillShape` with the faces
    /// `lf2` as the faces of the 2d case; `b.fill_face(&f, State::In, &l, State::Out, &mut wes, false)?`.
    pub fn fill_face(
        &mut self,
        f1: &Shape,
        to_build1: State,
        lf2: &List<Shape>,
        to_build2: State,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
        rev_ori: bool,
    ) -> Result<(), Failure> {
        self.my_list_of_face = lf2.clone();
        let r = self.fill_shape(f1, to_build1, lf2, to_build2, wes, rev_ori);
        self.my_list_of_face.clear();
        r
    }

    /// `FillSolid(S1, ToBuild1, LS2, ToBuild2, aSet, RevOri)`: `FillShape`;
    /// `b.fill_solid(&so, State::In, &l, State::Out, &mut sfs, false)?`.
    pub fn fill_solid(
        &mut self,
        s1: &Shape,
        to_build1: State,
        ls2: &List<Shape>,
        to_build2: State,
        a_set: &mut dyn TopOpeBRepBuildShapeSet,
        rev_ori: bool,
    ) -> Result<(), Failure> {
        self.fill_shape(s1, to_build1, ls2, to_build2, a_set, rev_ori)
    }

    /// `FillVertexSet(IT, ToBuild, PVS)`: a pave per remaining point of the iterator;
    /// `b.fill_vertex_set(&mut it, State::In, &mut pvs)?`.
    pub fn fill_vertex_set(
        &self,
        it: &mut TopOpeBRepDSPointIterator,
        to_build: State,
        pvs: &mut TopOpeBRepBuildPaveSet,
    ) -> Result<(), Failure> {
        while it.more() {
            self.fill_vertex_set_on_value(it, to_build, pvs)?;
            it.next()?;
        }
        Ok(())
    }

    /// `FillVertexSetOnValue(IT, ToBuild, PVS)`: the pave of the new vertex of the
    /// current point (or of its vertex) at its parameter, oriented on the state;
    /// `b.fill_vertex_set_on_value(&it, State::In, &mut pvs)?`.
    pub fn fill_vertex_set_on_value(
        &self,
        it: &TopOpeBRepDSPointIterator,
        to_build: State,
        pvs: &mut TopOpeBRepBuildPaveSet,
    ) -> Result<(), Failure> {
        let hds = self.hds()?;
        let ind = it.current()?;
        let mut v = if it.is_point()? && ind <= hds.nb_points() {
            self.new_vertex(ind)?.clone()
        } else {
            hds.shape(ind, true)
        };
        let par = it.parameter()?;
        self.my_build_tool
            .set_orientation(&mut v, it.orientation(to_build)?);
        pvs.append(&Handle::new(TopOpeBRepBuildPave::new(&v, par, false)));
        Ok(())
    }
}
