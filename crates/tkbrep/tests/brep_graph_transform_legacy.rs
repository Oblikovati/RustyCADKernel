//! Original BRepGraph_Transform_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    brep_graph::{
        BRepGraph, ChildRefId, CoEdgeId, EdgeId, FaceId, NodeId, NodeKind, OccurrenceRefId,
        ProductId, SolidId, VertexId,
        copy::{self, CachePolicy, GeomPolicy, MeshPolicy},
        refs_iterator::{CoEdgesOfWire, RefsWireOfFace},
        tool::vertex,
        transform,
    },
    topods::{CompSolid, Compound, Shape},
};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::{
    gp::{Point2, Point3, Transform3, Vector3},
    poly::{Polygon2D, Polygon3D, PolygonOnTriangulation, Triangle, Triangulation},
};
use rustycad_tktopalgo::brep_gprop::BRepGProp;
include!("graph_transform/fixtures.rs");
include!("graph_transform/whole.rs");
include!("graph_transform/nodes.rs");
include!("graph_transform/references.rs");
include!("graph_transform/mesh_fixtures.rs");
include!("graph_transform/mesh.rs");
