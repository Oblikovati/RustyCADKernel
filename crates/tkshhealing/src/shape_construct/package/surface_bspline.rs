// Rust translation of ShapeConstruct::ConvertSurfaceToBSpline (ShapeConstruct.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeConstruct::ConvertSurfaceToBSpline`: exact conversion of elementary and extrusion
//! surfaces, approximation (with a decreasing continuity on failure) of the others.
use super::ShapeConstruct;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomElementarySurface, GeomOffsetCurve,
    GeomRectangularTrimmedSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution, GeomSweptSurface,
};
use rustycad_tkgeombase::geom_convert;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Transform3};

/// The native `GeomAbs_Shape` values, indexed by their integer value.
const CONTINUITIES: [Continuity; 7] = [
    Continuity::C0,
    Continuity::G1,
    Continuity::C1,
    Continuity::G2,
    Continuity::C2,
    Continuity::C3,
    Continuity::CN,
];

/// The parametric box `[uf, ul] x [vf, vl]` of the conversion.
#[derive(Clone, Copy)]
pub struct ShapeConstructSurfaceBounds {
    /// First U parameter.
    pub uf: f64,
    /// Last U parameter.
    pub ul: f64,
    /// First V parameter.
    pub vf: f64,
    /// Last V parameter.
    pub vl: f64,
}

/// The approximation settings of `ConvertSurfaceToBSpline`.
#[derive(Clone, Copy)]
pub struct ShapeConstructApproxSettings {
    /// 3D tolerance.
    pub tol3d: f64,
    /// Required continuity.
    pub continuity: Continuity,
    /// Maximal number of segments.
    pub max_segments: i32,
    /// Maximal degree.
    pub max_degree: i32,
}

impl ShapeConstruct {
    /// `ConvertSurfaceToBSpline(surf, UF, UL, VF, VL, Tol3d, Continuity, MaxSegments,
    /// MaxDegree)`: converts the basis of a trimmed surface on the box. An elementary basis
    /// is converted exactly on the box; an extrusion is the converted basis curve swept
    /// between the V bounds; an offset basis curve of a revolution is converted first.
    /// Other surfaces are approximated with continuity min(Continuity, C3), lowered by one
    /// after each failure; an approximation above `tol3d` returns its surface (or a null
    /// handle). Example: `ShapeConstruct::convert_surface_to_bspline(&s, bounds, settings)?`.
    pub fn convert_surface_to_bspline(
        surf: &Handle<dyn GeomSurface>,
        bounds: ShapeConstructSurfaceBounds,
        settings: ShapeConstructApproxSettings,
    ) -> Result<Handle<GeomBSplineSurface>, Failure> {
        let mut basis = surf.clone();
        if let Some(trimmed) = surf.cast::<GeomRectangularTrimmedSurface>().get() {
            basis = trimmed.basis_surface();
        }
        if !basis.cast::<dyn GeomElementarySurface>().is_null() {
            return geom_convert::surface_to_bspline_surface(&trimmed_on(&basis, bounds)?);
        }
        if let Some(extrusion) = basis.cast::<GeomSurfaceOfLinearExtrusion>().get() {
            return extrusion_to_bspline(extrusion, bounds, settings);
        }
        let basis = revolution_with_converted_offset(basis, bounds, settings)?;
        approximate_surface(&trimmed_on(&basis, bounds)?, settings)
    }
}

fn trimmed_on(
    surface: &Handle<dyn GeomSurface>,
    bounds: ShapeConstructSurfaceBounds,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let trimmed = GeomRectangularTrimmedSurface::new(
        surface,
        (bounds.uf, bounds.ul),
        (bounds.vf, bounds.vl),
        (true, true),
    )?;
    Ok(Handle::new(trimmed).cast())
}

fn extrusion_to_bspline(
    extrusion: &GeomSurfaceOfLinearExtrusion,
    bounds: ShapeConstructSurfaceBounds,
    settings: ShapeConstructApproxSettings,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let continuity = settings.continuity.min(Continuity::C2);
    let spline = ShapeConstruct::convert_curve_to_bspline(
        &extrusion.basis_curve(),
        bounds.uf,
        bounds.ul,
        settings.tol3d,
        continuity,
        settings.max_segments,
        settings.max_degree,
    )?;
    let origin = extrusion.value(bounds.uf, 0.0)?;
    let first = shift(origin, extrusion.value(bounds.uf, bounds.vf)?);
    let last = shift(origin, extrusion.value(bounds.uf, bounds.vl)?);
    swept_spline(&spline, (first, last), (bounds.vf, bounds.vl))
}

fn shift(from: Point3, to: Point3) -> Transform3 {
    let mut transform = Transform3::default();
    transform.set_translation_between(from, to);
    transform
}

// Each basis pole gives the row (pole shifted to VF, pole shifted to VL), with its weight
// repeated; the V direction is linear with knots (VF, VL) of multiplicity 2.
fn swept_spline(
    spline: &GeomBSplineCurve,
    shifts: (Transform3, Transform3),
    v_range: (f64, f64),
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let nb_poles = spline.nb_poles();
    let mut poles = Vec::with_capacity(2 * nb_poles as usize);
    let mut weights = Vec::with_capacity(2 * nb_poles as usize);
    for j in 1..=nb_poles {
        let pole = spline.pole(j)?;
        poles.extend([pole.transformed(shifts.0), pole.transformed(shifts.1)]);
        let weight = spline.weight(j)?;
        weights.extend([weight, weight]);
    }
    let poles = Array2::from_shared(&poles, 1, nb_poles, 1, 2)
        .map_err(|e| e.into_failure("ShapeConstruct::ConvertSurfaceToBSpline"))?;
    let weights = Array2::from_shared(&weights, 1, nb_poles, 1, 2)
        .map_err(|e| e.into_failure("ShapeConstruct::ConvertSurfaceToBSpline"))?;
    let (knots, mults) = knot_vectors(spline)?;
    let (u_knots, u_mults) = (
        Array1::from_shared(&knots, 1),
        Array1::from_shared(&mults, 1),
    );
    let v_values = [v_range.0, v_range.1];
    let v_counts = [2, 2];
    let (v_knots, v_mults) = (
        Array1::from_shared(&v_values, 1),
        Array1::from_shared(&v_counts, 1),
    );
    let surface = GeomBSplineSurface::with_weights(
        &poles,
        &weights,
        (&u_knots, &u_mults),
        (&v_knots, &v_mults),
        (spline.degree(), 1),
        (spline.is_periodic(), false),
    )?;
    Ok(Handle::new(surface))
}

fn knot_vectors(spline: &GeomBSplineCurve) -> Result<(Vec<f64>, Vec<i32>), Failure> {
    let count = spline.nb_knots();
    let knots = (1..=count)
        .map(|i| spline.knot(i))
        .collect::<Result<Vec<f64>, _>>()?;
    let mults = (1..=count)
        .map(|i| spline.multiplicity(i))
        .collect::<Result<Vec<i32>, _>>()?;
    Ok((knots, mults))
}

fn revolution_with_converted_offset(
    basis: Handle<dyn GeomSurface>,
    bounds: ShapeConstructSurfaceBounds,
    settings: ShapeConstructApproxSettings,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let revolution = basis.cast::<GeomSurfaceOfRevolution>();
    let Some(revolution) = revolution.get() else {
        return Ok(basis);
    };
    let curve = revolution.basis_curve();
    if curve.cast::<GeomOffsetCurve>().is_null() {
        return Ok(basis);
    }
    let continuity = curve.continuity().min(Continuity::C2);
    let spline = ShapeConstruct::convert_curve_to_bspline(
        &curve,
        bounds.vf,
        bounds.vl,
        settings.tol3d,
        continuity,
        settings.max_segments,
        settings.max_degree,
    )?;
    let spline: Handle<dyn GeomCurve> = spline.cast();
    let surface = GeomSurfaceOfRevolution::new(&spline, revolution.axis())?;
    Ok(Handle::new(surface).cast())
}

// Native retries with the next lower continuity after an exception; an exception at C0
// makes the native loop spin forever (cnt stays 0), which Rust reports as a Failure.
fn approximate_surface(
    surface: &Handle<dyn GeomSurface>,
    settings: ShapeConstructApproxSettings,
) -> Result<Handle<GeomBSplineSurface>, Failure> {
    let mut level = settings.continuity.min(Continuity::C3) as usize;
    loop {
        let continuity = CONTINUITIES[level];
        let attempt = geom_convert::ApproxSurface::from_surface(
            surface,
            settings.tol3d / 2.,
            (continuity, continuity),
            (settings.max_degree, settings.max_degree),
            settings.max_segments,
            0,
        );
        match attempt {
            Ok(approx) => return Ok(accepted_or_error_surface(&approx, settings.tol3d)),
            Err(failure) if level == 0 => return Err(endless_retry(failure)),
            Err(_) => level -= 1,
        }
    }
}

fn accepted_or_error_surface(
    approx: &geom_convert::ApproxSurface,
    tol3d: f64,
) -> Handle<GeomBSplineSurface> {
    if approx.max_error() <= tol3d && approx.is_done() {
        return approx.surface();
    }
    if approx.has_result() {
        return approx.surface();
    }
    Handle::default()
}

fn endless_retry(cause: Failure) -> Failure {
    Failure::with_message(
        "Standard_Failure",
        &format!(
            "ShapeConstruct::ConvertSurfaceToBSpline: GeomConvert_ApproxSurface raised at \
             continuity C0 ({cause}); native retries C0 forever, expected a C0 approximation"
        ),
    )
}
