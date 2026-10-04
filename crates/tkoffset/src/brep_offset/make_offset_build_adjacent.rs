//! Arc joins for different-support planar and cylinder face boundaries.
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

pub(super) struct SharedSupportJoin {
    pub(super) source_edge: Edge,
    pub(super) face: Face,
}

pub(super) fn arc_patches_for_supported_faces(
    faces: &[Face],
    images: &[BRepOffsetOffsetFace],
    analysis: &BRepOffsetAnalyse,
    offset: f64,
) -> Result<Vec<SharedSupportJoin>, Failure> {
    let mut joins = Vec::new();
    for face in faces {
        for candidate in face_edges(face)? {
            append_shared_edge_join(faces, images, analysis, offset, candidate, &mut joins)?;
        }
    }
    require_nonempty_joins(&joins)?;
    Ok(joins)
}

fn append_shared_edge_join(
    faces: &[Face],
    images: &[BRepOffsetOffsetFace],
    analysis: &BRepOffsetAnalyse,
    offset: f64,
    edge: Edge,
    joins: &mut Vec<SharedSupportJoin>,
) -> Result<(), Failure> {
    if joins.iter().any(|join| join.source_edge.is_same(&edge)) {
        return Ok(());
    }
    let ancestors = analysis.ancestors(&edge.clone().into_shape())?;
    if ancestors.len() == 1 {
        return Ok(());
    }
    if ancestors.len() != 2 {
        return Err(not_implemented(&format!(
            "offset edge has {} face ancestors; expected at most two",
            ancestors.len()
        )));
    }
    require_supported_ancestors(faces, ancestors)?;
    require_offset_join(analysis, &edge, offset)?;
    let patch = offset_patch(faces, images, analysis, &edge, offset)?;
    joins.push(SharedSupportJoin {
        source_edge: edge,
        face: patch,
    });
    Ok(())
}

fn require_supported_ancestors(faces: &[Face], ancestors: &[Shape]) -> Result<(), Failure> {
    let pair = ancestor_faces(faces, ancestors)?;
    let types = [face_surface_type(pair[0])?, face_surface_type(pair[1])?];
    if !supported_analytic_pair(types) {
        return Err(not_implemented(&format!(
            "different-support join received surface types {types:?}; expected [Plane, Plane] or a Plane/Cylinder pair",
        )));
    }
    Ok(())
}

fn ancestor_faces<'a>(faces: &'a [Face], ancestors: &[Shape]) -> Result<[&'a Face; 2], Failure> {
    let first = source_face(faces, &ancestors[0])?;
    let second = source_face(faces, &ancestors[1])?;
    if std::ptr::eq(first, second) {
        return Err(not_implemented(
            "shared edge has the same face ancestor twice",
        ));
    }
    Ok([first, second])
}

fn source_face<'a>(faces: &'a [Face], source: &Shape) -> Result<&'a Face, Failure> {
    faces
        .iter()
        .find(|face| (*face).clone().into_shape().is_same(source))
        .ok_or_else(|| not_implemented("shared edge ancestor is outside the offset face set"))
}

fn face_edges(face: &Face) -> Result<Vec<Edge>, Failure> {
    Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .map(|edges| edges.map(Edge::unchecked).collect())
}

fn require_nonempty_joins(joins: &[SharedSupportJoin]) -> Result<(), Failure> {
    if joins.is_empty() {
        return Err(not_implemented(
            "different-support faces have no shared edge to join",
        ));
    }
    Ok(())
}

fn supported_analytic_pair(types: [rustycad_tkmath::geom_abs::SurfaceType; 2]) -> bool {
    use rustycad_tkmath::geom_abs::SurfaceType::{Cylinder, Plane};
    matches!(
        types,
        [Plane, Plane] | [Plane, Cylinder] | [Cylinder, Plane]
    )
}

fn face_surface_type(face: &Face) -> Result<rustycad_tkmath::geom_abs::SurfaceType, Failure> {
    BRepAdaptorSurface::with_face(face, false)?.get_type()
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
        "shared analytic edge has offset-side type {actual:?}; expected {expected:?}"
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
