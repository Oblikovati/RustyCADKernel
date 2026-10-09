// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_WireToFace.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepBuild_WireToFace`: the faces bounded by a set of wires on the surface
//! of a face.
use super::builder::TopOpeBRepBuildBuilder;
use super::face_builder::TopOpeBRepBuildFaceBuilder;
use super::shape_set::TopOpeBRepBuildShapeSet;
use super::wire_edge_set::TopOpeBRepBuildWireEdgeSet;
use crate::top_ope_brep_ds::TopOpeBRepDSBuildTool;
use crate::top_ope_brep_tool::TopOpeBRepToolOutCurveType;
use rustycad_tkbrep::topods::{Face, Shape, Wire};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;

/// The builder of faces from wires (`TopOpeBRepBuild_WireToFace`); every edge needs
/// a p-curve on the surface of the face unless it is planar;
/// `let mut wtf = TopOpeBRepBuildWireToFace::new(); wtf.add_wire(&w); wtf.make_faces(&f, &mut lf)?;`.
#[derive(Clone, Default)]
pub struct TopOpeBRepBuildWireToFace {
    my_lw: List<Shape>,
}

impl TopOpeBRepBuildWireToFace {
    /// `TopOpeBRepBuild_WireToFace()`; `TopOpeBRepBuildWireToFace::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init()`: no wire; `wtf.init()`.
    pub fn init(&mut self) {
        self.my_lw.clear();
    }

    /// `AddWire(W)`; `wtf.add_wire(&w)`.
    pub fn add_wire(&mut self, w: &Wire) {
        self.my_lw.append(w.as_ref().clone());
    }

    /// `MakeFaces(F, LF)`: the faces of the wires on `f`, classified with forced
    /// classification and made by a builder with an APPROX build tool;
    /// `wtf.make_faces(&face, &mut lf)?`.
    pub fn make_faces(&self, f: &Face, lf: &mut List<Shape>) -> Result<(), Failure> {
        lf.clear();
        let mut wes = TopOpeBRepBuildWireEdgeSet::new(f.as_ref())?;
        for w in self.my_lw.iter() {
            wes.add_shape(w)?;
        }
        let force_class = true;
        let mut fb = TopOpeBRepBuildFaceBuilder::new();
        fb.init_face_builder(&mut wes, f.as_ref(), force_class)?;
        let bt = TopOpeBRepDSBuildTool::with_type(TopOpeBRepToolOutCurveType::Approx);
        TopOpeBRepBuildBuilder::new(&bt).make_faces(f.as_ref(), &mut fb, lf)
    }
}
