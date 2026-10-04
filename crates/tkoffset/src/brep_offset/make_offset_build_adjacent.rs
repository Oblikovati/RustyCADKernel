//! Arc joins for the currently supported pair of adjacent planar faces.
use super::{BRepOffsetAnalyse, BRepOffsetOffset, BRepOffsetOffsetFace};
use rustycad_tkbrep::{
    brep_adaptor::BRepAdaptorSurface,
    topexp::Explorer,
    topods::{Edge, Face, Shape},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg3d::{adaptor3d::Adaptor3dSurface, top_abs::ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;

pub(super) struct AdjacentPlanarJoin {
    pub(super) source_edge: Edge,
    pub(super) face: Face,
}

pub(super) fn arc_patches_for_adjacent_planes(
    faces: &[Face],
    images: &[BRepOffsetOffsetFace],
    analysis: &BRepOffsetAnalyse,
    offset: f64,
) -> Result<Vec<AdjacentPlanarJoin>, Failure> {
    require_supported_pair(faces)?;
    let edge = shared_edge(faces)?;
    require_offset_join(analysis, &edge, offset)?;
    let patch = offset_patch(faces, images, analysis, &edge, offset)?;
    Ok(vec![AdjacentPlanarJoin {
        source_edge: edge,
        face: patch,
    }])
}

fn require_supported_pair(faces: &[Face]) -> Result<(), Failure> {
    if faces.len() != 2 {
        return Err(not_implemented(&format!(
            "different-support join received {} faces; expected exactly two planar faces",
            faces.len()
        )));
    }
    let types = [face_surface_type(&faces[0])?, face_surface_type(&faces[1])?];
    if types
        .iter()
        .any(|kind| *kind != rustycad_tkmath::geom_abs::SurfaceType::Plane)
    {
        return Err(not_implemented(&format!(
            "different-support join received surface types {types:?}; expected [Plane, Plane]",
        )));
    }
    Ok(())
}

fn face_surface_type(face: &Face) -> Result<rustycad_tkmath::geom_abs::SurfaceType, Failure> {
    BRepAdaptorSurface::with_face(face, false)?.get_type()
}

fn shared_edge(faces: &[Face]) -> Result<Edge, Failure> {
    let first = faces[0].clone().into_shape();
    let second = faces[1].clone().into_shape();
    let mut shared = Vec::new();
    for candidate in Explorer::new(&first, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if contains_edge(&second, &candidate)? {
            shared.push(Edge::unchecked(candidate));
        }
    }
    if shared.len() != 1 {
        return Err(not_implemented(&format!(
            "planar face pair has {} shared edges; expected exactly one",
            shared.len()
        )));
    }
    Ok(shared.remove(0))
}

fn contains_edge(face: &Shape, candidate: &Shape) -> Result<bool, Failure> {
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if edge.is_same(candidate) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn require_offset_join(
    analysis: &BRepOffsetAnalyse,
    edge: &Edge,
    offset: f64,
) -> Result<(), Failure> {
    let expected = if offset < 0.0 {
        TypeOfConcavity::Concave
    } else {
        TypeOfConcavity::Convex
    };
    let intervals = analysis.type_of(edge)?;
    if intervals
        .first()
        .map(|interval| interval.get_type())
        .transpose()?
        == Some(expected)
    {
        return Ok(());
    }
    let actual = intervals
        .first()
        .map(|interval| interval.get_type())
        .transpose()?;
    Err(not_implemented(&format!(
        "shared planar edge has offset-side type {actual:?}; expected {expected:?}"
    )))
}

fn offset_patch(
    faces: &[Face],
    images: &[BRepOffsetOffsetFace],
    analysis: &BRepOffsetAnalyse,
    edge: &Edge,
    offset: f64,
) -> Result<Face, Failure> {
    let order = analysis.ancestors(&edge.clone().into_shape())?;
    let rail1 = image_edge(faces, images, order.first(), edge)?;
    let rail2 = image_edge(faces, images, order.last(), edge)?;
    let offset_image =
        BRepOffsetOffset::from_edges(edge, &rail1, &rail2, offset, false, 1.0e-4, Continuity::C1)?;
    Ok(offset_image.face().clone())
}

fn image_edge(
    faces: &[Face],
    images: &[BRepOffsetOffsetFace],
    source: Option<&Shape>,
    edge: &Edge,
) -> Result<Edge, Failure> {
    let source = source.ok_or_else(|| not_implemented("shared edge has no face ancestor"))?;
    let index = faces
        .iter()
        .position(|face| face.clone().into_shape().is_same(source))
        .ok_or_else(|| not_implemented("shared edge ancestor is outside the face pair"))?;
    let image = images[index].generated(&edge.clone().into_shape());
    if image.is_null() {
        return Err(not_implemented("shared edge has no offset image"));
    }
    Ok(Edge::unchecked(image))
}

fn not_implemented(reason: &str) -> Failure {
    Failure::with_type(
        "Standard_NotImplemented",
        reason,
        "BRepOffset_MakeOffset::MakeOffsetShape",
    )
}
