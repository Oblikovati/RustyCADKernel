//! Triangulation initialization transforms, shrunken faces and free-edge order.
use super::sensitive_presentation::{SensitivePresentationGeometry, sensitive_polyline};
use crate::{
    prs3d::Prs3d,
    select3d::{Select3DSensitiveEntity, Select3DSensitiveTriangulation},
};
use rustycad_tkernel::{ncollection::Sequence, standard::Failure};
use rustycad_tkmath::gp::{Point3, Transform3, Xyz};
pub(super) fn shrunk_sensitive_triangle(points: [Point3; 3], center: Xyz) -> [Point3; 4] {
    let [a, b, c] = points.map(|point| Point3::from(center + (point.xyz() - center) * 0.9));
    [a, b, c, a]
}
pub(super) fn append_sensitive_triangulation(
    geometry: &mut SensitivePresentationGeometry,
    sensitive: &Select3DSensitiveTriangulation,
    location: Transform3,
) -> Result<(), Failure> {
    let location = if sensitive.has_init_location() {
        location.multiplied(sensitive.get_init_location().transformation())
    } else {
        location
    };
    let mesh = sensitive.triangulation();
    for index in 1..=mesh.nb_triangles() {
        let (a, b, c) = mesh.triangle(index)?.get();
        let points = [mesh.node(a)?, mesh.node(b)?, mesh.node(c)?].map(|p| p.transformed(location));
        let center = (points[0].xyz() + points[1].xyz() + points[2].xyz()) / 3.;
        geometry.append_line(shrunk_sensitive_triangle(points, center));
    }
    let mut free = Sequence::default();
    Prs3d::add_free_edges(&mut free, &mesh, &location)?;
    if !free.is_empty() {
        geometry
            .free_edges
            .append(sensitive_polyline(free.iter().copied()));
    }
    Ok(())
}
