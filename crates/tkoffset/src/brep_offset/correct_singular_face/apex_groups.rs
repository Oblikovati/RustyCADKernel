use super::{
    CorrectedEdge, EdgeSurfacePoints, edge_surface_points, make_apex_sphere, rebuild_singular_edge,
    replace_adjacent_vertices, spherical_patch::build_spherical_patch,
};
use crate::brep_offset::BRepOffsetOffsetFace;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp,
    topods::{Edge, Face, Vertex},
};
use rustycad_tkernel::{precision, standard::Failure};
use rustycad_tkg3d::geom::GeomSphericalSurface;
use rustycad_tkmath::gp::Point3;

struct SingularImage {
    apex: Vertex,
    face: Face,
    image: Edge,
    points: EdgeSurfacePoints,
}

pub(super) fn correct_singular_faces(
    offsets: &[&BRepOffsetOffsetFace],
    distance: f64,
) -> Result<Vec<Face>, Failure> {
    let groups = singular_image_groups(offsets)?;
    groups
        .iter()
        .map(|group| correct_apex_group(group, distance))
        .collect()
}

fn singular_image_groups(
    offsets: &[&BRepOffsetOffsetFace],
) -> Result<Vec<Vec<SingularImage>>, Failure> {
    let mut groups = Vec::new();
    for offset in offsets {
        append_offset_images(&mut groups, offset)?;
    }
    Ok(groups)
}

fn append_offset_images(
    groups: &mut Vec<Vec<SingularImage>>,
    offset: &BRepOffsetOffsetFace,
) -> Result<(), Failure> {
    for (source, image) in offset.edge_images() {
        append_singular_image(groups, source, image, offset.face())?;
    }
    Ok(())
}

fn append_singular_image(
    groups: &mut Vec<Vec<SingularImage>>,
    source: &Edge,
    image: &Edge,
    face: &Face,
) -> Result<(), Failure> {
    if !BRepTool::degenerated(source)? || !is_noncollapsed(face, image)? {
        return Ok(());
    }
    let points = edge_surface_points(face, image)?;
    let apex = topexp::vertices(source, false).0;
    insert_by_apex(
        groups,
        SingularImage {
            apex,
            face: face.clone(),
            image: image.clone(),
            points,
        },
    );
    Ok(())
}

fn is_noncollapsed(face: &Face, edge: &Edge) -> Result<bool, Failure> {
    if !BRepTool::degenerated(edge)? {
        return Ok(false);
    }
    let (first, middle, last, _, _) = edge_surface_points(face, edge)?;
    Ok(first.distance(&last) > 1.0e-5 || first.distance(&middle) > 1.0e-5)
}

fn insert_by_apex(groups: &mut Vec<Vec<SingularImage>>, candidate: SingularImage) {
    if let Some(group) = groups.iter_mut().find(|group| {
        group
            .first()
            .is_some_and(|first| first.apex.is_same(&candidate.apex))
    }) {
        group.push(candidate);
        return;
    }
    groups.push(vec![candidate]);
}

fn correct_apex_group(group: &[SingularImage], distance: f64) -> Result<Face, Failure> {
    let first = group.first().ok_or_else(empty_apex_group)?;
    let sphere = build_group_sphere(first, distance)?;
    let corrected = rebuild_group_edges(group, &sphere, first.points.0)?;
    build_spherical_patch(&corrected, &sphere)
}

fn build_group_sphere(
    first: &SingularImage,
    distance: f64,
) -> Result<rustycad_tkernel::standard::Handle<GeomSphericalSurface>, Failure> {
    make_apex_sphere(&first.apex, first.points, distance)
}

fn rebuild_group_edges(
    group: &[SingularImage],
    sphere: &rustycad_tkernel::standard::Handle<GeomSphericalSurface>,
    initial_point: Point3,
) -> Result<Vec<CorrectedEdge>, Failure> {
    let first_vertex = make_group_vertex(initial_point)?;
    let mut previous = first_vertex.clone();
    let mut corrected = Vec::new();
    for candidate in group {
        previous = append_group_edge(candidate, sphere, &first_vertex, previous, &mut corrected)?;
    }
    Ok(corrected)
}

fn append_group_edge(
    candidate: &SingularImage,
    sphere: &rustycad_tkernel::standard::Handle<GeomSphericalSurface>,
    first_vertex: &Vertex,
    previous_vertex: Vertex,
    corrected: &mut Vec<CorrectedEdge>,
) -> Result<Vertex, Failure> {
    let last_point = candidate.points.2;
    let last_vertex = group_end_vertex(first_vertex, last_point)?;
    append_rebuilt_edge(
        candidate,
        sphere,
        previous_vertex,
        last_vertex.clone(),
        corrected,
    )?;
    Ok(last_vertex)
}

fn append_rebuilt_edge(
    candidate: &SingularImage,
    sphere: &rustycad_tkernel::standard::Handle<GeomSphericalSurface>,
    first_vertex: Vertex,
    last_vertex: Vertex,
    corrected: &mut Vec<CorrectedEdge>,
) -> Result<(), Failure> {
    let old_vertices = topexp::vertices(&candidate.image, false);
    let edge = rebuild_singular_edge(
        &candidate.image,
        sphere,
        candidate.points,
        first_vertex,
        last_vertex,
    )?;
    replace_adjacent_vertices(&candidate.face, &candidate.image, &old_vertices, &edge)?;
    corrected.push(edge);
    Ok(())
}

fn group_end_vertex(first: &Vertex, last: Point3) -> Result<Vertex, Failure> {
    let first_point = BRepTool::pnt(first)?;
    if first_point.distance(&last) <= precision::confusion() {
        return Ok(first.clone());
    }
    make_group_vertex(last)
}

fn make_group_vertex(point: Point3) -> Result<Vertex, Failure> {
    let mut vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(&mut vertex, point, precision::confusion())?;
    Ok(vertex)
}

fn empty_apex_group() -> Failure {
    Failure::with_message(
        "Standard_ConstructionError",
        "singular apex group has no edges",
    )
}
