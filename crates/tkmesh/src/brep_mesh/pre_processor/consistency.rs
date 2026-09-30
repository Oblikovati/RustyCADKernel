use super::*;
use rustycad_tkmath::poly::Triangulation;
pub(super) fn check_face_mesh(face: &FaceHandle, allow_decrease: bool) -> Result<(), Failure> {
    if face.status().is_set(Status::Outdated) || face.get_face()?.is_null() {
        return Ok(());
    }
    let (mesh, _) = BRepTool::triangulation(&face.get_face()?, mesh_purpose::NONE)?;
    if mesh.is_null() {
        return Ok(());
    }
    let consistent = Deflection::is_consistent(
        requested_deflection(&mesh),
        face.get_deflection(),
        allow_decrease,
        0.1,
    ) && valid_triangle_nodes(&mesh)?;
    if consistent {
        face.status().set_status(Status::Reused);
        face.set_deflection(mesh.deflection());
    } else {
        face.status().set_status(Status::Outdated);
    }
    Ok(())
}
fn requested_deflection(mesh: &Triangulation) -> f64 {
    let parameters = mesh.parameters();
    if !parameters.is_null() && parameters.has_deflection() {
        parameters.deflection()
    } else {
        mesh.deflection()
    }
}
fn valid_triangle_nodes(mesh: &Triangulation) -> Result<bool, Failure> {
    for index in 1..=mesh.nb_triangles() {
        let (a, b, c) = mesh.triangle(index)?.get();
        if [a, b, c]
            .into_iter()
            .any(|node| node < 1 || node > mesh.nb_nodes())
        {
            return Ok(false);
        }
    }
    Ok(true)
}
