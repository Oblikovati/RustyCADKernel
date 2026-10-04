//! `BRepOffset_MakeOffset` image links and history queries for supported builds.
use super::{BRepOffsetMakeOffset, BRepOffsetOffsetFace};
use rustycad_tkbool::brep_algo::BRepAlgoImage;
use rustycad_tkbrep::{
    topexp::Explorer,
    topods::{Face, Shape},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

impl BRepOffsetMakeOffset {
    /// Returns face-to-offset-face links; `maker.offset_faces_from_shapes().image(&face)`.
    pub fn offset_faces_from_shapes(&self) -> &BRepAlgoImage {
        &self.offset_faces
    }

    /// Returns edge-to-offset-shape links; `maker.offset_edges_from_shapes().has_image(&edge)`.
    /// Generated vertices are queried through `maker.generated(&vertex)?` instead.
    pub fn offset_edges_from_shapes(&self) -> &BRepAlgoImage {
        &self.offset_edges
    }

    /// Returns final images that remain in the result; `maker.generated(&face)?`.
    /// This owned `Vec` is a snapshot; native `Generated()` returns a reusable internal list
    /// that later `Generated()` or `Modified()` calls replace.
    pub fn generated(&mut self, source: &Shape) -> Result<Vec<Shape>, Failure> {
        let images = self.images_for(source)?;
        filter_result_images(&self.result_shape, images)
    }

    /// Returns no modified faces for the supported open single-face skin path; `maker.modified(&face)`.
    /// This owned `Vec` is a snapshot; native `Modified()` returns a reusable internal list.
    pub fn modified(&self, _source: &Shape) -> Vec<Shape> {
        Vec::new()
    }

    /// Checks whether no original or replacement image remains in the result.
    /// `maker.is_deleted(&source)?`.
    pub fn is_deleted(&mut self, source: &Shape) -> Result<bool, Failure> {
        if result_contains(&self.result_shape, source)? {
            return Ok(false);
        }
        Ok(self.generated(source)?.is_empty() && self.modified(source).is_empty())
    }

    pub(super) fn record_offset_history(
        &mut self,
        source: &Face,
        image: &BRepOffsetOffsetFace,
    ) -> Result<(), Failure> {
        let source_shape = source.clone().into_shape();
        let image_shape = image.face().clone().into_shape();
        self.offset_faces.set_root(&source_shape);
        self.offset_faces.bind(&source_shape, &image_shape)?;
        record_subshape_history(&mut self.offset_edges, source, image, ShapeEnum::Edge)?;
        record_generated_vertices(&mut self.generated_vertices, source, image)
    }

    pub(super) fn record_join_generated_face(&mut self, source: &Shape, image: &Shape) {
        self.offset_edges.remove_root(source);
        self.generated_edge_faces
            .push((source.clone(), image.clone()));
    }

    fn images_for(&self, source: &Shape) -> Result<Vec<Shape>, Failure> {
        match source.shape_type()? {
            ShapeEnum::Face => Ok(self.offset_faces.last_image(source)),
            ShapeEnum::Edge => Ok(self.edge_images(source)),
            ShapeEnum::Vertex => Ok(vertex_image(&self.generated_vertices, source)
                .into_iter()
                .collect()),
            _ => Ok(Vec::new()),
        }
    }

    fn edge_images(&self, source: &Shape) -> Vec<Shape> {
        if self.offset_edges.has_image(source) {
            return self.offset_edges.last_image(source);
        }
        self.generated_edge_faces
            .iter()
            .find(|(edge, _)| edge.is_same(source))
            .map(|(_, image)| vec![image.clone()])
            .unwrap_or_default()
    }
}

fn record_generated_vertices(
    history: &mut Vec<(Shape, Shape)>,
    source: &Face,
    offset: &BRepOffsetOffsetFace,
) -> Result<(), Failure> {
    for vertex in Explorer::new(
        &source.clone().into_shape(),
        ShapeEnum::Vertex,
        ShapeEnum::Shape,
    )? {
        if history.iter().any(|(old, _)| old.is_same(&vertex)) {
            continue;
        }
        let image = offset.generated(&vertex);
        if !image.is_null() {
            history.push((vertex, image));
        }
    }
    Ok(())
}

fn vertex_image(history: &[(Shape, Shape)], source: &Shape) -> Option<Shape> {
    history
        .iter()
        .find(|(old, _)| old.is_same(source))
        .map(|(_, image)| image.clone())
}

fn record_subshape_history(
    history: &mut BRepAlgoImage,
    source: &Face,
    offset: &BRepOffsetOffsetFace,
    kind: ShapeEnum,
) -> Result<(), Failure> {
    let mut shapes = Explorer::new(&source.clone().into_shape(), kind, ShapeEnum::Shape)?;
    let mut seen = Vec::new();
    while shapes.more() {
        let old = shapes.current().clone();
        if !seen.iter().any(|shape: &Shape| shape.is_same(&old)) {
            seen.push(old.clone());
            bind_generated_shape(history, offset, &old)?;
        }
        shapes.next()?;
    }
    Ok(())
}

fn bind_generated_shape(
    history: &mut BRepAlgoImage,
    offset: &BRepOffsetOffsetFace,
    source: &Shape,
) -> Result<(), Failure> {
    if history.has_image(source) {
        return Ok(());
    }
    let image = offset.generated(source);
    if image.is_null() {
        return Ok(());
    }
    history.set_root(source);
    history.bind(source, &image)
}

fn filter_result_images(result: &Shape, images: Vec<Shape>) -> Result<Vec<Shape>, Failure> {
    let mut kept = Vec::new();
    for image in images {
        if result_contains(result, &image)? {
            kept.push(image);
        }
    }
    Ok(kept)
}

fn result_contains(result: &Shape, candidate: &Shape) -> Result<bool, Failure> {
    if result.is_null() || candidate.is_null() {
        return Ok(false);
    }
    if result.is_same(candidate) {
        return Ok(true);
    }
    contains_subshape(result, candidate, candidate.shape_type()?)
}

fn contains_subshape(result: &Shape, candidate: &Shape, kind: ShapeEnum) -> Result<bool, Failure> {
    for shape in Explorer::new(result, kind, ShapeEnum::Shape)? {
        if shape.is_same(candidate) {
            return Ok(true);
        }
    }
    Ok(false)
}
