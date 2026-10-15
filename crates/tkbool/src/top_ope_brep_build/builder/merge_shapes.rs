//! The merges of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_Merge.cxx): MergeShapes and
//! its solid forms, the solids, shells and edges of the builders and the merged maps.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildEdgeBuilder, TopOpeBRepBuildShellFaceSet, TopOpeBRepBuildSolidBuilder,
};
use crate::top_ope_brep_ds::MapOfShapeOn1State;
use crate::top_ope_brep_tool::TopOpeBRepToolShapeExplorer;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// The vertex of a new edge equal to `v` for MakeEdges: the same vertex, a vertex of the
/// same FORWARD or REVERSED orientation, or an INTERNAL or EXTERNAL one at the same
/// parameter.
pub(super) fn has_equal_vertex(new_edge: &Shape, v: &Shape, par_v: f64) -> Result<bool, Failure> {
    let ori_v = v.orientation();
    for ve in Explorer::new(new_edge, ShapeEnum::Vertex, ShapeEnum::Shape)? {
        let ori_ve = ve.orientation();
        if v.is_equal(&ve) {
            return Ok(true);
        }
        match ori_ve {
            Orientation::Forward | Orientation::Reversed if ori_v == ori_ve => return Ok(true),
            Orientation::Internal | Orientation::External => {
                let par_ve = BRepTool::parameter(&topods::vertex(&ve)?, &topods::edge(new_edge)?)?;
                if par_v == par_ve {
                    return Ok(true);
                }
            }
            _ => {}
        }
    }
    Ok(false)
}

/// The vertex of the edge builder replaced by the reference of its same domain.
type EdgeEnd = Option<(Shape, f64)>;

impl TopOpeBRepBuildBuilder {
    /// The explorer of the shapes of a shape to split: the solids, shells, faces or
    /// edges of a compound (the first type found), the edges of a wire, else its type.
    fn split_explorer(
        s: &Shape,
    ) -> Result<(ShapeEnum, ShapeEnum, TopOpeBRepToolShapeExplorer), Failure> {
        let t = Self::top_type(s)?;
        let tex = match t {
            ShapeEnum::Compound => {
                let mut found = ShapeEnum::Edge;
                for c in [ShapeEnum::Solid, ShapeEnum::Shell, ShapeEnum::Face] {
                    if TopOpeBRepToolShapeExplorer::new(s, c, ShapeEnum::Shape)?.more() {
                        found = c;
                        break;
                    }
                }
                found
            }
            ShapeEnum::Wire => ShapeEnum::Edge,
            t => t,
        };
        Ok((
            t,
            tex,
            TopOpeBRepToolShapeExplorer::new(s, tex, ShapeEnum::Shape)?,
        ))
    }

    /// The merged list of a compound set to the merged list of its first explored shape.
    fn merge_compound(
        &mut self,
        s: &Shape,
        t: ShapeEnum,
        tex: ShapeEnum,
        to_build: State,
    ) -> Result<(), Failure> {
        if t != ShapeEnum::Compound {
            return Ok(());
        }
        self.change_merged(s, to_build);
        let ex = TopOpeBRepToolShapeExplorer::new(s, tex, ShapeEnum::Shape)?;
        if ex.more() {
            let lss = self.change_merged(ex.current(), to_build).clone();
            *self.change_merged(s, to_build) = lss;
        }
        Ok(())
    }

    /// `MergeShapes(S1, ToBuild1, S2, ToBuild2)`: the shapes of each shape split (by
    /// MergeKPart in a particular case), the merged list of a compound that of its first
    /// shape; `b.merge_shapes(&s1, State::Out, &s2, State::Out)?`.
    pub fn merge_shapes(
        &mut self,
        s1: &Shape,
        to_build1: State,
        s2: &Shape,
        to_build2: State,
    ) -> Result<(), Failure> {
        if s1.is_equal(s2) {
            return Ok(());
        }
        (self.my_state1, self.my_state2) = (to_build1, to_build2);
        (self.my_shape1, self.my_shape2) = (s1.clone(), s2.clone());
        self.map_shapes(s1, s2)?;
        self.split_section_edges()?;
        if self.is_k_part() != 0 {
            self.merge_k_part()?;
            self.clear_maps();
            return Ok(());
        }
        let (rev1, rev2) = (
            Self::reverse(to_build1, to_build2),
            Self::reverse(to_build2, to_build1),
        );
        let mut sfs = TopOpeBRepBuildShellFaceSet::new()?;
        let mut tt1 = None;
        if !s1.is_null() {
            let (t1, tex1, mut ex1) = Self::split_explorer(s1)?;
            self.split_shapes(&mut ex1, to_build1, to_build2, &mut sfs, rev1)?;
            tt1 = Some((t1, tex1));
        }
        let mut tt2 = None;
        if !s2.is_null() {
            let (t2, tex2, mut ex2) = Self::split_explorer(s2)?;
            self.split_shapes(&mut ex2, to_build2, to_build1, &mut sfs, rev2)?;
            tt2 = Some((t2, tex2));
        }
        if let Some((t1, tex1)) = tt1 {
            self.merge_compound(s1, t1, tex1, to_build1)?;
        }
        if let Some((t2, tex2)) = tt2 {
            self.merge_compound(s2, t2, tex2, to_build2)?;
        }
        self.clear_maps();
        Ok(())
    }

    /// `MergeSolids(S1, ToBuild1, S2, ToBuild2)`: `MergeShapes`;
    /// `b.merge_solids(&s1, State::Out, &s2, State::Out)?`.
    pub fn merge_solids(
        &mut self,
        s1: &Shape,
        to_build1: State,
        s2: &Shape,
        to_build2: State,
    ) -> Result<(), Failure> {
        self.merge_shapes(s1, to_build1, s2, to_build2)
    }

    /// `MergeSolid(S, ToBuild)`: `MergeShapes` with a null second shape;
    /// `b.merge_solid(&s, State::Out)?`.
    pub fn merge_solid(&mut self, s: &Shape, to_build: State) -> Result<(), Failure> {
        self.merge_shapes(s, to_build, &Shape::default(), to_build)
    }

    /// The shell of the current shell of the solid builder: the old one, or a new shell
    /// of its faces.
    fn sobu_shell(&self, sobu: &mut TopOpeBRepBuildSolidBuilder) -> Result<Shape, Failure> {
        if sobu.is_old_shell()? {
            return sobu.old_shell();
        }
        let mut new_shell = Shape::default();
        self.my_build_tool.make_shell(&mut new_shell)?;
        sobu.init_face()?;
        while sobu.more_face() {
            self.my_build_tool
                .add_shell_face(&mut new_shell, sobu.face()?)?;
            sobu.next_face();
        }
        Ok(new_shell)
    }

    /// `MakeSolids(SOBU, L)`: a solid of the (closed) shells per solid of the builder,
    /// appended to `l`; `b.make_solids(&mut sobu, &mut l)?`.
    pub fn make_solids(
        &self,
        sobu: &mut TopOpeBRepBuildSolidBuilder,
        l: &mut List<Shape>,
    ) -> Result<(), Failure> {
        sobu.init_solid();
        while sobu.more_solid() {
            let mut new_solid = Shape::default();
            self.my_build_tool.make_solid(&mut new_solid)?;
            sobu.init_shell();
            while sobu.more_shell() {
                let mut new_shell = self.sobu_shell(sobu)?;
                self.my_build_tool.closed(&mut new_shell, true)?; // NYI : check exact du caractere closed du shell
                self.my_build_tool
                    .add_solid_shell(&mut new_solid, &new_shell)?;
                sobu.next_shell();
            }
            l.append(new_solid);
            sobu.next_solid();
        }
        Ok(())
    }

    /// `MakeShells(SOBU, L)`: the shells of the builder appended to `l`;
    /// `b.make_shells(&mut sobu, &mut l)?`.
    pub fn make_shells(
        &self,
        sobu: &mut TopOpeBRepBuildSolidBuilder,
        l: &mut List<Shape>,
    ) -> Result<(), Failure> {
        sobu.init_shell();
        while sobu.more_shell() {
            l.append(self.sobu_shell(sobu)?);
            sobu.next_shell();
        }
        Ok(())
    }

    /// The vertices of the current edge of the builder added (with their parameter) to
    /// the copy, but the EXTERNAL ones and those equal to a vertex of the copy; the
    /// FORWARD and REVERSED ends added, or None when no vertex is added.
    fn add_edbu_vertices(
        &self,
        edbu: &mut TopOpeBRepBuildEdgeBuilder,
        new_edge: &mut Shape,
    ) -> Result<Option<(EdgeEnd, EdgeEnd)>, Failure> {
        let hds = self.hds()?;
        let (mut hasvertex, mut fwd, mut rev): (bool, EdgeEnd, EdgeEnd) = (false, None, None);
        edbu.init_vertex();
        while edbu.more_vertex() {
            let mut v = edbu.vertex()?;
            let vori = v.orientation();
            if hds.has_same_domain(&v, true) {
                v = hds.shape(hds.same_domain_reference(&v), true);
                v.set_orientation(vori);
            }
            if v.orientation() != Orientation::External
                && !has_equal_vertex(new_edge, &v, edbu.parameter()?)?
            {
                hasvertex = true;
                let par_v = edbu.parameter()?;
                match vori {
                    Orientation::Forward => fwd = Some((v.clone(), par_v)),
                    Orientation::Reversed => rev = Some((v.clone(), par_v)),
                    _ => {}
                }
                self.my_build_tool.add_edge_vertex(new_edge, &v)?;
                self.my_build_tool.parameter(new_edge, &v, par_v)?;
            }
            edbu.next_vertex();
        }
        Ok(hasvertex.then_some((fwd, rev)))
    }

    /// `MakeEdges(E, EDBU, L)`: per edge of the builder with at least two vertices, a
    /// copy of `e` with the new vertices (degenerated when its two ends are the same
    /// vertex at the same parameter), appended to `l` when a vertex was added;
    /// `b.make_edges(&e, &mut eb, &mut l)?`.
    pub fn make_edges(
        &self,
        an_edge: &Shape,
        edbu: &mut TopOpeBRepBuildEdgeBuilder,
        l: &mut List<Shape>,
    ) -> Result<(), Failure> {
        edbu.init_edge();
        while edbu.more_edge() {
            let mut nloop = 0;
            edbu.init_vertex();
            while edbu.more_vertex() {
                nloop += 1;
                edbu.next_vertex();
            }
            if nloop > 1 {
                let mut new_edge = Shape::default();
                self.my_build_tool.copy_edge(an_edge, &mut new_edge)?;
                if let Some((fwd, rev)) = self.add_edbu_vertices(edbu, &mut new_edge)? {
                    if let (Some((vf, pf)), Some((vr, pr))) = (fwd, rev)
                        && vf.is_same(&vr)
                        && (pf - pr).abs() <= precision::p_confusion()
                    {
                        BRepBuilder::default().degenerated(&topods::edge(&new_edge)?, true)?;
                    }
                    l.append(new_edge);
                }
            }
            edbu.next_edge();
        }
        Ok(())
    }

    /// The merged map of a state (none for another state than IN, OUT, ON).
    fn merged_map(&self, to_build: State) -> Option<&MapOfShapeOn1State> {
        match to_build {
            State::Out => Some(&self.my_merged_out),
            State::In => Some(&self.my_merged_in),
            State::On => Some(&self.my_merged_on),
            _ => None,
        }
    }

    /// `IsMerged(S, ToBuild)`: `s` has merged shapes on the state; `b.is_merged(&s, State::In)`.
    pub fn is_merged(&self, s: &Shape, to_build: State) -> bool {
        self.merged_map(to_build).is_some_and(|p| p.is_bound(s))
            && !self.merged(s, to_build).is_empty()
    }

    /// `Merged(S, ToBuild)`: the merged shapes of `s` on the state, empty when none;
    /// `b.merged(&s, State::In)`.
    pub fn merged(&self, s: &Shape, to_build: State) -> &List<Shape> {
        match self.merged_map(to_build).and_then(|p| p.find(s).ok()) {
            Some(l) => l.list_on_state(),
            None => &self.my_empty_shape_list,
        }
    }

    /// `ChangeMerged(S, ToBuild)`: the merged shapes of `s` on the state, bound empty
    /// first (the shared empty list for another state); `b.change_merged(&s, State::In)`.
    pub fn change_merged(&mut self, s: &Shape, to_build: State) -> &mut List<Shape> {
        let p = match to_build {
            State::Out => &mut self.my_merged_out,
            State::In => &mut self.my_merged_in,
            State::On => &mut self.my_merged_on,
            _ => return &mut self.my_empty_shape_list,
        };
        Self::bound_losos(p, s).change_list_on_state()
    }

    /// `MergeEdges(L1, ToBuild1, L2, ToBuild2, Keepon1, Keepon2, Keepon12)`: nothing,
    /// as natively; `b.merge_edges(&l1, State::In, &l2, State::Out, true, true, true)`.
    #[allow(clippy::too_many_arguments)]
    pub fn merge_edges(
        &mut self,
        _l1: &List<Shape>,
        _tb1: State,
        _l2: &List<Shape>,
        _tb2: State,
        _k1: bool,
        _k2: bool,
        _k12: bool,
    ) {
    }

    /// `MergeFaces(S1, ToBuild1, S2, ToBuild2, onA, onB, onAB)`: nothing, as natively;
    /// `b.merge_faces(&l1, State::In, &l2, State::Out, true, true, true)`.
    #[allow(clippy::too_many_arguments)]
    pub fn merge_faces(
        &mut self,
        _l1: &List<Shape>,
        _tb1: State,
        _l2: &List<Shape>,
        _tb2: State,
        _a: bool,
        _b: bool,
        _ab: bool,
    ) {
    }
}
