// SPDX-License-Identifier: GPL-2.0-only
//! Sweep history: native images indexed into the solid's deterministic traversal order and
//! checked for exact one-to-one coverage of every face, edge and vertex.
use super::operation_result::*;
use super::regions::PlanarRegion;
use super::sweep_images::*;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Solid, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;

/// The indexed topology of a solid and its complete sweep history.
pub(crate) struct IndexedSweep {
    pub faces: Vec<Face>,
    pub edges: Vec<Edge>,
    pub vertices: Vec<Vertex>,
    pub history: OperationHistory,
}

/// Index the solid and project every image; `sweep_history(&mut maker, &solid, region)?`.
pub(crate) fn sweep_history(
    maker: &mut BRepPrimAPIMakePrism,
    solid: &Solid,
    region: &PlanarRegion,
) -> HistoryCheck<IndexedSweep> {
    let images = collect_images(maker, region)?;
    let maps =
        [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex].map(|kind| indexed(solid, kind));
    let [faces, edges, vertices] = maps;
    let maps = [faces?, edges?, vertices?];
    let mut records = images
        .iter()
        .map(|image| record(image, &maps))
        .collect::<HistoryCheck<Vec<_>>>()?;
    records.sort();
    require_one_to_one(&records, [0, 1, 2].map(|kind| maps[kind].extent()))?;
    let history = OperationHistory {
        records,
        deleted: Vec::new(),
    };
    typed_sweep(&maps, history).map_err(|f| native("typed result index", f))
}

/// Index one sub-shape kind of the solid in native traversal order.
fn indexed(solid: &Solid, kind: ShapeEnum) -> HistoryCheck<IndexedMapOfShape> {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(solid, kind, &mut map).map_err(|f| native("result index", f))?;
    Ok(map)
}

fn typed_sweep(
    maps: &[IndexedMapOfShape; 3],
    history: OperationHistory,
) -> Result<IndexedSweep, Failure> {
    Ok(IndexedSweep {
        faces: typed(&maps[0], topods::face)?,
        edges: typed(&maps[1], topods::edge)?,
        vertices: typed(&maps[2], topods::vertex)?,
        history,
    })
}

fn typed<T>(
    map: &IndexedMapOfShape,
    cast: fn(&Shape) -> Result<T, Failure>,
) -> Result<Vec<T>, Failure> {
    (1..=map.extent())
        .map(|index| cast(map.find_key(index)?))
        .collect()
}

fn record(image: &SweepImage, maps: &[IndexedMapOfShape; 3]) -> HistoryCheck<HistoryRecord> {
    let kind = image
        .shape
        .shape_type()
        .map_err(|f| native("image kind", f))?;
    let (slot, entity): (usize, fn(usize) -> ResultEntity) = match kind {
        ShapeEnum::Face => (0, ResultEntity::Face),
        ShapeEnum::Edge => (1, ResultEntity::Edge),
        ShapeEnum::Vertex => (2, ResultEntity::Vertex),
        other => return Err(image_gap(image, &format!("is a {other:?}"))),
    };
    let index = maps[slot].find_index(&image.shape);
    if index == 0 {
        return Err(image_gap(image, "is not in the solid"));
    }
    let (source, role, relation) = (image.source, image.role, HistoryRelation::Generated);
    Ok(HistoryRecord {
        source,
        role,
        relation,
        result: entity(index - 1),
    })
}

fn image_gap(image: &SweepImage, problem: &str) -> super::decline::DeclineCause {
    incomplete(format!(
        "{:?} {:?} image {problem}",
        image.source, image.role
    ))
}

/// Every result entity has exactly one record; a shared or missing image is a gap.
fn require_one_to_one(records: &[HistoryRecord], extents: [usize; 3]) -> HistoryCheck<()> {
    let mut counts = extents.map(|extent| vec![0usize; extent]);
    for record in records {
        let (kind, index) = match record.result {
            ResultEntity::Face(index) => (0, index),
            ResultEntity::Edge(index) => (1, index),
            ResultEntity::Vertex(index) => (2, index),
        };
        counts[kind][index] += 1;
    }
    let gap = counts.iter().enumerate().find_map(|(kind, counts)| {
        let index = counts.iter().position(|count| *count != 1)?;
        Some(format!(
            "result kind {kind} entity {index} has {} parents, expected one",
            counts[index]
        ))
    });
    gap.map_or(Ok(()), |reason| Err(incomplete(reason)))
}
