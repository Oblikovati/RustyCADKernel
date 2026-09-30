// Rust translation of ModelingData/TKG3d/GeomAdaptor/GeomAdaptor.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomAdaptor::MakeCurve` and `GeomAdaptor::MakeSurface`: Geom objects from
//! adapted curves and surfaces.
use crate::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use crate::geom::errors::{domain, null_object};
use crate::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomEllipse, GeomGeometry,
    GeomHyperbola, GeomLine, GeomOffsetSurface, GeomParabola, GeomPlane,
    GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution, GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};

/// A Geom curve equal to the adapted curve (`GeomAdaptor::MakeCurve`).
///
/// Elementary kinds are rebuilt from their gp geometry, Bezier and B-spline curves
/// copied; when the adaptor range differs from the curve range the result is a
/// trimmed curve on the adaptor range. Standard_DomainError for every other kind
/// (offset curves included); `make_curve(&*adaptor)?`.
pub fn make_curve(adaptor: &dyn Adaptor3dCurve) -> Result<Handle<dyn GeomCurve>, Failure> {
    let curve: Handle<dyn GeomCurve> = curve_of_kind(adaptor)?;
    let (first, last) = (adaptor.first_parameter()?, adaptor.last_parameter()?);
    // Natively `(!C.IsNull() && First != C->First) || Last != C->Last`.
    let first_differs: bool = curve.get().is_some_and(|c| first != c.first_parameter());
    if !first_differs {
        let Some(built) = curve.get() else {
            return Err(null_object(
                "GeomAdaptor::MakeCurve: the adaptor gave a null curve",
            ));
        };
        if last == built.last_parameter() {
            return Ok(curve);
        }
    }
    Ok(Handle::new(GeomTrimmedCurve::new(&curve, first, last, true, true)?).cast())
}

// The untrimmed curve of the adaptor's kind.
fn curve_of_kind(adaptor: &dyn Adaptor3dCurve) -> Result<Handle<dyn GeomCurve>, Failure> {
    Ok(match adaptor.get_type()? {
        CurveType::Line => Handle::new(GeomLine::from_lin(adaptor.line()?)).cast(),
        CurveType::Circle => Handle::new(GeomCircle::from_circ(adaptor.circle()?)).cast(),
        CurveType::Ellipse => Handle::new(GeomEllipse::from_elips(adaptor.ellipse()?)).cast(),
        CurveType::Parabola => Handle::new(GeomParabola::from_parab(adaptor.parabola()?)).cast(),
        CurveType::Hyperbola => Handle::new(GeomHyperbola::from_hypr(adaptor.hyperbola()?)).cast(),
        CurveType::BezierCurve => live(&adaptor.bezier()?, "Bezier")?.copy()?.cast(),
        CurveType::BSplineCurve => live(&adaptor.bspline()?, "BSpline")?.copy()?.cast(),
        _ => return Err(domain("GeomAdaptor::MakeCurve : OtherCurve")),
    })
}

/// A Geom surface equal to the adapted surface (`GeomAdaptor::MakeSurface`).
///
/// Elementary kinds are rebuilt from their gp geometry, Bezier and B-spline surfaces
/// copied, swept and offset surfaces rebuilt from their made basis; with
/// `trim_flag`, a result whose bounds differ from the adaptor bounds is trimmed to
/// them. Standard_DomainError for OtherSurface; `make_surface(&*adaptor, true)?`.
pub fn make_surface(
    adaptor: &dyn Adaptor3dSurface,
    trim_flag: bool,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surface: Handle<dyn GeomSurface> = surface_of_kind(adaptor)?;
    let Some(built) = surface.get() else {
        return Ok(surface);
    };
    if !trim_flag {
        return Ok(surface);
    }
    let bounds = (
        adaptor.first_u_parameter()?,
        adaptor.last_u_parameter()?,
        adaptor.first_v_parameter()?,
        adaptor.last_v_parameter()?,
    );
    if bounds == built.bounds() {
        return Ok(surface);
    }
    let (u1, u2, v1, v2) = bounds;
    let trimmed = GeomRectangularTrimmedSurface::new(&surface, (u1, u2), (v1, v2), (true, true))?;
    Ok(Handle::new(trimmed).cast())
}

// The untrimmed surface of the adaptor's kind.
fn surface_of_kind(adaptor: &dyn Adaptor3dSurface) -> Result<Handle<dyn GeomSurface>, Failure> {
    Ok(match adaptor.get_type()? {
        SurfaceType::Plane => Handle::new(GeomPlane::from_pln(adaptor.plane()?)).cast(),
        SurfaceType::Cylinder => {
            Handle::new(GeomCylindricalSurface::from_cylinder(adaptor.cylinder()?)).cast()
        }
        SurfaceType::Cone => Handle::new(GeomConicalSurface::from_cone(adaptor.cone()?)).cast(),
        SurfaceType::Sphere => {
            Handle::new(GeomSphericalSurface::from_sphere(adaptor.sphere()?)).cast()
        }
        SurfaceType::Torus => Handle::new(GeomToroidalSurface::from_torus(adaptor.torus()?)).cast(),
        SurfaceType::BezierSurface => live(&adaptor.bezier()?, "Bezier")?.copy()?.cast(),
        SurfaceType::BSplineSurface => live(&adaptor.bspline()?, "BSpline")?.copy()?.cast(),
        SurfaceType::SurfaceOfRevolution => swept_revolution(adaptor)?,
        SurfaceType::SurfaceOfExtrusion => swept_extrusion(adaptor)?,
        SurfaceType::OffsetSurface => {
            let basis: Handle<dyn Adaptor3dSurface> = adaptor.basis_surface()?;
            let basis_surface: Handle<dyn GeomSurface> =
                make_surface(live(&basis, "BasisSurface")?, true)?;
            let offset = GeomOffsetSurface::new(&basis_surface, adaptor.offset_value()?, false)?;
            Handle::new(offset).cast()
        }
        SurfaceType::OtherSurface => {
            return Err(domain("GeomAdaptor::MakeSurface : OtherSurface"));
        }
    })
}

fn swept_revolution(adaptor: &dyn Adaptor3dSurface) -> Result<Handle<dyn GeomSurface>, Failure> {
    let basis: Handle<dyn Adaptor3dCurve> = adaptor.basis_curve()?;
    let curve: Handle<dyn GeomCurve> = make_curve(live(&basis, "BasisCurve")?)?;
    let surface = GeomSurfaceOfRevolution::new(&curve, adaptor.axe_of_revolution()?)?;
    Ok(Handle::new(surface).cast())
}

fn swept_extrusion(adaptor: &dyn Adaptor3dSurface) -> Result<Handle<dyn GeomSurface>, Failure> {
    let basis: Handle<dyn Adaptor3dCurve> = adaptor.basis_curve()?;
    let curve: Handle<dyn GeomCurve> = make_curve(live(&basis, "BasisCurve")?)?;
    let surface = GeomSurfaceOfLinearExtrusion::new(&curve, adaptor.direction()?)?;
    Ok(Handle::new(surface).cast())
}

// The object behind a handle an adaptor returned; natively a null one is
// dereferenced.
fn live<'a, T: RuntimeTyped + ?Sized>(
    handle: &'a Handle<T>,
    member: &str,
) -> Result<&'a T, Failure> {
    handle.get().ok_or_else(|| {
        null_object(&format!(
            "GeomAdaptor: {member}() returned a null handle; expected an object"
        ))
    })
}
