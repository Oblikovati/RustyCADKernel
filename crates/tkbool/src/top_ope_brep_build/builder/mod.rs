// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_Builder.hxx
// and its sources (TopOpeBRepBuild_Builder.cxx, _BuildEdges, _BuildFaces, _End, _FREGU,
// _ffsfs, _ffwesk, _fctwes, _Grid, _GridEE, _GridFF, _GridSS, _Griddump, _KPart, _kpkole,
// _keep, _makeedges, _makefaces, _makesolids, _Merge, _Section, _SREGU) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_Builder`: the builder of the result shapes of a topological
//! operation from a data structure. The virtual members overridden by
//! `TopOpeBRepBuild_Builder1` dispatch on the extension data it installs.
mod accessors;
mod build;
mod build_edges;
mod end;
mod ffsfs;
mod fill;
mod fill_wes;
mod grid;
mod grid_ee;
mod grid_ee_paves;
mod grid_ff;
mod grid_ff_edges;
mod grid_ss;
mod grid_ss_on;
mod grid_ss_on_patches;
mod kpart;
mod kpart_disj;
mod kpart_disj_merge;
mod kpart_koletge;
mod kpart_koletge_is;
mod kpkole;
mod kpkole_merge;
mod make_edges_solids;
mod make_faces;
mod merge;
mod merge_shapes;
mod perform;
mod regu;
mod section;
mod section_edge;
mod section_iso;
mod split;
mod split_eviso;

pub(crate) use build_edges::curve_indices;
pub use ffsfs::{fun_ismotheropedef, fun_motherope, fun_setmotherope, fun_unsetmotherope};
pub(crate) use grid_ee::fun_aresamegeom;
pub use kpart::kpresu;

use crate::top_ope_brep_ds::{
    MapOfShapeOn1State, TopOpeBRepDSBuildTool, TopOpeBRepDSHDataStructure,
};
use crate::top_ope_brep_tool::TopOpeBRepToolShapeClassifier;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Solid};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, IndexedMap, List, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;

/// The shapes by shape (`NCollection_DataMap<TopoDS_Shape, NCollection_List<TopoDS_Shape>>`).
pub type DataMapOfShapeListOfShape = DataMap<Shape, List<Shape>, ShapeMapHasher>;

/// The builder of topological operations (`TopOpeBRepBuild_Builder`);
/// `let mut b = TopOpeBRepBuildBuilder::new(&TopOpeBRepDSBuildTool::new());`.
pub struct TopOpeBRepBuildBuilder {
    pub(crate) my_state1: State,
    pub(crate) my_state2: State,
    pub(crate) my_shape1: Shape,
    pub(crate) my_shape2: Shape,
    /// `myDataStructure`: null until Perform.
    pub(crate) my_data_structure: Handle<TopOpeBRepDSHDataStructure>,
    pub(crate) my_build_tool: TopOpeBRepDSBuildTool,
    /// `myNewVertices` (indices 0 to the point count, empty until BuildVertices).
    pub(crate) my_new_vertices: Vec<Shape>,
    pub(crate) my_new_edges: DataMap<i32, List<Shape>>,
    pub(crate) my_coincident_edges: DataMap<Shape, Shape, ShapeMapHasher>,
    pub(crate) my_consumed_faces: DataMap<Shape, bool, ShapeMapHasher>,
    pub(crate) my_consumed_face_edges: DataMap<Shape, bool, ShapeMapHasher>,
    /// `myNewFaces` (indices 0 to the surface count, empty until BuildFaces).
    pub(crate) my_new_faces: Vec<List<Shape>>,
    pub(crate) my_split_in: MapOfShapeOn1State,
    pub(crate) my_split_on: MapOfShapeOn1State,
    pub(crate) my_split_out: MapOfShapeOn1State,
    pub(crate) my_merged_in: MapOfShapeOn1State,
    pub(crate) my_merged_on: MapOfShapeOn1State,
    pub(crate) my_merged_out: MapOfShapeOn1State,
    pub(crate) my_empty_shape_list: List<Shape>,
    #[allow(dead_code)] // native member, never read natively either
    pub(crate) my_list_of_solid: List<Shape>,
    pub(crate) my_list_of_face: List<Shape>,
    #[allow(dead_code)] // native member, never read natively either
    pub(crate) my_list_of_edge: List<Shape>,
    pub(crate) my_f_splits: DataMapOfShapeListOfShape,
    pub(crate) my_e_splits: DataMapOfShapeListOfShape,
    pub(crate) my_section_done: bool,
    pub(crate) my_split_section_edges_done: bool,
    pub(crate) my_section: List<Shape>,
    pub(crate) my_solid_reference: Solid,
    pub(crate) my_solid_to_fill: Solid,
    #[allow(dead_code)] // native member, never read natively either
    pub(crate) my_face_avoid: List<Shape>,
    pub(crate) my_face_reference: Face,
    pub(crate) my_face_to_fill: Face,
    pub(crate) my_edge_avoid: List<Shape>,
    pub(crate) my_edge_reference: Edge,
    #[allow(dead_code)] // native member, never read natively either
    pub(crate) my_edge_to_fill: Edge,
    #[allow(dead_code)] // native member, never read natively either
    pub(crate) my_vertex_avoid: List<Shape>,
    pub(crate) my_map1: IndexedMap<Shape, ShapeMapHasher>,
    pub(crate) my_map2: IndexedMap<Shape, ShapeMapHasher>,
    pub(crate) my_is_k_part: i32,
    pub(crate) my_kp_map_f1f2: DataMapOfShapeListOfShape,
    #[allow(dead_code)] // native member, never read natively either
    pub(crate) my_shaset_index: i32,
    pub(crate) my_classify_def: bool,
    pub(crate) my_classify_val: bool,
    pub(crate) my_shape_classifier: TopOpeBRepToolShapeClassifier,
    pub(crate) my_memo_split: Map<Shape, ShapeMapHasher>,
    pub(crate) my_process_on: bool,
    pub(crate) my_on_faces_map: IndexedDataMap<Shape, Shape, ShapeMapHasher>,
    pub(crate) my_on_elem_map: IndexedMap<Shape>,
    /// `myEmptyAS`: the empty name of the release dumps.
    #[allow(dead_code)] // native member, read only by the debug dumps
    pub(crate) my_empty_as: String,
    /// The members of `TopOpeBRepBuild_Builder1` when this builder is one.
    pub(crate) b1: Option<Box<crate::top_ope_brep_build::builder1::Builder1Data>>,
}

impl TopOpeBRepBuildBuilder {
    /// `TopOpeBRepBuild_Builder(BT)`: a copy of the build tool, no data structure,
    /// classification on, then `InitSection()`;
    /// `TopOpeBRepBuildBuilder::new(&TopOpeBRepDSBuildTool::new())`.
    pub fn new(bt: &TopOpeBRepDSBuildTool) -> Self {
        let mut b = Self {
            my_state1: State::Unknown,
            my_state2: State::Unknown,
            my_shape1: Shape::default(),
            my_shape2: Shape::default(),
            my_data_structure: Handle::default(),
            my_build_tool: *bt,
            my_new_vertices: Vec::new(),
            my_new_edges: DataMap::default(),
            my_coincident_edges: DataMap::default(),
            my_consumed_faces: DataMap::default(),
            my_consumed_face_edges: DataMap::default(),
            my_new_faces: Vec::new(),
            my_split_in: MapOfShapeOn1State::default(),
            my_split_on: MapOfShapeOn1State::default(),
            my_split_out: MapOfShapeOn1State::default(),
            my_merged_in: MapOfShapeOn1State::default(),
            my_merged_on: MapOfShapeOn1State::default(),
            my_merged_out: MapOfShapeOn1State::default(),
            my_empty_shape_list: List::default(),
            my_list_of_solid: List::default(),
            my_list_of_face: List::default(),
            my_list_of_edge: List::default(),
            my_f_splits: DataMap::default(),
            my_e_splits: DataMap::default(),
            my_section_done: false,
            my_split_section_edges_done: false,
            my_section: List::default(),
            my_solid_reference: Solid::default(),
            my_solid_to_fill: Solid::default(),
            my_face_avoid: List::default(),
            my_face_reference: Face::default(),
            my_face_to_fill: Face::default(),
            my_edge_avoid: List::default(),
            my_edge_reference: Edge::default(),
            my_edge_to_fill: Edge::default(),
            my_vertex_avoid: List::default(),
            my_map1: IndexedMap::default(),
            my_map2: IndexedMap::default(),
            my_is_k_part: 0,
            my_kp_map_f1f2: DataMap::default(),
            my_shaset_index: 0,
            my_classify_def: false,
            my_classify_val: true,
            my_shape_classifier: TopOpeBRepToolShapeClassifier::new(),
            my_memo_split: Map::default(),
            my_process_on: false,
            my_on_faces_map: IndexedDataMap::default(),
            my_on_elem_map: IndexedMap::default(),
            my_empty_as: String::new(),
            b1: None,
        };
        b.init_section();
        b
    }

    /// The data structure, Standard_NullObject before Perform.
    pub(crate) fn hds(&self) -> Result<&TopOpeBRepDSHDataStructure, Failure> {
        self.my_data_structure.get().ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "no data structure",
                "TopOpeBRepBuild_Builder",
            )
        })
    }
}
