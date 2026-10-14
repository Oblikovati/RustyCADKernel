// SPDX-License-Identifier: GPL-2.0-only
//! Rebuild every row solely through read views; cached binding bytes cannot mask aliased operands.
use super::{rows::*, *};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_prism::surface_charts::CapturedPrismSurface;
use crate::sketch_region_source_conformance::{CapturedPlaneSupport, CapturedSpaceRepresentation};
use rustycad_tkmath::predicates::roots::metric_bounds::MetricCurve;

pub(super) fn capture(body: &OwnedNamedPrismBody) -> Vec<u8> {
    let rows = PrismBodyRows {
        members: body.members().iter().map(member).collect(),
        occurrences: body.occurrences().iter().map(occurrence).collect(),
        faces: body.faces().iter().map(face).collect(),
        endpoints: body.edge_endpoints().iter().map(endpoint).collect(),
        coedges: body.coedges().iter().map(coedge).collect(),
        wires: body.wire_traversals().iter().map(wire).collect(),
    };
    let mut bytes = Vec::new();
    replay::append_rows(&rows, &mut bytes);
    bytes
}
fn member(row: &PrismBodyMember) -> PrismBodyMember {
    PrismBodyMember {
        key: row.key().clone(),
        output_slot: row.output_slot(),
        rounded_world_point: row.rounded_world_point(),
        curve: row.space_curve().map(space),
        tolerance: row.tolerance(),
    }
}
fn occurrence(row: &PrismBodyOccurrence) -> PrismBodyOccurrence {
    PrismBodyOccurrence {
        key: row.key().clone(),
        orientation: row.orientation(),
        placement: row.placement(),
        children: row.children().to_vec(),
        local_vertex: row.local_vertex(),
    }
}
fn face(row: &PrismBodyFace) -> PrismBodyFace {
    PrismBodyFace {
        key: row.key().clone(),
        surface: surface(row.surface()),
        trims: row.trims().iter().map(trim).collect(),
    }
}
fn trim(row: &PrismBodyTrim) -> PrismBodyTrim {
    PrismBodyTrim {
        occurrence: row.occurrence().clone(),
        role: row.role(),
        pcurve: curve(row.pcurve()),
        native_interval: row.native_interval(),
        space: space(row.space_curve()),
    }
}
fn endpoint(row: &PrismBodyEdgeEndpoints) -> PrismBodyEdgeEndpoints {
    PrismBodyEdgeEndpoints {
        edge: row.edge().clone(),
        first: row.first().clone(),
        last: row.last().clone(),
    }
}
fn coedge(row: &PrismBodyCoedge) -> PrismBodyCoedge {
    PrismBodyCoedge {
        occurrence: row.occurrence().clone(),
        face: row.face().clone(),
        wire: row.wire().clone(),
        edge: row.edge().clone(),
        reversed: row.reversed(),
    }
}
fn wire(row: &PrismBodyWireTraversal) -> PrismBodyWireTraversal {
    PrismBodyWireTraversal {
        occurrence: row.occurrence().clone(),
        coedges: row.coedges().to_vec(),
    }
}
fn space(view: PrismBodySpaceCurveView<'_>) -> CapturedSpaceRepresentation {
    CapturedSpaceRepresentation {
        curve: curve(view.curve),
        interval: view.native_interval,
        placement: view.representation_placement,
        same_parameter: view.same_parameter,
    }
}
fn surface(view: PrismBodySurfaceView<'_>) -> CapturedPrismSurface {
    match view {
        PrismBodySurfaceView::Plane {
            frame,
            representation_placement,
        } => CapturedPrismSurface::Plane(CapturedPlaneSupport {
            frame,
            placement: representation_placement,
        }),
        PrismBodySurfaceView::Extrusion {
            basis,
            direction,
            representation_placement,
        } => CapturedPrismSurface::Extrusion {
            basis: curve(basis),
            direction,
            placement: representation_placement,
        },
    }
}
fn curve<const D: usize>(view: MetricCurve<'_, D>) -> CapturedMetricCurve<D> {
    match view {
        MetricCurve::Bezier(value) => CapturedMetricCurve::Bezier {
            poles: value.poles.to_vec(),
            weights: value.weights.map(<[f64]>::to_vec),
        },
        MetricCurve::Line { origin, direction } => CapturedMetricCurve::Line { origin, direction },
    }
}
