//! The states of the shapes of `TopOpeBRepBuild_Builder1` (Builder1_1.cxx, new
//! builder): the shells, faces, wires and edges of each shape classified against the
//! other, the edges with interferences split with states.
use super::split_edge::classify_edge_to_solid_by_one_point;
use super::{PROCESSED_EDGES, UNK_STATE_VER, USED_VERTEX_MAP};
use crate::top_ope_brep_build::globals::GLOBAL_FACES2D;
use crate::top_ope_brep_build::tools::DataMapOfShapeState;
use crate::top_ope_brep_build::{TopOpeBRepBuildBuilder, TopOpeBRepBuildTools};
use crate::top_ope_brep_ds::{
    MapOfShapeWithState, TopOpeBRepDSListOfShapeOn1State, TopOpeBRepDSShapeWithState,
};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeClassifier;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::{IndexedMapOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// The sub-shapes of a kind.
fn subs(s: &Shape, kind: ShapeEnum) -> Result<IndexedMapOfShape, Failure> {
    let mut m = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, kind, &mut m)?;
    Ok(m)
}

/// The two vertices of an edge added to the used vertices.
fn use_vertices(e: &Shape) -> Result<(), Failure> {
    let (vf, vl) = topexp::vertices(&topods::edge(e)?, false);
    USED_VERTEX_MAP.with_borrow_mut(|m| {
        m.add(vf.into());
        m.add(vl.into());
    });
    Ok(())
}

impl TopOpeBRepBuildBuilder {
    /// The map of shapes with states of the shape (None when it is of no operand) with
    /// the shape classifier.
    fn with_sws<R>(
        &mut self,
        obj: &Shape,
        f: impl FnOnce(
            &mut MapOfShapeWithState,
            &mut TopOpeBRepToolShapeClassifier,
        ) -> Result<R, Failure>,
    ) -> Result<Option<R>, Failure> {
        let h = self.my_data_structure.clone();
        let hds = h.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "no data structure",
                "TopOpeBRepBuild_Builder1",
            )
        })?;
        let mut ds = hds.change_ds();
        let (map, flag) = ds.change_map_of_shape_with_state(obj);
        if !flag {
            return Ok(None);
        }
        f(map, &mut self.my_shape_classifier).map(Some)
    }

    /// The used vertices of the new edges and of the splits of the section edges.
    fn fill_used_vertices(&mut self) -> Result<(), Failure> {
        let nbc = self.hds()?.ds().nb_curves();
        for i in 1..=nbc {
            for e in self.change_new_edges(i).clone().iter() {
                use_vertices(e)?;
            }
        }
        let n = self.hds()?.ds().nb_section_edges();
        for i in 1..=n {
            let e: Shape = self.hds()?.ds().section_edge(i, true)?.into();
            if e.is_null() {
                continue;
            }
            for st in [State::On, State::In, State::Out] {
                for s in self.splits(&e, st).clone().iter() {
                    use_vertices(s)?;
                }
            }
        }
        Ok(())
    }

    /// The rejected faces of a shape added OUT with their wires and edges.
    fn add_rejected(
        map: &mut MapOfShapeWithState,
        rejected: &IndexedMapOfShape,
    ) -> Result<(), Failure> {
        let mut sws = TopOpeBRepDSShapeWithState::default();
        sws.set_is_splitted(false);
        sws.set_state(State::Out);
        for f in rejected.iter() {
            if f.shape_type()? != ShapeEnum::Face {
                continue;
            }
            for w in subs(f, ShapeEnum::Wire)?.iter() {
                for e in subs(w, ShapeEnum::Edge)?.iter() {
                    map.add(e.clone(), sws.clone());
                }
                map.add(w.clone(), sws.clone());
            }
            map.add(f.clone(), sws.clone());
        }
        Ok(())
    }

    /// `PerformShapeWithStates()` (Builder1): the used vertices noted, the maps of shapes
    /// with states restarted with the operands and their rejected faces, then each
    /// operand classified against the other; `b.perform_shape_with_states()?`.
    pub fn perform_shape_with_states(&mut self) -> Result<(), Failure> {
        USED_VERTEX_MAP.with_borrow_mut(|m| m.clear(true));
        UNK_STATE_VER.with_borrow_mut(|m| m.clear(true));
        {
            let mut ds = self.hds()?.change_ds();
            ds.change_map_of_shape_with_state_obj().clear(true);
            ds.change_map_of_shape_with_state_tool().clear(true);
        }
        self.fill_used_vertices()?;
        {
            let (s1, s2) = (self.my_shape1.clone(), self.my_shape2.clone());
            let mut ds = self.hds()?.change_ds();
            ds.change_map_of_shape_with_state_obj()
                .add(s1, TopOpeBRepDSShapeWithState::default());
            ds.change_map_of_shape_with_state_tool()
                .add(s2, TopOpeBRepDSShapeWithState::default());
            let rej_obj = ds.change_map_of_rejected_shapes_obj().clone();
            let rej_tool = ds.change_map_of_rejected_shapes_tool().clone();
            Self::add_rejected(ds.change_map_of_shape_with_state_obj(), &rej_obj)?;
            Self::add_rejected(ds.change_map_of_shape_with_state_tool(), &rej_tool)?;
        }
        let (s1, s2) = (self.my_shape1.clone(), self.my_shape2.clone());
        self.perform_shape_with_states_of(&s1, &s2)?;
        PROCESSED_EDGES.with_borrow_mut(|m| m.clear(true));
        self.perform_shape_with_states_of(&s2, &s1)?;
        PROCESSED_EDGES.with_borrow_mut(|m| m.clear(true));
        Ok(())
    }

    /// The state of a shape through a vertex, spread to its children.
    fn state_through_vertex(&mut self, obj: &Shape, s: &Shape) -> Result<(), Failure> {
        let avoid = UNK_STATE_VER.with_borrow(|m| m.clone());
        self.with_sws(obj, |map, sc| {
            TopOpeBRepBuildTools::find_state_through_vertex(s, sc, map, &avoid)
        })?;
        Ok(())
    }

    /// `PerformShapeWithStates(anObj, aReference)` (Builder1): each shell of the shape
    /// not yet with a state classified through a vertex (without interferences) or by
    /// its faces: those with interferences by their edges, the others by propagation;
    /// `b.perform_shape_with_states_of(&s1, &s2)?`.
    pub fn perform_shape_with_states_of(
        &mut self,
        an_obj: &Shape,
        a_reference: &Shape,
    ) -> Result<(), Failure> {
        self.my_shape_classifier.set_reference(a_reference)?;
        if self.with_sws(an_obj, |_, _| Ok(()))?.is_none() {
            return Ok(());
        }
        for shell in subs(an_obj, ShapeEnum::Shell)?.iter() {
            if self.with_sws(an_obj, |m, _| Ok(m.contains(shell)))? == Some(true) {
                continue;
            }
            if !self.hds()?.has_shape(shell, true) {
                self.state_through_vertex(an_obj, shell)?;
                continue;
            }
            self.shell_states(an_obj, shell)?;
        }
        Ok(())
    }

    /// The states of the faces of a shell: the edges of the faces with a state noted,
    /// the faces with interferences by their edges, the others propagated.
    fn shell_states(&mut self, an_obj: &Shape, shell: &Shape) -> Result<(), Failure> {
        let (mut with_interf, mut rest) =
            (IndexedMapOfShape::default(), IndexedMapOfShape::default());
        let mut spl_faces_state = DataMapOfShapeState::default();
        for face in subs(shell, ShapeEnum::Face)?.iter() {
            let edges = subs(face, ShapeEnum::Edge)?;
            let noted = self.with_sws(an_obj, |m, _| {
                if !m.contains(face) {
                    return Ok(false);
                }
                for e in edges.iter() {
                    spl_faces_state.bind(e.clone(), m.find_from_key(e)?.state());
                }
                Ok(true)
            })?;
            if noted == Some(true) {
                continue;
            }
            if self.hds()?.has_shape(face, true) {
                with_interf.add(face.clone());
            } else {
                rest.add(face.clone());
            }
        }
        self.perform_faces_with_states(an_obj, &with_interf, &mut spl_faces_state)?;
        self.with_sws(an_obj, |m, sc| {
            TopOpeBRepBuildTools::propagate_state(
                &spl_faces_state,
                &rest,
                ShapeEnum::Edge,
                ShapeEnum::Face,
                sc,
                m,
                &MapOfShape::default(),
            )?;
            TopOpeBRepBuildTools::propagate_state_for_wires(&rest, m)
        })?;
        Ok(())
    }

    /// `PerformFacesWithStates(anObj, aFaces, aSplF)` (Builder1): the wires of each face
    /// without interferences classified through a vertex, the edges of the others split
    /// with states (accumulated over the wires of the face) and the states of the
    /// others noted; `b.perform_faces_with_states(&s, &faces, &mut states)?`.
    pub fn perform_faces_with_states(
        &mut self,
        an_obj: &Shape,
        faces: &IndexedMapOfShape,
        spl_faces_state: &mut DataMapOfShapeState,
    ) -> Result<(), Failure> {
        if self.with_sws(an_obj, |_, _| Ok(()))?.is_none() {
            return Ok(());
        }
        for face in faces.iter() {
            let (mut to_split, mut to_rest) =
                (IndexedMapOfShape::default(), IndexedMapOfShape::default());
            for wire in subs(face, ShapeEnum::Wire)?.iter() {
                if !self.hds()?.has_shape(wire, true) {
                    self.state_through_vertex(an_obj, wire)?;
                    continue;
                }
                for e in subs(wire, ShapeEnum::Edge)?.iter() {
                    if self.hds()?.has_shape(e, true) {
                        to_split.add(e.clone());
                    } else {
                        to_rest.add(e.clone());
                    }
                }
                self.status_edges_to_split(an_obj, &to_split, &to_rest)?;
                self.with_sws(an_obj, |m, _| {
                    for e in to_rest.iter() {
                        if m.contains(e) {
                            spl_faces_state.bind(e.clone(), m.find_from_key(e)?.state());
                        }
                    }
                    Ok(())
                })?;
            }
        }
        Ok(())
    }

    /// The states of the splits of an edge already processed (when split).
    fn processed_edge_states(
        &mut self,
        an_obj: &Shape,
        e: &Shape,
        states: &mut DataMapOfShapeState,
    ) -> Result<(), Failure> {
        self.with_sws(an_obj, |m, _| {
            if !m.contains(e) || !m.find_from_key(e)?.is_splitted() {
                return Ok(());
            }
            let sws = m.find_from_key(e)?;
            for st in [State::On, State::Out, State::In] {
                for s in sws.part(st).iter() {
                    states.bind(s.clone(), st);
                }
            }
            Ok(())
        })?;
        Ok(())
    }

    /// The ON, IN then OUT splits of an edge split ON, oriented as the edge, with their
    /// states.
    fn existing_splits_states(
        &self,
        e: &Shape,
        sws: &mut TopOpeBRepDSShapeWithState,
        states: &mut DataMapOfShapeState,
    ) {
        for st in [State::On, State::In, State::Out] {
            for s in self.splits(e, st).iter() {
                sws.add_part(&s.oriented(e.orientation()), st);
                states.bind(s.clone(), st);
            }
        }
        sws.set_is_splitted(true);
    }

    /// The vertices of a part noted as used, those whose 3d state differs (not ON)
    /// from the part's as of unknown state.
    fn note_part_vertices(
        &mut self,
        part: &Shape,
        state: State,
        check: bool,
    ) -> Result<(), Failure> {
        let (vf, vl) = topexp::vertices(&topods::edge(part)?, false);
        use_vertices(part)?;
        if !check {
            return Ok(());
        }
        for (k, v) in [vf.clone(), vl.clone()].iter().enumerate() {
            if k == 1 && vf.is_same(&vl) {
                break;
            }
            self.my_shape_classifier
                .state_p3d_reference(BRepTool::pnt(v)?)?;
            let st = self.my_shape_classifier.state();
            if st != state && st != State::On {
                UNK_STATE_VER.with_borrow_mut(|m| m.add(v.clone().into()));
            }
        }
        Ok(())
    }

    /// The unsplit edge rebuilt on the reference vertices of its same domain vertices
    /// (unless the edge has a same domain), else itself.
    fn unsplit_edge(&self, e: &Shape) -> Result<Shape, Failure> {
        let (mut vf, mut vl) = topexp::vertices(&topods::edge(e)?, false);
        let mut sd1 = self.hds()?.has_same_domain(&vf, true);
        let mut sd2 = self.hds()?.has_same_domain(&vl, true);
        if self.hds()?.has_same_domain(e, true) {
            (sd1, sd2) = (false, false);
        }
        if !sd1 && !sd2 {
            return Ok(e.clone());
        }
        let eor = topods::edge(&e.oriented(Orientation::Forward))?;
        let (par_f, par_l) = (
            BRepTool::parameter(&vf, &eor)?,
            BRepTool::parameter(&vl, &eor)?,
        );
        let mut new_e = Shape::default();
        self.my_build_tool.copy_edge(&eor, &mut new_e)?;
        if sd1 {
            vf = topods::vertex(
                &self
                    .hds()?
                    .shape(self.hds()?.same_domain_reference(&vf), true)
                    .oriented(Orientation::Forward),
            )?;
        }
        if sd2 {
            vl = topods::vertex(
                &self
                    .hds()?
                    .shape(self.hds()?.same_domain_reference(&vl), true)
                    .oriented(Orientation::Reversed),
            )?;
        }
        self.my_build_tool.add_edge_vertex(&mut new_e, &vf)?;
        self.my_build_tool.parameter(&new_e, &vf, par_f)?;
        self.my_build_tool.add_edge_vertex(&mut new_e, &vl)?;
        self.my_build_tool.parameter(&new_e, &vl, par_l)?;
        Ok(new_e.oriented(e.orientation()))
    }

    /// The parts of an edge not split ON: split with states (in 2d), the IN parts
    /// classified in 3d; the edge itself (on reference vertices) classified when no part.
    fn new_splits_states(
        &mut self,
        e: &Shape,
        sws: &mut TopOpeBRepDSShapeWithState,
        states: &mut DataMapOfShapeState,
    ) -> Result<(), Failure> {
        let (mut l_new, mut map_state) = (List::default(), DataMapOfShapeState::default());
        let old = GLOBAL_FACES2D.replace(true);
        let r = self.split_edge_with_states(e, &mut l_new, &mut map_state);
        GLOBAL_FACES2D.set(old);
        r?;
        if l_new.is_empty() {
            let new_e = self.unsplit_edge(e)?;
            let st = classify_edge_to_solid_by_one_point(
                &topods::edge(&new_e)?,
                &mut self.my_shape_classifier,
            )?;
            sws.set_is_splitted(true);
            sws.add_part(&new_e, st);
            states.bind(new_e.clone(), st);
            let degenerated = BRepTool::degenerated(&topods::edge(&new_e)?)?;
            return self.note_part_vertices(&new_e, st, !degenerated);
        }
        for s in l_new.iter() {
            let mut st = *map_state.find(s)?;
            let isdegen = BRepTool::degenerated(&topods::edge(s)?)?;
            if st == State::In && !isdegen {
                st = classify_edge_to_solid_by_one_point(
                    &topods::edge(s)?,
                    &mut self.my_shape_classifier,
                )?;
            }
            sws.add_part(s, st);
            sws.set_is_splitted(true);
            states.bind(s.clone(), st);
            self.note_part_vertices(s, st, !isdegen)?;
        }
        Ok(())
    }

    /// The states of one edge to split, its parts kept in the map of shapes with states;
    /// its new ON parts made ON splits of a new section edge.
    fn status_edge_to_split(
        &mut self,
        an_obj: &Shape,
        e: &Shape,
        states: &mut DataMapOfShapeState,
    ) -> Result<(), Failure> {
        PROCESSED_EDGES.with_borrow_mut(|m| m.add(e.clone()));
        let mut sws = TopOpeBRepDSShapeWithState::default();
        let is_split_on = self.is_split(e, State::On);
        if is_split_on {
            self.existing_splits_states(e, &mut sws, states);
            self.with_sws(an_obj, |m, _| Ok(m.add(e.clone(), sws)))?;
            return Ok(());
        }
        self.new_splits_states(e, &mut sws, states)?;
        let esp_on = sws.part(State::On);
        if !esp_on.is_empty() {
            let mut on_spl = TopOpeBRepDSListOfShapeOn1State::default();
            *on_spl.change_list_on_state() = esp_on;
            on_spl.split(true);
            self.my_split_on.bind(e.clone(), on_spl);
            self.hds()?.change_ds().add_section_edge(&topods::edge(e)?);
        }
        self.with_sws(an_obj, |m, _| Ok(m.add(e.clone(), sws)))?;
        Ok(())
    }

    /// `StatusEdgesToSplit(anObj, anEdgesToSplitMap, anEdgesToRestMap)` (Builder1): each
    /// edge to split (not yet processed) split with states, the states of the edges to
    /// rest noted (unsplit) and propagated through the vertices;
    /// `b.status_edges_to_split(&s, &split, &rest)?`.
    pub fn status_edges_to_split(
        &mut self,
        an_obj: &Shape,
        to_split: &IndexedMapOfShape,
        to_rest: &IndexedMapOfShape,
    ) -> Result<(), Failure> {
        if self.with_sws(an_obj, |_, _| Ok(()))?.is_none() || to_split.extent() == 0 {
            return Ok(());
        }
        let mut states = DataMapOfShapeState::default();
        for e in to_split.iter() {
            if PROCESSED_EDGES.with_borrow(|m| m.contains(e)) {
                self.processed_edge_states(an_obj, e, &mut states)?;
                continue;
            }
            self.status_edge_to_split(an_obj, e, &mut states)?;
        }
        self.with_sws(an_obj, |m, _| {
            for e in to_rest.iter() {
                if m.contains(e) && !m.find_from_key(e)?.is_splitted() {
                    states.bind(e.clone(), m.find_from_key(e)?.state());
                }
            }
            Ok(())
        })?;
        if to_rest.extent() > 0 {
            let avoid = UNK_STATE_VER.with_borrow(|m| m.clone());
            self.with_sws(an_obj, |m, sc| {
                TopOpeBRepBuildTools::propagate_state(
                    &states,
                    to_rest,
                    ShapeEnum::Vertex,
                    ShapeEnum::Edge,
                    sc,
                    m,
                    &avoid,
                )
            })?;
        }
        Ok(())
    }
}
