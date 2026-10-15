//! The edges (TopOpeBRepBuild_makeedges.cxx) and solids (TopOpeBRepBuild_makesolids.cxx)
//! of the general builder from its pave and shell face sets.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildEdgeBuilder, TopOpeBRepBuildLoopSet, TopOpeBRepBuildPaveClassifier,
    TopOpeBRepBuildPaveSet, TopOpeBRepBuildShellFaceSet, TopOpeBRepBuildSolidBuilder,
};
use crate::top_ope_brep_tool::TopOpeBRepToolTOOL;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The ends of a new edge of GEDBUMakeEdges: the counts of FORWARD and REVERSED
/// vertices added, the first of each with its parameter, and an INTERNAL one added.
#[derive(Default)]
struct NewEdgeEnds {
    n_vf: i32,
    n_vr: i32,
    vf: (Shape, f64),
    vr: (Shape, f64),
    tosplit: bool,
}

impl TopOpeBRepBuildBuilder {
    /// `GPVSMakeEdges(EF, PVS, LOE)`: the edges of the paves of `pvs` (none without a
    /// pave); `b.gpvs_make_edges(&ef, &mut pvs, &mut loe)?`.
    pub fn gpvs_make_edges(
        &self,
        ef: &Shape,
        pvs: &mut TopOpeBRepBuildPaveSet,
        loe: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut vcl = TopOpeBRepBuildPaveClassifier::new(ef)?;
        if pvs.has_equal_parameters()? {
            vcl.set_first_parameter(pvs.equal_parameters());
        }
        pvs.init_loop()?;
        if !pvs.more_loop() {
            return Ok(());
        }
        let mut edbu = TopOpeBRepBuildEdgeBuilder::new();
        edbu.init_edge_builder(pvs, &mut vcl, false)?;
        self.gedbu_make_edges(ef, &mut edbu, loe)
    }

    /// The vertices of the current edge of the builder added to the copy as in
    /// MakeEdges, counted by orientation.
    fn gedbu_add_vertices(
        &self,
        edbu: &mut TopOpeBRepBuildEdgeBuilder,
        new_edge: &mut Shape,
    ) -> Result<NewEdgeEnds, Failure> {
        let hds = self.hds()?;
        let mut ends = NewEdgeEnds::default();
        edbu.init_vertex();
        while edbu.more_vertex() {
            let mut v = edbu.vertex()?;
            let vori = v.orientation();
            if hds.has_same_domain(&v, true) {
                v = hds.shape(hds.same_domain_reference(&v), true);
                v.set_orientation(vori);
            }
            let ori_v = v.orientation();
            let par_v = edbu.parameter()?;
            if ori_v != Orientation::External
                && !super::merge_shapes::has_equal_vertex(new_edge, &v, par_v)?
            {
                if vori == Orientation::Forward {
                    ends.n_vf += 1;
                    if ends.n_vf == 1 {
                        ends.vf = (v.clone(), par_v);
                    }
                }
                if vori == Orientation::Reversed {
                    ends.n_vr += 1;
                    if ends.n_vr == 1 {
                        ends.vr = (v.clone(), par_v);
                    }
                }
                ends.tosplit |= ori_v == Orientation::Internal;
                self.my_build_tool.add_edge_vertex(new_edge, &v)?;
                self.my_build_tool.parameter(new_edge, &v, par_v)?;
            }
            edbu.next_vertex();
        }
        Ok(ends)
    }

    /// `GEDBUMakeEdges(EF, EDBU, LOE)`: per edge of the builder with at least two
    /// vertices, a copy of `ef` with its new vertices, kept with one FORWARD and one
    /// REVERSED vertex (degenerated on one vertex at one parameter), split at its
    /// INTERNAL vertices; `b.gedbu_make_edges(&ef, &mut eb, &mut loe)?`.
    pub fn gedbu_make_edges(
        &self,
        ef: &Shape,
        edbu: &mut TopOpeBRepBuildEdgeBuilder,
        loe: &mut List<Shape>,
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
                self.my_build_tool.copy_edge(ef, &mut new_edge)?;
                let ends = self.gedbu_add_vertices(edbu, &mut new_edge)?;
                if ends.n_vf == 1 && ends.n_vr == 1 {
                    self.add_gedbu_edge(new_edge, &ends, loe)?;
                }
            }
            edbu.next_edge();
        }
        Ok(())
    }

    /// A new edge appended (degenerated on one vertex at one parameter), or its splits
    /// at its INTERNAL vertices.
    fn add_gedbu_edge(
        &self,
        new_edge: Shape,
        ends: &NewEdgeEnds,
        loe: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let ne = topods::edge(&new_edge)?;
        if ends.vf.0.is_same(&ends.vr.0)
            && (ends.vf.1 - ends.vr.1).abs() <= precision::p_confusion()
        {
            BRepBuilder::default().degenerated(&ne, true)?;
        }
        if ends.tosplit {
            let mut l = List::default();
            if TopOpeBRepToolTOOL::split_e(&ne, &mut l)? {
                loe.append_list(&mut l);
                return Ok(());
            }
        }
        loe.append(new_edge);
        Ok(())
    }

    /// `GSFSMakeSolids(SOF, SFS, LOSO)`: the solids of the shell face set (classes
    /// forced); `b.gsfs_make_solids(&so, &mut sfs, &mut loso)?`.
    pub fn gsfs_make_solids(
        &mut self,
        sof: &Shape,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
        loso: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut sobu = TopOpeBRepBuildSolidBuilder::new();
        sobu.init_solid_builder(sfs, true)?;
        self.gsobu_make_solids(sof, &mut sobu, loso)
    }

    /// Whether every non-degenerated FORWARD or REVERSED edge of the shell has two faces.
    fn shell_closed(new_shell: &Shape) -> Result<bool, Failure> {
        let mut edgemap = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(
            new_shell,
            ShapeEnum::Edge,
            ShapeEnum::Face,
            &mut edgemap,
        )?;
        for iedge in 1..=edgemap.len() {
            let e = edgemap.find_key(iedge)?;
            if matches!(
                e.orientation(),
                Orientation::Internal | Orientation::External
            ) || BRepTool::degenerated(&topods::edge(e)?)?
            {
                continue;
            }
            if edgemap.find_from_index(iedge)?.len() < 2 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// The solid of the current solid of the builder: its old shells, or new shells
    /// closed when each edge has two faces; the shell count and `nfa` set to the face
    /// count of the last new shell (natively kept across the solids).
    fn gsobu_solid(
        &self,
        sobu: &mut TopOpeBRepBuildSolidBuilder,
        nfa: &mut i32,
    ) -> Result<(Shape, i32), Failure> {
        let mut new_solid = Shape::default();
        self.my_build_tool.make_solid(&mut new_solid)?;
        let nsh = sobu.init_shell();
        while sobu.more_shell() {
            let isold = sobu.is_old_shell()?;
            let mut new_shell = if isold {
                sobu.old_shell()?
            } else {
                let mut sh = Shape::default();
                self.my_build_tool.make_shell(&mut sh)?;
                *nfa = sobu.init_face()?;
                while sobu.more_face() {
                    self.my_build_tool.add_shell_face(&mut sh, sobu.face()?)?;
                    sobu.next_face();
                }
                sh
            };
            if !isold {
                let closed = Self::shell_closed(&new_shell)?;
                self.my_build_tool.closed(&mut new_shell, closed)?;
            }
            self.my_build_tool
                .add_solid_shell(&mut new_solid, &new_shell)?;
            sobu.next_shell();
        }
        Ok((new_solid, nsh))
    }

    /// `GSOBUMakeSolids(SOF, SOBU, LOSO)`: per solid of the builder with a vertex (not a
    /// single face of degenerated edges only), its regularized solids appended to `loso`;
    /// `b.gsobu_make_solids(&so, &mut sb, &mut loso)?`.
    pub fn gsobu_make_solids(
        &mut self,
        sof: &Shape,
        sobu: &mut TopOpeBRepBuildSolidBuilder,
        loso: &mut List<Shape>,
    ) -> Result<(), Failure> {
        let mut nfa = 0;
        sobu.init_solid();
        while sobu.more_solid() {
            let (new_solid, nsh) = self.gsobu_solid(sobu, &mut nfa)?;
            let has_vertex = Explorer::new(&new_solid, ShapeEnum::Vertex, ShapeEnum::Shape)?.more();
            let degenerated_only = nsh == 1
                && nfa == 1
                && !Explorer::new(&new_solid, ShapeEnum::Edge, ShapeEnum::Shape)?
                    .map(|e| topods::edge(&e).and_then(|e| BRepTool::degenerated(&e)))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .any(|d| !d);
            if has_vertex && !degenerated_only {
                let mut new_solid_los = List::default();
                self.regularize_solid(sof, &new_solid, &mut new_solid_los)?;
                loso.append_list(&mut new_solid_los);
            }
            sobu.next_solid();
        }
        Ok(())
    }
}
