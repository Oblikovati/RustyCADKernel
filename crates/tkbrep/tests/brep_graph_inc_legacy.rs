//! Original BRepGraphInc_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
mod support;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    brep_graph::{
        layer_topo_supplement::{AttachmentKind, LayerTopoSupplement},
        tool::{edge, face},
        *,
    },
    brep_graph_inc::{ParityOrientation, Storage, populate, reconstruct},
    topexp::Explorer,
    topods::{self, CompSolid, Compound, Edge, Face, Shape, Shell, Solid, Wire},
};
use rustycad_tkernel::{precision, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomPlane, GeomSurface},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::{
    gp::{Direction2, Plane3, Point2, Point3, Transform3, Vector3},
    toploc::Location,
};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::{brep_builder_api::*, brep_gprop::BRepGProp};
include!("graph_inc/fixtures.rs");
include!("graph_inc/primitives.rs");
include!("graph_inc/storage.rs");
include!("graph_inc/relations.rs");
include!("graph_inc/mutations.rs");
include!("graph_inc/cleanup.rs");
include!("graph_inc/supplements.rs");
include!("graph_inc/locations.rs");

include!("graph_inc/rebinding.rs");
