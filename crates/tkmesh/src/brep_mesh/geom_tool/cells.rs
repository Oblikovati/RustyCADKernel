use super::*;
use crate::brep_mesh::DefaultRangeSplitter;
use rustycad_tkmath::bvh::BvhScalar;
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
impl GeomTool<'_> {
    /// Estimate native acceleration-grid counts; `GeomTool::cells_count(&surface,12,0.1,Some(&splitter))?`.
    pub fn cells_count(
        surface: &Handle<dyn Adaptor3dSurface>,
        vertices: i32,
        deflection: f64,
        splitter: Option<&DefaultRangeSplitter>,
    ) -> Result<(i32, i32), Failure> {
        let Some(splitter) = splitter else {
            return Ok((-1, -1));
        };
        let kind = surface.get_type()?;
        let errors = error_factors(surface.get().unwrap(), kind, deflection)?;
        let u = splitter.get_range_u();
        let v = splitter.get_range_v();
        let delta = splitter.get_delta();
        let (u, v) = (u.1 - u.0, v.1 - v.0);
        let mut counts = match kind {
            SurfaceType::Torus => (count(u / delta.0), count(v / delta.1)),
            SurfaceType::Cylinder => (count(u / delta.0 / v), count(v / errors.1)),
            _ => (count(u / delta.0 / errors.0), count(v / delta.1 / errors.1)),
        };
        adjust_counts(surface.get().unwrap(), kind, vertices, &mut counts)?;
        Ok(counts)
    }
}
fn count(estimate: f64) -> i32 {
    2_f64.powf(estimate.log10()).ceil().truncate_to_int()
}
fn error_factors(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    deflection: f64,
) -> Result<(f64, f64), Failure> {
    let mut errors = (deflection * 10.0, deflection * 10.0);
    match kind {
        SurfaceType::Cylinder | SurfaceType::Cone | SurfaceType::Sphere | SurfaceType::Torus => {}
        SurfaceType::SurfaceOfExtrusion | SurfaceType::SurfaceOfRevolution => {
            let curve = surface.basis_curve()?;
            if curve.get_type()? == CurveType::BSplineCurve && curve.degree()? > 2 {
                errors.1 /= f64::from(curve.degree()?.wrapping_mul(curve.nb_knots()?));
            }
        }
        SurfaceType::BezierSurface | SurfaceType::BSplineSurface => {
            surface_errors(surface, kind, &mut errors)?
        }
        _ => errors = (1.0, 1.0),
    }
    Ok(errors)
}
fn surface_errors(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    errors: &mut (f64, f64),
) -> Result<(), Failure> {
    if surface.u_degree()? > 2 {
        errors.0 /= f64::from(surface.u_degree()?.wrapping_mul(
            if kind == SurfaceType::BSplineSurface {
                surface.nb_u_knots()?
            } else {
                1
            },
        ));
    }
    if surface.v_degree()? > 2 {
        errors.1 /= f64::from(surface.v_degree()?.wrapping_mul(
            if kind == SurfaceType::BSplineSurface {
                surface.nb_v_knots()?
            } else {
                1
            },
        ));
    }
    Ok(())
}
fn adjust_counts(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    vertices: i32,
    counts: &mut (i32, i32),
) -> Result<(), Failure> {
    if kind == SurfaceType::OtherSurface {
        *counts = (-1, -1);
        return Ok(());
    }
    let fallback = count(f64::from(vertices));
    adjust_surface_counts(surface, kind, fallback, counts)?;
    counts.0 = counts.0.max(2);
    counts.1 = counts.1.max(2);
    Ok(())
}
fn adjust_surface_counts(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    fallback: i32,
    counts: &mut (i32, i32),
) -> Result<(), Failure> {
    match kind {
        SurfaceType::Plane => *counts = (fallback, fallback),
        SurfaceType::Cylinder | SurfaceType::Cone => counts.1 = fallback,
        SurfaceType::SurfaceOfExtrusion | SurfaceType::SurfaceOfRevolution => {
            adjust_swept(surface, kind, fallback, counts)?
        }
        SurfaceType::BezierSurface | SurfaceType::BSplineSurface => {
            if surface.u_degree()? < 2 {
                counts.0 = fallback;
            }
            if surface.v_degree()? < 2 {
                counts.1 = fallback;
            }
        }
        _ => {}
    }
    Ok(())
}
fn adjust_swept(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
    fallback: i32,
    counts: &mut (i32, i32),
) -> Result<(), Failure> {
    let curve = surface.basis_curve()?;
    if curve.get_type()? == CurveType::Line
        || (curve.get_type()? == CurveType::BSplineCurve && curve.degree()? < 2)
    {
        if kind == SurfaceType::SurfaceOfExtrusion {
            counts.0 = fallback;
        } else {
            counts.1 = fallback;
        }
    }
    if kind == SurfaceType::SurfaceOfExtrusion {
        counts.1 = fallback;
    }
    Ok(())
}
