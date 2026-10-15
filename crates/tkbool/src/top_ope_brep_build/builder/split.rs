//! The splits of `TopOpeBRepBuild_Builder` (TopOpeBRepBuild_Builder.cxx): the edges by
//! their paves, the faces by their wire edge sets and the solids by their shell face
//! sets, with the shapes of the same domain sharing the result.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildEdgeBuilder, TopOpeBRepBuildFaceBuilder, TopOpeBRepBuildLoopSet,
    TopOpeBRepBuildPaveClassifier, TopOpeBRepBuildPaveSet, TopOpeBRepBuildShapeSet,
    TopOpeBRepBuildShellFaceSet, TopOpeBRepBuildSolidBuilder, TopOpeBRepBuildWireEdgeSet,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
use std::cell::Cell;

/// The forward face, its two same domain lists and their reversal flags.
type FaceDomains = (Shape, List<Shape>, List<Shape>, bool, bool);

thread_local! {
    /// `STATIC_SOLIDINDEX`: the rank of the solids being filled by SplitSolid.
    pub(crate) static STATIC_SOLIDINDEX: Cell<i32> = const { Cell::new(0) };
}

impl TopOpeBRepBuildBuilder {
    /// The split lists of the shapes of `l` on the state set to `parts` when connected
    /// (marked split in any case).
    fn connect_splits(
        &mut self,
        l: &List<Shape>,
        to_build: State,
        connect: bool,
        parts: &List<Shape>,
    ) {
        for cur in l.iter() {
            self.mark_split(cur, to_build, true);
            let el = self.change_split(cur, to_build);
            if connect {
                *el = parts.clone();
            }
        }
    }

    /// The paves and the pave classifier of the forward edge on the state.
    fn edge_paves(
        &self,
        eforward: &Shape,
        to_build1: State,
    ) -> Result<(TopOpeBRepBuildPaveSet, TopOpeBRepBuildPaveClassifier), Failure> {
        let mut pvs = TopOpeBRepBuildPaveSet::new(eforward)?;
        let mut epit = self.hds()?.edge_points(eforward)?;
        self.fill_vertex_set(&mut epit, to_build1, &mut pvs)?;
        let mut vcl = TopOpeBRepBuildPaveClassifier::new(eforward)?;
        if pvs.has_equal_parameters()? {
            vcl.set_first_parameter(pvs.equal_parameters());
        }
        Ok((pvs, vcl))
    }

    /// The edges of the paves appended to the merged list of the edge on the state
    /// (nothing when no pave); the merged list returned.
    fn merge_edge_paves(
        &mut self,
        eforward: &Shape,
        to_build1: State,
        pvs: &mut TopOpeBRepBuildPaveSet,
        vcl: &mut TopOpeBRepBuildPaveClassifier,
        merged: bool,
    ) -> Result<Option<List<Shape>>, Failure> {
        pvs.init_loop()?;
        if !pvs.more_loop() {
            return Ok(None);
        }
        let mut ebu = TopOpeBRepBuildEdgeBuilder::with_paves(pvs, vcl, false)?;
        let target = if merged {
            self.change_merged(eforward, to_build1)
        } else {
            self.change_split(eforward, to_build1)
        };
        let mut edge_list = std::mem::take(target);
        let r = self.make_edges(eforward, &mut ebu, &mut edge_list);
        let target = if merged {
            self.change_merged(eforward, to_build1)
        } else {
            self.change_split(eforward, to_build1)
        };
        *target = edge_list.clone();
        r.map(|()| Some(edge_list))
    }

    /// `SplitEdge(E, ToBuild1, ToBuild2)`: `SplitEdge1`; `b.split_edge(&e, State::In, State::Out)?`.
    pub fn split_edge(
        &mut self,
        e: &Shape,
        to_build1: State,
        to_build2: State,
    ) -> Result<(), Failure> {
        self.split_edge1(e, to_build1, to_build2)
    }

    /// `SplitEdge1(E, ToBuild1, ToBuild2)`: the forward edge split by its paves on the
    /// state, the parts merged and set as the splits of the edges of its same domain of
    /// the first shape; `b.split_edge1(&e, State::In, State::Out)?`.
    pub fn split_edge1(
        &mut self,
        eoriented: &Shape,
        to_build1: State,
        to_build2: State,
    ) -> Result<(), Failure> {
        let eforward = eoriented.oriented(Orientation::Forward);
        if !self.to_split(eoriented, to_build1)? {
            return Ok(());
        }
        let (mut le1, mut le2) = (List::default(), List::default());
        le1.append(eforward.clone());
        self.find_same_domain(&mut le1, &mut le2)?;
        let (mut pvs, mut vcl) = self.edge_paves(&eforward, to_build1)?;
        self.mark_split(&eforward, to_build1, true);
        let Some(edge_list) =
            self.merge_edge_paves(&eforward, to_build1, &mut pvs, &mut vcl, true)?
        else {
            return Ok(());
        };
        self.connect_splits(&le1, to_build1, true, &edge_list);
        self.connect_splits(&le2, to_build2, false, &edge_list);
        Ok(())
    }

    /// `SplitEdge2(E, ToBuild1, ToBuild2)`: the forward edge split by its paves into its
    /// split list on the state; `b.split_edge2(&e, State::In, State::Out)?`.
    pub fn split_edge2(
        &mut self,
        eoriented: &Shape,
        to_build1: State,
        _to_build2: State,
    ) -> Result<(), Failure> {
        if !self.to_split(eoriented, to_build1)? {
            return Ok(());
        }
        let mut eforward = eoriented.clone();
        self.my_build_tool
            .set_orientation(&mut eforward, Orientation::Forward);
        let (mut pvs, mut vcl) = self.edge_paves(&eforward, to_build1)?;
        self.mark_split(&eforward, to_build1, true);
        self.merge_edge_paves(&eforward, to_build1, &mut pvs, &mut vcl, false)?;
        Ok(())
    }

    /// `SplitFace(F, ToBuild1, ToBuild2)`: `SplitFace1`; `b.split_face(&f, State::In, State::Out)?`.
    pub fn split_face(
        &mut self,
        f: &Shape,
        to_build1: State,
        to_build2: State,
    ) -> Result<(), Failure> {
        self.split_face1(f, to_build1, to_build2)
    }

    /// The forward face and its same domain lists, with the reversal flags (cleared when
    /// the other list is empty).
    fn face_domains(
        &mut self,
        foriented: &Shape,
        to_build1: State,
        to_build2: State,
    ) -> Result<FaceDomains, Failure> {
        let (mut rev1, mut rev2) = (
            Self::reverse(to_build1, to_build2),
            Self::reverse(to_build2, to_build1),
        );
        let mut fforward = foriented.clone();
        self.my_build_tool
            .set_orientation(&mut fforward, Orientation::Forward);
        let (mut lf1, mut lf2) = (List::default(), List::default());
        lf1.append(fforward.clone());
        self.find_same_domain(&mut lf1, &mut lf2)?;
        if lf2.is_empty() {
            rev1 = false;
        }
        if lf1.is_empty() {
            rev2 = false;
        }
        Ok((fforward, lf1, lf2, rev1, rev2))
    }

    /// The faces of a wire edge set merged on the state of the forward face (not for a
    /// consumed face); the merged list returned.
    fn merge_face_wes(
        &mut self,
        fforward: &Shape,
        to_build: State,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
    ) -> Result<List<Shape>, Failure> {
        let mut fbu = TopOpeBRepBuildFaceBuilder::with_wire_edge_set(wes, fforward, false)?;
        let mut face_list = std::mem::take(self.change_merged(fforward, to_build));
        let r = if self.my_consumed_faces.is_bound(fforward) {
            Ok(())
        } else {
            self.make_faces(fforward, &mut fbu, &mut face_list)
        };
        *self.change_merged(fforward, to_build) = face_list.clone();
        r.map(|()| face_list)
    }

    /// `SplitFace1(F, ToBuild1, ToBuild2)`: the wire edge set of the faces of the same
    /// domain and of the intersection edges built into faces, the splits of the faces of
    /// the first shape; `b.split_face1(&f, State::In, State::Out)?`.
    pub fn split_face1(
        &mut self,
        foriented: &Shape,
        to_build1: State,
        to_build2: State,
    ) -> Result<(), Failure> {
        if !self.to_split(foriented, to_build1)? {
            return Ok(());
        }
        let (mut fforward, lf1, lf2, rev1, rev2) =
            self.face_domains(foriented, to_build1, to_build2)?;
        let mut wes = TopOpeBRepBuildWireEdgeSet::new(&fforward)?;
        for fcur in lf1.iter() {
            self.fill_face(fcur, to_build1, &lf2, to_build2, &mut wes, rev1)?;
        }
        for fcur in lf2.iter() {
            self.fill_face(fcur, to_build2, &lf1, to_build1, &mut wes, rev2)?;
        }
        self.add_intersection_edges(&mut fforward, to_build1, rev1, &mut wes)?;
        let face_list = self.merge_face_wes(&fforward, to_build1, &mut wes)?;
        self.connect_splits(&lf1, to_build1, true, &face_list);
        self.connect_splits(&lf2, to_build2, false, &face_list);
        Ok(())
    }

    /// The faces of `l` filled into `wes` on the state against `lo` (the other state).
    fn fill_faces(
        &mut self,
        l: &List<Shape>,
        tob: State,
        lo: &List<Shape>,
        tobo: State,
        wes: &mut TopOpeBRepBuildWireEdgeSet,
        rev: bool,
    ) -> Result<(), Failure> {
        for fcur in l.iter() {
            self.fill_face(fcur, tob, lo, tobo, wes, rev)?;
        }
        Ok(())
    }

    /// `SplitFace2(F, ToBuild1, ToBuild2)`: the faces of the same domain split by their
    /// orientation and rank into two wire edge sets, one per state;
    /// `b.split_face2(&f, State::In, State::Out)?`.
    pub fn split_face2(
        &mut self,
        foriented: &Shape,
        to_build1: State,
        to_build2: State,
    ) -> Result<(), Failure> {
        if !self.to_split(foriented, to_build1)? {
            return Ok(());
        }
        let (mut fforward, lf1, lf2, rev1, rev2) =
            self.face_domains(foriented, to_build1, to_build2)?;
        let (mut lfso, mut lfoo) = (List::default(), List::default());
        lfso.append(fforward.clone());
        self.find_same_domain_same_orientation(&mut lfso, &mut lfoo)?;
        let rank_f = self.shape_rank(foriented);
        let rank_x = match rank_f {
            0 => 0,
            1 => 2,
            _ => 1,
        };
        let (mut lfso1, mut lfoo1, mut lfso2, mut lfoo2) = Default::default();
        self.find_same_rank(&lfso, rank_f, &mut lfso1);
        self.find_same_rank(&lfoo, rank_f, &mut lfoo1);
        self.find_same_rank(&lfso, rank_x, &mut lfso2);
        self.find_same_rank(&lfoo, rank_x, &mut lfoo2);
        let comp = |s: State| {
            if s == State::In {
                State::Out
            } else {
                State::In
            }
        };
        let (tob1, tob2, tob1c, tob2c) = (to_build1, to_build2, comp(to_build1), comp(to_build2));
        let mut wes1 = TopOpeBRepBuildWireEdgeSet::new(&fforward)?;
        self.fill_faces(&lfso1, tob1, &lf2, tob2, &mut wes1, rev1)?;
        self.fill_faces(&lfso2, tob2, &lf1, tob1, &mut wes1, rev2)?;
        self.fill_faces(&lfoo1, tob1c, &lf2, to_build2, &mut wes1, !rev1)?;
        self.fill_faces(&lfoo2, tob2c, &lf1, to_build1, &mut wes1, !rev2)?;
        self.add_intersection_edges(&mut fforward, to_build1, rev1, &mut wes1)?;
        let face_list1 = self.merge_face_wes(&fforward, to_build1, &mut wes1)?;
        self.connect_splits(&lf1, to_build1, true, &face_list1);
        let mut wes2 = TopOpeBRepBuildWireEdgeSet::new(&fforward)?;
        self.fill_faces(&lfso1, tob1c, &lf2, tob2, &mut wes2, !rev1)?;
        self.fill_faces(&lfso2, tob2c, &lf1, tob1, &mut wes2, !rev2)?;
        self.fill_faces(&lfoo1, tob1, &lf2, to_build2, &mut wes2, rev1)?;
        self.fill_faces(&lfoo2, tob2, &lf1, to_build1, &mut wes2, rev2)?;
        self.add_intersection_edges(&mut fforward, to_build2, rev2, &mut wes2)?;
        let face_list2 = self.merge_face_wes(&fforward, to_build2, &mut wes2)?;
        self.connect_splits(&lf2, to_build2, false, &face_list2);
        Ok(())
    }

    /// Whether a shell (by one of its faces) or a solid is to split.
    fn solid_to_split(&self, s1: &Shape, to_build1: State) -> Result<bool, Failure> {
        if s1.shape_type()? != ShapeEnum::Shell {
            return self.to_split(s1, to_build1);
        }
        for sh in Explorer::new(s1, ShapeEnum::Face, ShapeEnum::Shape)? {
            if self.to_split(&sh, to_build1)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// The new faces of the surfaces of the solid, oriented on the state, as elements.
    fn add_solid_surfaces(
        &self,
        s1forward: &Shape,
        to_build1: State,
        sfs: &mut TopOpeBRepBuildShellFaceSet,
    ) -> Result<(), Failure> {
        let hds = self.hds()?;
        if hds.nb_surfaces() <= 0 {
            return Ok(());
        }
        let mut ssurfaces = hds.solid_surfaces(s1forward)?;
        while ssurfaces.more() {
            let is = ssurfaces.current()?;
            for f in self.new_faces(is)?.iter() {
                let mut a_face = f.clone();
                self.my_build_tool
                    .set_orientation(&mut a_face, ssurfaces.orientation(to_build1)?);
                sfs.add_element(&a_face)?;
            }
            ssurfaces.next()?;
        }
        Ok(())
    }

    /// `SplitSolid(S1, ToBuild1, ToBuild2)`: the shell face set of the solids (or shells)
    /// of the same domain and of the new faces built into solids (or shells), the
    /// splits of the solids of the first shape; `b.split_solid(&so, State::In, State::Out)?`.
    pub fn split_solid(
        &mut self,
        s1oriented: &Shape,
        to_build1: State,
        to_build2: State,
    ) -> Result<(), Failure> {
        let is_shell = s1oriented.shape_type()? == ShapeEnum::Shell;
        if !self.solid_to_split(s1oriented, to_build1)? {
            return Ok(());
        }
        let (s1forward, ls1, ls2, rev1, rev2) =
            self.face_domains(s1oriented, to_build1, to_build2)?;
        let mut sfs = TopOpeBRepBuildShellFaceSet::new()?;
        STATIC_SOLIDINDEX.set(1);
        for scur in ls1.iter() {
            self.fill_solid(scur, to_build1, &ls2, to_build2, &mut sfs, rev1)?;
        }
        STATIC_SOLIDINDEX.set(2);
        for scur in ls2.iter() {
            self.fill_solid(scur, to_build2, &ls1, to_build1, &mut sfs, rev2)?;
        }
        self.add_solid_surfaces(&s1forward, to_build1, &mut sfs)?;
        let mut sobu = TopOpeBRepBuildSolidBuilder::with_shell_face_set(&mut sfs, false)?;
        let mut solid_list = std::mem::take(self.change_merged(s1oriented, to_build1));
        let r = if is_shell {
            self.make_shells(&mut sobu, &mut solid_list)
        } else {
            self.make_solids(&mut sobu, &mut solid_list)
        };
        *self.change_merged(s1oriented, to_build1) = solid_list.clone();
        r?;
        self.connect_splits(&ls1, to_build1, true, &solid_list);
        self.connect_splits(&ls2, to_build2, false, &solid_list);
        Ok(())
    }
}
