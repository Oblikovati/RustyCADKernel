// SPDX-License-Identifier: GPL-2.0-only
//! Native sweep images: the generated, first and last shapes of the region face, of each
//! curve edge and of each vertex (authored, or the seam vertex of a full circle).
use super::decline::DeclineCause;
use super::operation_result::{HistoryRole, HistorySource};
use super::regions::PlanarRegion;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

pub(crate) type HistoryCheck<T> = Result<T, DeclineCause>;

pub(crate) fn incomplete(reason: String) -> DeclineCause {
    DeclineCause::IncompleteHistory { reason }
}

pub(crate) fn native(context: &str, failure: rustycad_tkernel::standard::Failure) -> DeclineCause {
    incomplete(format!("{context}: {failure}"))
}

/// One native image before indexing.
pub(crate) struct SweepImage {
    pub source: HistorySource,
    pub role: HistoryRole,
    pub shape: Shape,
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
            let count = images.len();
            return Err(incomplete(format!(
                "{source:?} generated {count} shapes, expected one {role:?}"
            )));
        };
        self.push(source, role, image.clone());
        Ok(())
    }

    fn vertex(&mut self, source: HistorySource, vertex: &Shape) -> HistoryCheck<()> {
        self.generated(source, vertex, HistoryRole::LateralEdge)?;
        self.ends(
            source,
            vertex,
            [HistoryRole::StartVertex, HistoryRole::EndVertex],
        )
    }

    fn curve(
        &mut self,
        region: &PlanarRegion,
        source: HistorySource,
        edge: &Edge,
    ) -> HistoryCheck<()> {
        self.generated(source, edge, HistoryRole::SideFace)?;
        self.ends(source, edge, [HistoryRole::StartEdge, HistoryRole::EndEdge])?;
        match seam_vertex(region, edge) {
            Some(seam) => self.vertex(source, &seam),
            None => Ok(()),
        }
    }
}

/// Collect every image of `region`'s face, curves and vertices.
pub(crate) fn collect_images(
    maker: &mut BRepPrimAPIMakePrism,
    region: &PlanarRegion,
) -> HistoryCheck<Vec<SweepImage>> {
    let mut collector = ImageCollector {
        maker,
        images: Vec::new(),
    };
    let caps = [HistoryRole::StartCap, HistoryRole::EndCap];
    collector.ends(HistorySource::Region(region.id()), region.face(), caps)?;
    for (curve, edge) in region.edges() {
        collector.curve(region, HistorySource::Curve(*curve), edge)?;
    }
    for (vertex, shape) in region.vertices() {
        collector.vertex(HistorySource::Vertex(*vertex), shape)?;
    }
    Ok(collector.images)
}

/// The vertex of a closed curve edge that no caller vertex names (a full circle's seam).
fn seam_vertex(region: &PlanarRegion, edge: &Edge) -> Option<Shape> {
    let vertex = topexp::first_vertex(edge, false);
    let authored = region
        .vertices()
        .iter()
        .any(|(_, shape)| shape.is_same(&vertex));
    (!authored).then(|| vertex.into_shape())
}
