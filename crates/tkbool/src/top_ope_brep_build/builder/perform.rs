//! `Perform`, `Clear` and `AddIntersectionEdges` of `TopOpeBRepBuild_Builder`, with the
//! static helpers of the faces consumed by coincident curves.
use super::TopOpeBRepBuildBuilder;
use crate::top_ope_brep_build::TopOpeBRepBuildShapeSet;
use crate::top_ope_brep_ds::{
    TopOpeBRepDSBuildTool, TopOpeBRepDSCurveExplorer, TopOpeBRepDSFilter,
    TopOpeBRepDSHDataStructure, TopOpeBRepDSReducer,
};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{self, Orientation, ShapeEnum, State};

/// `SubstituteCoincidentEdges(theShape, theEdges, theBuildTool)` (static): a copy of a
/// face or wire with its edges replaced by their coincident edges (keeping their
/// orientation), the shape itself when nothing changes.
pub(crate) fn substitute_coincident_edges(
    the_shape: &Shape,
    the_edges: &DataMap<Shape, Shape, ShapeMapHasher>,
    bt: &TopOpeBRepDSBuildTool,
) -> Result<Shape, Failure> {
    let t = the_shape.shape_type()?;
    if t != ShapeEnum::Face && t != ShapeEnum::Wire {
        return Ok(the_shape.clone());
    }
    let mut a_copy = the_shape.empty_copied()?;
    let mut is_modified = false;
    for mut sub in ShapeIterator::new(the_shape, false, false) {
        let st = sub.shape_type()?;
        if st == ShapeEnum::Edge && the_edges.is_bound(&sub) {
            let o = sub.orientation();
            sub = the_edges.find(&sub).cloned().unwrap_or_default();
            sub.set_orientation(o);
            is_modified = true;
        } else if st == ShapeEnum::Wire {
            let new_wire = substitute_coincident_edges(&sub, the_edges, bt)?;
            is_modified = is_modified || !new_wire.is_same(&sub);
            sub = new_wire;
        }
        if t == ShapeEnum::Face {
            bt.add_face_wire(&mut a_copy, &sub)?;
        } else {
            bt.add_wire_edge(&mut a_copy, &sub)?;
        }
    }
    Ok(if is_modified {
        a_copy
    } else {
        the_shape.clone()
    })
}

/// `FillEquivalentCurveOrientationMasks(...)` (static): per reference curve of the
/// curves of the face, the orientations (1 FORWARD, 2 REVERSED) of its equivalents.
fn fill_equivalent_curve_orientation_masks(
    the_face: &Shape,
    the_state: State,
    the_reverse: bool,
    hds: &TopOpeBRepDSHDataStructure,
    masks: &mut DataMap<i32, i32>,
) -> Result<(), Failure> {
    let mut it = hds.face_curves(the_face)?;
    while it.more() {
        let c = hds.curve(it.current()?);
        let a_reference_curve = c.equivalent_curve();
        if a_reference_curve > 0 {
            let mut o = TopOpeBRepBuildBuilder::orient(it.orientation(the_state)?, the_reverse);
            if c.is_equivalent_curve_reversed() {
                o = top_abs::reverse(o);
            }
            let mut mask = masks.find(&a_reference_curve).copied().unwrap_or(0);
            match o {
                Orientation::Forward => mask |= 1,
                Orientation::Reversed => mask |= 2,
                _ => {}
            }
            masks.bind(a_reference_curve, mask);
        }
        it.next()?;
    }
    Ok(())
}

/// `IsConsumedByCoincidentCurves(theFace, theDataStructure)` (static): a reference
/// curve of the face has equivalents of both orientations, IN or OUT.
fn is_consumed_by_coincident_curves(
    the_face: &Shape,
    hds: &TopOpeBRepDSHDataStructure,
) -> Result<bool, Failure> {
    for a_state in [State::In, State::Out] {
        let mut masks = DataMap::default();
        fill_equivalent_curve_orientation_masks(the_face, a_state, false, hds, &mut masks)?;
        if masks.items().any(|(_, v)| *v == 3) {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `RegisterConsumedFaceEdges(theFace, theRestrictions, theConsumedEdges)` (static):
/// the edges of the face that are not restrictions of a curve.
fn register_consumed_face_edges(
    the_face: &Shape,
    restrictions: &DataMap<Shape, bool, ShapeMapHasher>,
    consumed: &mut DataMap<Shape, bool, ShapeMapHasher>,
) -> Result<(), Failure> {
    for e in Explorer::new(the_face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if !restrictions.is_bound(&e) && !consumed.is_bound(&e) {
            consumed.bind(e, true);
        }
    }
    Ok(())
}

impl TopOpeBRepBuildBuilder {
    /// `Perform(HDS)` (virtual): the new vertices, edges and faces of the data
    /// structure, the section edges split and the face interferences filtered and
    /// reduced; `b.perform(&hds)?`.
    pub fn perform(&mut self, hds: &Handle<TopOpeBRepDSHDataStructure>) -> Result<(), Failure> {
        if self.b1.is_some() {
            return self.b1_perform(hds);
        }
        self.perform_base(hds)
    }

    /// `TopOpeBRepBuild_Builder::Perform(HDS)`, the base version.
    pub(crate) fn perform_base(
        &mut self,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
    ) -> Result<(), Failure> {
        self.clear()?;
        self.my_data_structure = hds.clone();
        self.build_vertices(hds)?;
        self.split_eviso_on_periodic_f()?;
        self.build_edges(hds)?;
        self.find_consumed_faces()?;
        self.build_faces(hds)?;
        self.my_is_k_part = 0;
        self.init_section();
        self.split_section_edges()?;
        TopOpeBRepDSFilter::new(hds, Some(&mut self.my_shape_classifier))
            .process_face_interferences(&self.my_split_on)?;
        TopOpeBRepDSReducer::new(hds).process_face_interferences(&self.my_split_on)
    }

    /// The faces consumed by coincident curves and their edges not restricting a curve.
    fn find_consumed_faces(&mut self) -> Result<(), Failure> {
        self.my_consumed_faces.clear(true);
        self.my_consumed_face_edges.clear(true);
        let hds = self.my_data_structure.clone();
        let hds = hds
            .get()
            .ok_or_else(|| Failure::new("TopOpeBRepBuild_Builder::Perform: null data structure"))?;
        let mut reused = DataMap::default();
        {
            let ds = hds.ds();
            let mut it = TopOpeBRepDSCurveExplorer::new(&ds, false);
            while it.more() {
                let r = it.curve().existing_edge();
                if !r.is_null() && !reused.is_bound(r.as_ref()) {
                    reused.bind(r.clone().into_shape(), true);
                }
                it.next();
            }
        }
        for i in 1..=hds.nb_shapes() {
            let s = hds.shape(i, false);
            if !s.is_null()
                && s.shape_type()? == ShapeEnum::Face
                && is_consumed_by_coincident_curves(&s, hds)?
            {
                self.my_consumed_faces.bind(s.clone(), true);
                register_consumed_face_edges(&s, &reused, &mut self.my_consumed_face_edges)?;
            }
        }
        Ok(())
    }

    /// `Perform(HDS, S1, S2)` (virtual): `Perform(HDS)`, the two shapes kept and the
    /// particular case found; `b.perform_shapes(&hds, &s1, &s2)?`.
    pub fn perform_shapes(
        &mut self,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        s1: &Shape,
        s2: &Shape,
    ) -> Result<(), Failure> {
        if self.b1.is_some() {
            return self.b1_perform_shapes(hds, s1, s2);
        }
        self.perform_shapes_base(hds, s1, s2)
    }

    /// `TopOpeBRepBuild_Builder::Perform(HDS, S1, S2)`, the base version.
    pub(crate) fn perform_shapes_base(
        &mut self,
        hds: &Handle<TopOpeBRepDSHDataStructure>,
        s1: &Shape,
        s2: &Shape,
    ) -> Result<(), Failure> {
        self.perform(hds)?;
        self.my_shape1 = s1.clone();
        self.my_shape2 = s2.clone();
        self.my_is_k_part = self.find_is_k_part()?;
        Ok(())
    }

    /// `AddIntersectionEdges(F, ToBuild1, RevOri1, ES)`: the new edges of the curves of
    /// the face (not of a consumed group, not a boundary edge, not EXTERNAL), oriented
    /// and with their p-curve, as start elements of `wes`;
    /// `b.add_intersection_edges(&mut f, State::In, false, &mut wes)?`.
    pub fn add_intersection_edges(
        &self,
        a_face: &mut Shape,
        to_build1: State,
        rev_ori1: bool,
        wes: &mut dyn TopOpeBRepBuildShapeSet,
    ) -> Result<(), Failure> {
        let hds = self.hds()?;
        let mut consumed = DataMap::default();
        fill_equivalent_curve_orientation_masks(a_face, to_build1, rev_ori1, hds, &mut consumed)?;
        let mut fcurves = hds.face_curves(a_face)?;
        while fcurves.more() {
            let ic = fcurves.current()?;
            let c = hds.curve(ic);
            for edge in self.new_edges(ic).iter() {
                let mut an_edge = edge.clone();
                let rc = c.equivalent_curve();
                if rc > 0 && consumed.find(&rc).ok() == Some(&3) {
                    continue;
                }
                let existing = c.existing_edge();
                let is_boundary =
                    Explorer::new(a_face, ShapeEnum::Edge, ShapeEnum::Shape)?.any(|fe| {
                        an_edge.is_same(&fe)
                            || (!existing.is_null() && existing.as_ref().is_same(&fe))
                    });
                if is_boundary {
                    continue;
                }
                let mut newori = Self::orient(fcurves.orientation(to_build1)?, rev_ori1);
                if c.is_equivalent_curve_reversed() {
                    newori = top_abs::reverse(newori);
                }
                if newori == Orientation::External {
                    continue;
                }
                self.my_build_tool.set_orientation(&mut an_edge, newori);
                let pc = fcurves.pcurve()?;
                self.my_build_tool
                    .pcurve_of_curve(a_face, &mut an_edge, &c, &pc)?;
                wes.add_start_element(&an_edge)?;
            }
            fcurves.next()?;
        }
        Ok(())
    }

    /// `Clear()` (virtual): the split lists of the edges not section edges emptied and
    /// the merged maps cleared; `b.clear()?`.
    pub fn clear(&mut self) -> Result<(), Failure> {
        if self.b1.is_some() {
            return self.b1_clear();
        }
        self.clear_base()
    }

    /// `TopOpeBRepBuild_Builder::Clear()`, the base version.
    pub(crate) fn clear_base(&mut self) -> Result<(), Failure> {
        if let Some(hds) = self.my_data_structure.clone().get() {
            let bds = hds.ds();
            for m in [
                &mut self.my_split_out,
                &mut self.my_split_in,
                &mut self.my_split_on,
            ] {
                for (e, l) in m.items_mut() {
                    if e.shape_type()? == ShapeEnum::Edge
                        && !bds.is_section_edge(&topods::edge(e)?, true)
                    {
                        l.clear();
                    }
                }
            }
        }
        self.my_merged_out.clear(true);
        self.my_merged_in.clear(true);
        self.my_merged_on.clear(true);
        Ok(())
    }
}
