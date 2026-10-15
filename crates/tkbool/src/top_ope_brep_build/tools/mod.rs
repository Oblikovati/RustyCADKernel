// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_Tools.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_Tools`: the propagation of the states of the split shapes to the
//! shapes connected to them, the normals and tangents of the 2d pieces, the p-curve
//! updates of the faces and (Tools_1) the tolerance corrections.
mod geometry;
mod tolerances;

use crate::top_ope_brep_ds::{MapOfShapeWithState, TopOpeBRepDSShapeWithState};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeClassifier;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::{
    IndexedDataMapOfShapeListOfShape, IndexedMapOfShape, MapOfShape, ShapeMapHasher,
};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

/// The states of shapes (`TopTools_DataMapOfShapeState`).
pub type DataMapOfShapeState = DataMap<Shape, State, ShapeMapHasher>;

/// `PAR_T`: the relative parameter of the sample points of the edges.
pub(crate) const PAR_T: f64 = 0.43213918;

/// The tools of the builder (`TopOpeBRepBuild_Tools`, static methods only);
/// `TopOpeBRepBuildTools::is_deg_edges_the_same(&e1, &e2)?`.
pub struct TopOpeBRepBuildTools;

/// The sub-shapes of a kind of a shape, in exploration order.
fn sub_shapes(s: &Shape, kind: ShapeEnum) -> Result<IndexedMapOfShape, Failure> {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(s, kind, &mut map)?;
    Ok(map)
}

/// A shape with a state, not split.
fn with_state(state: State) -> TopOpeBRepDSShapeWithState {
    let mut sws = TopOpeBRepDSShapeWithState::default();
    sws.set_state(state);
    sws.set_is_splitted(false);
    sws
}

/// The first sub-shape of a kind not avoided (a null shape when none).
fn first_not_avoided(s: &Shape, kind: ShapeEnum, avoid: &MapOfShape) -> Result<Shape, Failure> {
    let map = sub_shapes(s, kind)?;
    Ok(map
        .iter()
        .find(|x| !avoid.contains(x))
        .cloned()
        .unwrap_or_default())
}

impl TopOpeBRepBuildTools {
    /// `FindState(aSubsh, aState, aSubshEnum, aMapSubshAnc, aMapProcessedSubsh, aMapSS)`:
    /// the state given to the unprocessed sub-shapes of the ancestors of `subsh`,
    /// recursively; `TopOpeBRepBuildTools::find_state(&v, State::In, ShapeEnum::Vertex, &anc, &mut done, &mut ss)?`.
    pub fn find_state(
        subsh: &Shape,
        state: State,
        subsh_enum: ShapeEnum,
        map_subsh_anc: &IndexedDataMapOfShapeListOfShape,
        processed: &mut MapOfShape,
        map_ss: &mut DataMapOfShapeState,
    ) -> Result<(), Failure> {
        for s in map_subsh_anc.find_from_key(subsh)?.iter() {
            for ss in sub_shapes(s, subsh_enum)?.iter() {
                if !processed.contains(ss) {
                    processed.add(ss.clone());
                    map_ss.bind(ss.clone(), state);
                    Self::find_state(ss, state, subsh_enum, map_subsh_anc, processed, map_ss)?;
                }
            }
        }
        Ok(())
    }

    /// `FindState2(...)`: the state given to the ancestors (edges) of the vertex `subsh`
    /// and, recursively, to their unprocessed vertices;
    /// `TopOpeBRepBuildTools::find_state2(&v, State::In, &anc, &mut done, &mut ss)?`.
    pub fn find_state2(
        subsh: &Shape,
        state: State,
        map_subsh_anc: &IndexedDataMapOfShapeListOfShape,
        processed: &mut MapOfShape,
        map_ss: &mut DataMapOfShapeState,
    ) -> Result<(), Failure> {
        for s in map_subsh_anc.find_from_key(subsh)?.iter() {
            map_ss.bind(s.clone(), state);
            for ss in sub_shapes(s, ShapeEnum::Vertex)?.iter() {
                if !processed.contains(ss) {
                    processed.add(ss.clone());
                    map_ss.bind(ss.clone(), state);
                    Self::find_state2(ss, state, map_subsh_anc, processed, map_ss)?;
                }
            }
        }
        Ok(())
    }

    /// `FindState1(...)`: the state given to the ancestors (faces) of the edge `subsh`,
    /// their wires and, recursively, their unprocessed edges;
    /// `TopOpeBRepBuildTools::find_state1(&e, State::In, &anc, &mut done, &mut ss)?`.
    pub fn find_state1(
        subsh: &Shape,
        state: State,
        map_subsh_anc: &IndexedDataMapOfShapeListOfShape,
        processed: &mut MapOfShape,
        map_ss: &mut DataMapOfShapeState,
    ) -> Result<(), Failure> {
        for s in map_subsh_anc.find_from_key(subsh)?.iter() {
            map_ss.bind(s.clone(), state);
            for w in sub_shapes(s, ShapeEnum::Wire)?.iter() {
                map_ss.bind(w.clone(), state);
            }
            for ss in sub_shapes(s, ShapeEnum::Edge)?.iter() {
                if !processed.contains(ss) {
                    processed.add(ss.clone());
                    map_ss.bind(ss.clone(), state);
                    Self::find_state1(ss, state, map_subsh_anc, processed, map_ss)?;
                }
            }
        }
        Ok(())
    }

    /// `FindStateThroughVertex(aShape, aShapeClassifier, aMapOfShapeWithState, anAvoidSubshMap)`:
    /// the state of the first vertex (else edge) not avoided by the classifier, given to
    /// the shape and its children; UNKNOWN when none;
    /// `TopOpeBRepBuildTools::find_state_through_vertex(&e, &mut sc, &mut sws, &avoid)?`.
    pub fn find_state_through_vertex(
        shape: &Shape,
        classifier: &mut TopOpeBRepToolShapeClassifier,
        map_sws: &mut MapOfShapeWithState,
        avoid: &MapOfShape,
    ) -> Result<State, Failure> {
        let mut subsh = first_not_avoided(shape, ShapeEnum::Vertex, avoid)?;
        if subsh.is_null() {
            subsh = first_not_avoided(shape, ShapeEnum::Edge, avoid)?;
            if subsh.is_null() {
                return Ok(State::Unknown);
            }
        }
        let state = classifier.state_shape_reference(&subsh, &Shape::default())?;
        map_sws.add(shape.clone(), with_state(state));
        Self::spread_state_to_child(shape, state, map_sws)?;
        Ok(state)
    }

    /// `SpreadStateToChild(aShape, aState, aMapOfShapeWithState)`: the faces, then the
    /// wires, then the edges of the shape added with the state;
    /// `TopOpeBRepBuildTools::spread_state_to_child(&f, State::In, &mut sws)?`.
    pub fn spread_state_to_child(
        shape: &Shape,
        state: State,
        map_sws: &mut MapOfShapeWithState,
    ) -> Result<(), Failure> {
        let mut children = IndexedMapOfShape::default();
        for kind in [ShapeEnum::Face, ShapeEnum::Wire, ShapeEnum::Edge] {
            topexp::map_shapes_of_type(shape, kind, &mut children)?;
        }
        for c in children.iter() {
            map_sws.add(c.clone(), with_state(state));
        }
        Ok(())
    }

    /// `PropagateStateForWires(aFacesToRestMap, aMapOfShapeWithState)`: the wires of the
    /// faces with a state, and their edges without one, added with the face state;
    /// `TopOpeBRepBuildTools::propagate_state_for_wires(&faces, &mut sws)?`.
    pub fn propagate_state_for_wires(
        faces: &IndexedMapOfShape,
        map_sws: &mut MapOfShapeWithState,
    ) -> Result<(), Failure> {
        for f in faces.iter() {
            if !map_sws.contains(f) {
                continue;
            }
            let st = map_sws.find_from_key(f)?.state();
            for w in sub_shapes(f, ShapeEnum::Wire)?.iter() {
                map_sws.add(w.clone(), with_state(st));
                for e in sub_shapes(w, ShapeEnum::Edge)?.iter() {
                    if !map_sws.contains(e) {
                        map_sws.add(e.clone(), with_state(st));
                    }
                }
            }
        }
        Ok(())
    }

    /// `PropagateState(aSplShapesState, aShapesToRestMap, aSubshEnum, aShapeEnum,
    /// aShapeClassifier, aMapOfShapeWithState, anAvoidSubshMap)`: the states of the
    /// split shapes spread through their sub-shapes to the connected shapes to rest;
    /// the shapes not reached are classified through a vertex and spread in turn;
    /// `TopOpeBRepBuildTools::propagate_state(&spl, &rest, ShapeEnum::Vertex, ShapeEnum::Edge, &mut sc, &mut sws, &avoid)?`.
    pub fn propagate_state(
        spl_states: &DataMapOfShapeState,
        rest: &IndexedMapOfShape,
        subsh_enum: ShapeEnum,
        shape_enum: ShapeEnum,
        classifier: &mut TopOpeBRepToolShapeClassifier,
        map_sws: &mut MapOfShapeWithState,
        avoid: &MapOfShape,
    ) -> Result<(), Failure> {
        let mut map_ss = DataMapOfShapeState::default();
        for (s, &state) in spl_states.items() {
            for sub in sub_shapes(s, subsh_enum)?.iter() {
                if !avoid.contains(sub) {
                    map_ss.bind(sub.clone(), state);
                }
            }
        }
        let map_ss1 = map_ss.clone();
        let mut anc = IndexedDataMapOfShapeListOfShape::default();
        for s in rest.iter() {
            topexp::map_shapes_and_ancestors(s, subsh_enum, shape_enum, &mut anc)?;
        }
        let mut processed = MapOfShape::default();
        for (subsh, &state) in map_ss1.items() {
            if anc.contains(subsh) {
                processed.add(subsh.clone());
                Self::find_state(subsh, state, subsh_enum, &anc, &mut processed, &mut map_ss)?;
            }
        }
        let non_passed = Self::rest_states(rest, subsh_enum, &map_ss, classifier, map_sws)?;
        if non_passed.extent() > 0 {
            Self::propagate_non_passed(
                &non_passed,
                subsh_enum,
                shape_enum,
                classifier,
                map_sws,
                avoid,
            )?;
        }
        Ok(())
    }

    /// The shapes to rest whose first sub-shape has a state added with it (ON shapes
    /// classified by the reference); returns the shapes not reached.
    fn rest_states(
        rest: &IndexedMapOfShape,
        subsh_enum: ShapeEnum,
        map_ss: &DataMapOfShapeState,
        classifier: &mut TopOpeBRepToolShapeClassifier,
        map_sws: &mut MapOfShapeWithState,
    ) -> Result<MapOfShape, Failure> {
        let mut non_passed = MapOfShape::default();
        for s in rest.iter() {
            let subsh = sub_shapes(s, subsh_enum)?.find_key(1)?.clone();
            if map_ss.is_bound(&subsh) {
                let mut state = *map_ss.find(&subsh)?;
                if state == State::On {
                    state = classifier.state_shape_reference(s, &Shape::default())?;
                }
                map_sws.add(s.clone(), with_state(state));
            } else {
                non_passed.add(s.clone());
            }
        }
        Ok(non_passed)
    }

    /// The shapes not reached: each classified through a vertex, its state spread by
    /// its first sub-shape not avoided to the connected shapes, then every shape (not a
    /// vertex) of the spread added with its state.
    fn propagate_non_passed(
        non_passed: &MapOfShape,
        subsh_enum: ShapeEnum,
        shape_enum: ShapeEnum,
        classifier: &mut TopOpeBRepToolShapeClassifier,
        map_sws: &mut MapOfShapeWithState,
        avoid: &MapOfShape,
    ) -> Result<(), Failure> {
        let mut anc = IndexedDataMapOfShapeListOfShape::default();
        for s in non_passed.iter() {
            topexp::map_shapes_and_ancestors(s, subsh_enum, shape_enum, &mut anc)?;
        }
        let mut map_ss = DataMapOfShapeState::default();
        for s in non_passed.iter() {
            if map_ss.is_bound(s) {
                continue;
            }
            let state = Self::find_state_through_vertex(s, classifier, map_sws, avoid)?;
            map_ss.bind(s.clone(), state);
            let first = first_not_avoided(s, subsh_enum, avoid)?;
            if first.is_null() {
                continue;
            }
            map_ss.bind(first.clone(), state);
            let mut processed = MapOfShape::default();
            if subsh_enum == ShapeEnum::Edge {
                Self::find_state1(&first, state, &anc, &mut processed, &mut map_ss)?;
            } else {
                Self::find_state2(&first, state, &anc, &mut processed, &mut map_ss)?;
            }
        }
        for (s, &state) in map_ss.items() {
            if s.shape_type()? != ShapeEnum::Vertex {
                map_sws.add(s.clone(), with_state(state));
            }
        }
        Ok(())
    }
}
