//! The new vertices (TopOpeBRepBuild_BuildVertices.cxx) and faces
//! (TopOpeBRepBuild_BuildFaces.cxx) of the points and surfaces of the structure.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::{
    TopOpeBRepBuildFaceBuilder, TopOpeBRepBuildShapeSet, TopOpeBRepBuildWireEdgeSet,
};
use crate::top_ope_brep_ds::TopOpeBRepDSHDataStructure;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, State};

/// The null data structure of a build.
pub(crate) fn null_hds() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "null data structure",
        "TopOpeBRepBuild_Builder",
    )
}

impl TopOpeBRepBuildBuilder {
    /// `BuildVertices(HDS)`: a vertex per point of the structure;
    /// `b.build_vertices(&hds)?`.
    pub fn build_vertices(
        &mut self,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
    ) -> Result<(), Failure> {
        let h = hds.get().ok_or_else(null_hds)?;
        let n = h.nb_points();
        self.my_new_vertices = vec![Shape::default(); usize::try_from(n).unwrap_or(0) + 1];
        for ip in 1..=n {
            let p = h.point(ip)?;
            let mut v = Shape::default();
            self.my_build_tool.make_vertex(&mut v, &p)?;
            *self.change_new_vertex(ip)? = v;
        }
        Ok(())
    }

    /// The orientations (1 FORWARD, 2 REVERSED) of the equivalent curves of the curves
    /// of the surface, by reference curve.
    fn surface_curve_masks(
        hds: &TopOpeBRepDSHDataStructure,
        is: i32,
    ) -> Result<DataMap<i32, i32>, Failure> {
        let mut masks = DataMap::default();
        let mut it = hds.surface_curves(is)?;
        while it.more() {
            let c = hds.curve(it.current()?);
            let rc = c.equivalent_curve();
            if rc > 0 {
                let mut o = it.orientation(State::In)?;
                if c.is_equivalent_curve_reversed() {
                    o = top_abs::reverse(o);
                }
                let mut mask = masks.find(&rc).copied().unwrap_or(0);
                match o {
                    Orientation::Forward => mask |= 1,
                    Orientation::Reversed => mask |= 2,
                    _ => {}
                }
                masks.bind(rc, mask);
            }
            it.next()?;
        }
        Ok(masks)
    }

    /// `BuildFaces(iS, HDS)`: the faces of the surface `is` built from the new edges of
    /// its curves (not of a group of both orientations), toleranced at least as the
    /// surface; `b.build_faces_of(1, &hds)?`.
    pub fn build_faces_of(
        &mut self,
        is: i32,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
    ) -> Result<(), Failure> {
        let h = hds.get().ok_or_else(null_hds)?;
        let tbs = h.surface(is);
        let tbs_tol = tbs.tolerance();
        let mut a_face = Shape::default();
        self.my_build_tool.make_face(&mut a_face, &tbs)?;
        let mut wes = TopOpeBRepBuildWireEdgeSet::new(&a_face)?;
        let masks = Self::surface_curve_masks(h, is)?;
        let bb = BRepBuilder::default();
        let mut scurves = h.surface_curves(is)?;
        while scurves.more() {
            let ic = scurves.current()?;
            let cds = h.curve(ic);
            let rc = cds.equivalent_curve();
            if !(rc > 0 && masks.find(&rc).ok() == Some(&3)) {
                for e in self.new_edges(ic).clone().iter() {
                    let mut an_edge = e.clone();
                    let ae = topods::edge(&an_edge)?;
                    if BRepTool::edge_tolerance(&ae)? < tbs_tol {
                        bb.update_edge_tolerance(&ae, tbs_tol)?;
                    }
                    let mut ori = scurves.orientation(State::In)?;
                    if cds.is_equivalent_curve_reversed() {
                        ori = top_abs::reverse(ori);
                    }
                    self.my_build_tool.set_orientation(&mut an_edge, ori);
                    let pc = scurves.pcurve()?;
                    self.my_build_tool
                        .pcurve_of_curve(&mut a_face, &mut an_edge, &cds, &pc)?;
                    wes.add_start_element(&an_edge)?;
                }
            }
            scurves.next()?;
        }
        let mut fabu = TopOpeBRepBuildFaceBuilder::with_wire_edge_set(&mut wes, &a_face, false)?;
        let mut face_list = std::mem::take(self.change_new_faces(is)?);
        let r = self.make_faces(&a_face, &mut fabu, &mut face_list);
        *self.change_new_faces(is)? = face_list;
        r
    }

    /// `BuildFaces(HDS)`: the faces of every surface of the structure;
    /// `b.build_faces(&hds)?`.
    pub fn build_faces(&mut self, hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
        let n = hds.get().ok_or_else(null_hds)?.nb_surfaces();
        self.my_new_faces = vec![List::default(); usize::try_from(n).unwrap_or(0) + 1];
        for is in 1..=n {
            self.build_faces_of(is, hds)?;
        }
        Ok(())
    }
}
