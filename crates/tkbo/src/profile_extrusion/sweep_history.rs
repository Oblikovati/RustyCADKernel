// SPDX-License-Identifier: GPL-2.0-only
//! Sweep history: native generated/first/last images of the region, its curves and its
//! vertices, indexed into the solid and checked for exact one-to-one coverage.
use super::decline::DeclineCause;
use super::operation_result::*;
use super::regions::PlanarRegion;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Shape, Solid};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

type HistoryCheck<T> = Result<T, DeclineCause>;

fn incomplete(reason: String) -> DeclineCause {
    DeclineCause::IncompleteHistory { reason }
}

fn native(context: &str, failure: rustycad_tkernel::standard::Failure) -> DeclineCause {
    incomplete(format!("{context}: {failure}"))
}

/// One native image before indexing.
struct SweepImage {
    source: HistorySource,
    role: HistoryRole,
    shape: Shape,
}

/// Images of every source sub-shape; generated lists must hold exactly one shape.
struct ImageCollector<'a> {
    maker: &'a mut BRepPrimAPIMakePrism,
    images: Vec<SweepImage>,
}

impl ImageCollector<'_> {
    fn push(&mut self, source: HistorySource, role: HistoryRole, shape: Shape) {
        self.images.push(SweepImage {
            source,
            role,
            shape,
        });
    }

    fn ends(
        &mut self,
        source: HistorySource,
        shape: &Shape,
        roles: [HistoryRole; 2],
    ) -> HistoryCheck<()> {
        let first = self
            .maker
            .first_shape_of(shape)
            .map_err(|f| native("first image", f))?;
        let last = self
            .maker
            .last_shape_of(shape)
            .map_err(|f| native("last image", f))?;
        self.push(source, roles[0], first);
        self.push(source, roles[1], last);
        Ok(())
    }

    fn generated(
        &mut self,
        source: HistorySource,
        shape: &Shape,
        role: HistoryRole,
    ) -> HistoryCheck<()> {
        let generated = self
            .maker
            .generated(shape)
            .map_err(|f| native("generated image", f))?;
        let images: Vec<Shape> = generated.iter().cloned().collect();
        let [image] = images.as_slice() else {
            return Err(incomplete(format!(
                "{source:?} generated {} shapes, expected one {role:?}",
                images.len()
            )));
        };
        let image = image.clone();
        self.push(source, role, image);
        Ok(())
    }
}

/// Collect every image of `region`'s face, curves and vertices.
fn collect(
    maker: &mut BRepPrimAPIMakePrism,
    region: &PlanarRegion,
) -> HistoryCheck<Vec<SweepImage>> {
    let mut collector = ImageCollector {
        maker,
        images: Vec::new(),
    };
    let face = HistorySource::Region(region.id());
    collector.ends(
        face,
        region.face(),
        [HistoryRole::StartCap, HistoryRole::EndCap],
    )?;
    for (curve, edge) in region.edges() {
        let source = HistorySource::Curve(*curve);
        collector.generated(source, edge, HistoryRole::SideFace)?;
        collector.ends(source, edge, [HistoryRole::StartEdge, HistoryRole::EndEdge])?;
        if let Some(seam) = seam_vertex(region, edge) {
            vertex_images(&mut collector, source, &seam)?;
        }
    }
    for (vertex, shape) in region.vertices() {
        vertex_images(&mut collector, HistorySource::Vertex(*vertex), shape)?;
    }
    Ok(collector.images)
}

fn vertex_images(
    collector: &mut ImageCollector<'_>,
    source: HistorySource,
    vertex: &Shape,
) -> HistoryCheck<()> {
    collector.generated(source, vertex, HistoryRole::LateralEdge)?;
    collector.ends(
        source,
        vertex,
        [HistoryRole::StartVertex, HistoryRole::EndVertex],
    )
}

/// The vertex of a closed curve edge that no caller vertex names (a full circle's seam).
fn seam_vertex(region: &PlanarRegion, edge: &rustycad_tkbrep::topods::Edge) -> Option<Shape> {
    let vertex = topexp::first_vertex(edge, false);
    let authored = region
        .vertices()
        .iter()
        .any(|(_, shape)| shape.is_same(&vertex));
    (!authored).then(|| vertex.into_shape())
}

/// Index one sub-shape kind of the solid in native traversal order.
fn indexed(solid: &Solid, kind: ShapeEnum) -> HistoryCheck<IndexedMapOfShape> {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(solid, kind, &mut map).map_err(|f| native("result index", f))?;
    Ok(map)
}

/// The indexed topology of a solid and its complete sweep history.
pub(crate) struct IndexedSweep {
    pub faces: IndexedMapOfShape,
    pub edges: IndexedMapOfShape,
    pub vertices: IndexedMapOfShape,
    pub history: OperationHistory,
}

/// Index the solid and project every image; `sweep_history(&mut maker, &solid, region)?`.
pub(crate) fn sweep_history(
    maker: &mut BRepPrimAPIMakePrism,
    solid: &Solid,
    region: &PlanarRegion,
) -> HistoryCheck<IndexedSweep> {
    let images = collect(maker, region)?;
    let faces = indexed(solid, ShapeEnum::Face)?;
    let edges = indexed(solid, ShapeEnum::Edge)?;
    let vertices = indexed(solid, ShapeEnum::Vertex)?;
    let maps = [&faces, &edges, &vertices];
    let mut records = images
        .iter()
        .map(|image| record(image, maps))
        .collect::<HistoryCheck<Vec<_>>>()?;
    records.sort();
    require_one_to_one(&records, maps.map(IndexedMapOfShape::extent))?;
    let history = OperationHistory {
        records,
        deleted: Vec::new(),
    };
    Ok(IndexedSweep {
        faces,
        edges,
        vertices,
        history,
    })
}

fn record(
    image: &SweepImage,
    [faces, edges, vertices]: [&IndexedMapOfShape; 3],
) -> HistoryCheck<HistoryRecord> {
    let kind = image
        .shape
        .shape_type()
        .map_err(|f| native("image kind", f))?;
    let (map, entity): (_, fn(usize) -> ResultEntity) = match kind {
        ShapeEnum::Face => (faces, ResultEntity::Face),
        ShapeEnum::Edge => (edges, ResultEntity::Edge),
        ShapeEnum::Vertex => (vertices, ResultEntity::Vertex),
        other => {
            return Err(incomplete(format!(
                "{:?} {:?} image is a {other:?}",
                image.source, image.role
            )));
        }
    };
    let index = map.find_index(&image.shape);
    if index == 0 {
        return Err(incomplete(format!(
            "{:?} {:?} image is not in the solid",
            image.source, image.role
        )));
    }
    let (source, role, relation) = (image.source, image.role, HistoryRelation::Generated);
    Ok(HistoryRecord {
        source,
        role,
        relation,
        result: entity(index - 1),
    })
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
    for (kind, counts) in counts.iter().enumerate() {
        if let Some(index) = counts.iter().position(|count| *count != 1) {
            let count = counts[index];
            return Err(incomplete(format!(
                "result kind {kind} entity {index} has {count} parents, expected one"
            )));
        }
    }
    Ok(())
}
