// SPDX-License-Identifier: GPL-2.0-only
//! Stored local points are distinct operands from BRepTool's rounded world coordinates.
use super::*;
use rustycad_tkbrep::brep::access::vertex_tshape;
use rustycad_tkg3d::top_abs::ShapeEnum;

pub(super) fn capture(
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
) -> CaptureResult<Vec<JoinedVertexOperand>> {
    let mut rows = Vec::new();
    for (index, context) in bound.contexts.iter().enumerate() {
        if context
            .occurrence
            .shape_type()
            .map_err(PrismFreezeCause::Native)?
            == ShapeEnum::Vertex
        {
            rows.push(capture_vertex(bound, projection, index)?);
        }
    }
    rows.sort_by(|a, b| a.occurrence.cmp(&b.occurrence));
    Ok(rows)
}
fn capture_vertex(
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
    index: usize,
) -> CaptureResult<JoinedVertexOperand> {
    let occurrence = &bound.use_names[index];
    let matches: Vec<_> = projection
        .uses()
        .iter()
        .filter(|row| &row.key == occurrence)
        .collect();
    let current = *only_bound(&matches, "one actual authored vertex occurrence placement")?;
    let actual = &bound.contexts[index].occurrence;
    let placement = super::super::history::freeze_placement(actual.location())?;
    let mut expected = Vec::new();
    let mut observed = Vec::new();
    encoding::placement(placement, &mut expected);
    encoding::placement(current.placement, &mut observed);
    if expected != observed {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "raw vertex owner placement",
        });
    }
    Ok(JoinedVertexOperand {
        vertex: bound.names[index],
        occurrence: occurrence.clone(),
        local_point: local_point(actual)?,
        placement,
    })
}
pub(super) fn local_point(actual: &rustycad_tkbrep::topods::Shape) -> CaptureResult<[f64; 3]> {
    let point = vertex_tshape(actual, "Martensite same-body local vertex operand")
        .map_err(PrismFreezeCause::Native)?
        .pnt();
    let value = [point.x(), point.y(), point.z()];
    for actual in value {
        if !actual.is_finite() {
            return Err(PrismFreezeCause::InvalidScalar {
                quantity: "finite stored local vertex point",
                actual,
            });
        }
    }
    Ok(value)
}
pub(super) fn append(rows: &[JoinedVertexOperand], bytes: &mut Vec<u8>) {
    bytes.extend((rows.len() as u64).to_le_bytes());
    for row in rows {
        replay::text(&format!("{:?}/{:?}", row.vertex, row.occurrence), bytes);
        encoding::numbers(row.local_point, bytes);
        encoding::placement(row.placement, bytes);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rustycad_tkmath::{
        gp::{Point3, Transform3, Vector3},
        toploc::Location,
    };
    #[test]
    fn exact_local_operand_is_not_replaced_by_rounded_world_point() {
        let (_, receipt) = super::super::super::tests::authored_receipt();
        let contexts = super::super::super::super::contexts::body_contexts(
            &receipt.diagnostic.as_ref().unwrap().body,
        )
        .ok()
        .unwrap();
        let actual = &contexts
            .iter()
            .find(|c| c.occurrence.shape_type().unwrap() == ShapeEnum::Vertex)
            .unwrap()
            .occurrence;
        vertex_tshape(actual, "owned rounding trap")
            .unwrap()
            .set_pnt(Point3::new(1., 0., 0.));
        let mut transform = Transform3::default();
        transform.set_translation(Vector3::new(9007199254740992., 0., 0.));
        let placed = actual.located(&Location::new(transform), true).unwrap();
        let world = rustycad_tkbrep::brep::BRepTool::pnt(
            &rustycad_tkbrep::topods::vertex(&placed).unwrap(),
        )
        .unwrap();
        assert_eq!(world.x(), 9007199254740992.);
        assert_eq!(local_point(&placed).unwrap(), [1., 0., 0.]);
        let placement = super::super::super::history::freeze_placement(placed.location()).unwrap();
        assert_eq!(placement.origin, [9007199254740992., 0., 0.]);
    }
}
