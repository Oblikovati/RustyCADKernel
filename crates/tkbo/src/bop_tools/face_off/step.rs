use super::*;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Line3;
pub(super) fn minimum_step(
    edge: &Edge,
    face: &Face,
    candidates: &List<CoupleOfShape>,
    point: Point3,
    context: &Context,
) -> Result<(f64, bool), Failure> {
    let mut faces: Vec<Face> = candidates
        .iter()
        .map(|pair| Face::unchecked(pair.shape2().clone()))
        .collect();
    faces.push(face.clone());
    let tolerance = BRepTool::edge_tolerance(edge)?;
    let mut maximum: f64 = -1.;
    let mut minimum: f64 = 5e-6;
    for face in &faces {
        maximum = maximum.max(2. * (tolerance + BRepTool::face_tolerance(face)?));
        let cached = context.surface_adaptor(face)?;
        let adaptor = cached.read();
        let radius = match adaptor.get_type()? {
            SurfaceType::Cylinder => adaptor.cylinder()?.radius(),
            SurfaceType::Cone => Line3::from_axis(adaptor.cone()?.axis()).distance(point),
            SurfaceType::Sphere => {
                minimum = minimum.max(5e-4);
                adaptor.sphere()?.radius()
            }
            SurfaceType::Torus => adaptor.torus()?.major_radius(),
            _ => {
                minimum = minimum.max(5e-4);
                0.
            }
        };
        if radius > 100. {
            let d = 10. * precision::p_confusion();
            minimum = minimum.max((d * d + 2. * d * radius).sqrt());
        }
    }
    maximum = maximum.max(minimum);
    for face in &faces {
        if small_face(face, maximum, context)? {
            return Ok((maximum, true));
        }
    }
    Ok((maximum, false))
}
fn small_face(face: &Face, step: f64, context: &Context) -> Result<bool, Failure> {
    let cached = context.surface_adaptor(face)?;
    let adaptor = cached.read();
    let (umin, umax, vmin, vmax) = context.uv_bounds(face)?;
    if umax > umin && 2. * adaptor.u_resolution(step)? > umax - umin {
        return Ok(true);
    }
    Ok(vmax > vmin && 2. * adaptor.v_resolution(step)? > vmax - vmin)
}
