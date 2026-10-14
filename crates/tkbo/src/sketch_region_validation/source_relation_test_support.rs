// SPDX-License-Identifier: GPL-2.0-only
//! Named, actual topology fixtures keep diagnostic IDs separate from root conveniences.
use super::source_relations::{SourceMemberRelations, SourceRelation};
use crate::sketch_region::{OriginalMemberIdentity, RegionSourceId};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex;

pub(super) fn actual_vertex() -> Shape {
    let maker = BRepBuilderAPIMakeVertex::new(Point3::new(1.0, 2.0, 3.0)).unwrap();
    maker.vertex().unwrap().into_shape()
}

pub(super) fn member_identity(source: u64, original: &Shape) -> OriginalMemberIdentity {
    OriginalMemberIdentity {
        source: RegionSourceId(source),
        topology_address: original.tshape_address(),
        location: original.location().clone(),
        orientation: original.orientation(),
    }
}

pub(super) fn test_source_relation(output: &Shape, sources: &[u64]) -> SourceRelation {
    SourceRelation {
        output: output.clone(),
        parents: SourceMemberRelations {
            modified: sources
                .iter()
                .map(|id| member_identity(*id, output))
                .collect(),
            ..SourceMemberRelations::default()
        },
    }
}
