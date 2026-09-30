//! Original BRepGraph_ScenarioMatrix_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_graph::{
        editor::{EdgeOps, FaceOps, VertexOps},
        tool::{co_edge, edge, vertex},
        validate::{self, Options, Severity},
        *,
    },
    brep_graph_inc::{ParityOrientation, populate},
    topexp::{self, Explorer},
    topods::{self, CompSolid, Compound, Shape, ShapeIterator, Vertex, Wire},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::{
    gp::{Direction2, Direction3, Point2, Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere,
};
use rustycad_tktopalgo::{
    brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape},
    brep_gprop::BRepGProp,
};
include!("graph_scenarios/fixtures.rs");
include!("graph_scenarios/mutations.rs");
include!("graph_scenarios/containers.rs");
include!("graph_scenarios/assemblies.rs");
include!("graph_scenarios/seams.rs");
include!("graph_scenarios/editor.rs");
