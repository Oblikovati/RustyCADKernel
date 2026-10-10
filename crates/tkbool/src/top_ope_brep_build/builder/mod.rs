// Rust translation of the constructor and accessors of
// ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_Builder.hxx/.cxx,
// InitSection of TopOpeBRepBuild_Section.cxx and Classify/ChangeClassify of
// TopOpeBRepBuild_Merge.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_Builder`: the builder of the result shapes of a topological
//! operation from a data structure; the members of the parts not ported yet (the
//! splits, the merges and the section) arrive with them.
mod merge;

use crate::top_ope_brep_ds::{TopOpeBRepDSBuildTool, TopOpeBRepDSHDataStructure};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Handle;

/// The builder of topological operations (`TopOpeBRepBuild_Builder`);
/// `let mut b = TopOpeBRepBuildBuilder::new(&TopOpeBRepDSBuildTool::new());`.
pub struct TopOpeBRepBuildBuilder {
    my_build_tool: TopOpeBRepDSBuildTool,
    /// `myDataStructure`: null until Perform.
    my_data_structure: Handle<TopOpeBRepDSHDataStructure>,
    #[allow(dead_code)] // read by the parts of the builder not ported yet
    my_section_done: bool,
    my_section: List<Shape>,
    #[allow(dead_code)] // read by the parts of the builder not ported yet
    my_split_section_edges_done: bool,
    #[allow(dead_code)] // read by the parts of the builder not ported yet
    my_is_k_part: i32,
    #[allow(dead_code)] // read by the parts of the builder not ported yet
    my_classify_def: bool,
    my_classify_val: bool,
    #[allow(dead_code)] // read by the parts of the builder not ported yet
    my_process_on: bool,
}

impl TopOpeBRepBuildBuilder {
    /// `TopOpeBRepBuild_Builder(BT)`: a copy of the build tool, no data structure,
    /// classification on, then `InitSection()`;
    /// `TopOpeBRepBuildBuilder::new(&TopOpeBRepDSBuildTool::new())`.
    pub fn new(bt: &TopOpeBRepDSBuildTool) -> Self {
        let mut b = Self {
            my_build_tool: *bt,
            my_data_structure: Handle::default(),
            my_section_done: false,
            my_section: List::default(),
            my_split_section_edges_done: false,
            my_is_k_part: 0,
            my_classify_def: false,
            my_classify_val: true,
            my_process_on: false,
        };
        b.init_section();
        b
    }

    /// `InitSection()`: no section done and no section edge; `b.init_section()`.
    pub fn init_section(&mut self) {
        self.my_section_done = false;
        self.my_section.clear();
        self.my_split_section_edges_done = false;
    }

    /// `ChangeBuildTool()`; `b.change_build_tool()`.
    pub fn change_build_tool(&mut self) -> &mut TopOpeBRepDSBuildTool {
        &mut self.my_build_tool
    }

    /// `BuildTool()`; `b.build_tool()`.
    pub fn build_tool(&self) -> &TopOpeBRepDSBuildTool {
        &self.my_build_tool
    }

    /// `DataStructure()`: null until Perform; `b.data_structure()`.
    pub fn data_structure(&self) -> Handle<TopOpeBRepDSHDataStructure> {
        self.my_data_structure.clone()
    }

    /// `Classify()`; `b.classify()`.
    pub fn classify(&self) -> bool {
        self.my_classify_val
    }

    /// `ChangeClassify(classify)`: a defined classification flag; `b.change_classify(false)`.
    pub fn change_classify(&mut self, classify: bool) {
        self.my_classify_def = true;
        self.my_classify_val = classify;
    }
}
