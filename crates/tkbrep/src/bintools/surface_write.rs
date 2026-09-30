// Rust translation of BinTools_SurfaceSet::WriteSurface and its static `operator<<`
// overloads (BinTools_SurfaceSet.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Writing one `Geom_Surface` record: a type code byte (1 plane to 11 offset surface)
//! and the fields of that class.
use super::curve_write::write_curve;
use super::errors::{null_geometry, wrap_failure};
use super::ostream::BinOStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d_hash::class_dispatch::exact_class;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface, GeomToroidalSurface,
};
use rustycad_tkmath::gp::{Axis1, Point3};

/// The `#define` type codes of BinTools_SurfaceSet.cxx.
pub(crate) mod code {
    pub(crate) const PLANE: u8 = 1;
    pub(crate) const CYLINDER: u8 = 2;
    pub(crate) const CONE: u8 = 3;
    pub(crate) const SPHERE: u8 = 4;
    pub(crate) const TORUS: u8 = 5;
    pub(crate) const LINEAR_EXTRUSION: u8 = 6;
    pub(crate) const REVOLUTION: u8 = 7;
    pub(crate) const BEZIER: u8 = 8;
    pub(crate) const BSPLINE: u8 = 9;
    pub(crate) const RECTANGULAR: u8 = 10;
    pub(crate) const OFFSET: u8 = 11;
}

/// `BinTools_SurfaceSet::WriteSurface`: dispatches on the exact class; another class
/// raises "UNKNOWN SURFACE TYPE". Every failure is wrapped in "EXCEPTION in
/// BinTools_SurfaceSet::WriteSurface(..)"; a null surface raises Standard_NullObject
/// unwrapped (a crash natively).
pub(crate) fn write_surface(
    surface: &Handle<dyn GeomSurface>,
    out: &mut BinOStream<'_>,
) -> Result<(), Failure> {
    if surface.is_null() {
        return Err(null_geometry(
            "BinTools_SurfaceSet::WriteSurface",
            "Geom_Surface",
        ));
    }
    wrap_failure("EXCEPTION in BinTools_SurfaceSet::WriteSurface(..)", || {
        if write_elementary(surface, out)?
            || write_swept(surface, out)?
            || write_derived(surface, out)?
        {
            return Ok(());
        }
        Err(Failure::new("UNKNOWN SURFACE TYPE"))
    })
}

/// The position of an elementary surface: the location, then the main, X and Y
/// directions.
fn write_frame(out: &mut BinOStream<'_>, code: u8, location: Point3, axes: [Axis1; 3]) {
    out.write_byte(code);
    out.write_point3(location);
    for axis in axes {
        out.write_direction3(axis.direction());
    }
}

fn write_elementary(
    surface: &Handle<dyn GeomSurface>,
    out: &mut BinOStream<'_>,
) -> Result<bool, Failure> {
    if let Some(plane) = exact_class::<GeomPlane, _>(surface) {
        let pln = plane.pln();
        write_frame(
            out,
            code::PLANE,
            pln.location(),
            [pln.axis(), pln.x_axis(), pln.y_axis()],
        );
    } else if let Some(cylinder) = exact_class::<GeomCylindricalSurface, _>(surface) {
        let cyl = cylinder.cylinder()?;
        write_frame(
            out,
            code::CYLINDER,
            cyl.location(),
            [cyl.axis(), cyl.x_axis(), cyl.y_axis()],
        );
        out.write_real(cyl.radius());
    } else if let Some(cone) = exact_class::<GeomConicalSurface, _>(surface) {
        let cone = cone.cone()?;
        write_frame(
            out,
            code::CONE,
            cone.location(),
            [cone.axis(), cone.x_axis(), cone.y_axis()],
        );
        out.write_real(cone.ref_radius());
        out.write_real(cone.semi_angle());
    } else {
        return write_round(surface, out);
    }
    Ok(true)
}

fn write_round(
    surface: &Handle<dyn GeomSurface>,
    out: &mut BinOStream<'_>,
) -> Result<bool, Failure> {
    if let Some(sphere) = exact_class::<GeomSphericalSurface, _>(surface) {
        let sphere = sphere.sphere()?;
        let axes = [sphere.position().axis(), sphere.x_axis(), sphere.y_axis()];
        write_frame(out, code::SPHERE, sphere.location(), axes);
        out.write_real(sphere.radius());
    } else if let Some(torus) = exact_class::<GeomToroidalSurface, _>(surface) {
        let torus = torus.torus()?;
        let axes = [torus.axis(), torus.x_axis(), torus.y_axis()];
        write_frame(out, code::TORUS, torus.location(), axes);
        out.write_real(torus.major_radius());
        out.write_real(torus.minor_radius());
    } else {
        return Ok(false);
    }
    Ok(true)
}

/// The swept surfaces: their direction (and axis location), then the basis curve.
fn write_swept(
    surface: &Handle<dyn GeomSurface>,
    out: &mut BinOStream<'_>,
) -> Result<bool, Failure> {
    let basis = if let Some(extrusion) = exact_class::<GeomSurfaceOfLinearExtrusion, _>(surface) {
        out.write_byte(code::LINEAR_EXTRUSION);
        out.write_direction3(extrusion.direction());
        extrusion.basis_curve()
    } else if let Some(revolution) = exact_class::<GeomSurfaceOfRevolution, _>(surface) {
        out.write_byte(code::REVOLUTION);
        out.write_point3(revolution.location());
        out.write_direction3(revolution.direction());
        revolution.basis_curve()
    } else {
        return Ok(false);
    };
    write_curve(&basis, out)?;
    Ok(true)
}

fn write_derived(
    surface: &Handle<dyn GeomSurface>,
    out: &mut BinOStream<'_>,
) -> Result<bool, Failure> {
    if let Some(bezier) = exact_class::<GeomBezierSurface, _>(surface) {
        write_bezier(&bezier, out)?;
    } else if let Some(bspline) = exact_class::<GeomBSplineSurface, _>(surface) {
        write_bspline(&bspline, out)?;
    } else if let Some(trimmed) = exact_class::<GeomRectangularTrimmedSurface, _>(surface) {
        out.write_byte(code::RECTANGULAR);
        let (u1, u2, v1, v2) = trimmed.bounds();
        for value in [u1, u2, v1, v2] {
            out.write_real(value);
        }
        write_surface(&trimmed.basis_surface(), out)?;
    } else if let Some(offset) = exact_class::<GeomOffsetSurface, _>(surface) {
        out.write_byte(code::OFFSET);
        out.write_real(offset.offset());
        write_surface(&offset.basis_surface(), out)?;
    } else {
        return Ok(false);
    }
    Ok(true)
}

/// The rational flags, the `char16_t` degrees, then the pole grid row by row, each
/// pole followed by its weight when either direction is rational.
fn write_bezier(
    bezier: &Handle<GeomBezierSurface>,
    out: &mut BinOStream<'_>,
) -> Result<(), Failure> {
    out.write_byte(code::BEZIER);
    let rational = (bezier.is_u_rational(), bezier.is_v_rational());
    out.write_bool(rational.0);
    out.write_bool(rational.1);
    let degrees = (bezier.u_degree(), bezier.v_degree());
    out.write_ext_char(degrees.0 as u16);
    out.write_ext_char(degrees.1 as u16);
    let pole = |row: i32, col: i32| bezier.pole(row, col);
    let weight = |row: i32, col: i32| bezier.weight(row, col);
    let grid = PoleGrid {
        rows: degrees.0 + 1,
        cols: degrees.1 + 1,
        weighted: rational.0 || rational.1,
    };
    grid.write(out, &pole, &weight)
}

/// The rational and periodic flags, the degrees, the pole and knot counts, the pole
/// grid, then the U and V knots with their multiplicities.
fn write_bspline(
    bspline: &Handle<GeomBSplineSurface>,
    out: &mut BinOStream<'_>,
) -> Result<(), Failure> {
    out.write_byte(code::BSPLINE);
    let rational = (bspline.is_u_rational(), bspline.is_v_rational());
    for flag in [
        rational.0,
        rational.1,
        bspline.is_u_periodic(),
        bspline.is_v_periodic(),
    ] {
        out.write_bool(flag);
    }
    out.write_ext_char(bspline.u_degree() as u16);
    out.write_ext_char(bspline.v_degree() as u16);
    let poles = (bspline.nb_u_poles(), bspline.nb_v_poles());
    let knots = (bspline.nb_u_knots(), bspline.nb_v_knots());
    for count in [poles.0, poles.1, knots.0, knots.1] {
        out.write_integer(count);
    }
    let pole = |row: i32, col: i32| bspline.pole(row, col);
    let weight = |row: i32, col: i32| bspline.weight(row, col);
    let grid = PoleGrid {
        rows: poles.0,
        cols: poles.1,
        weighted: rational.0 || rational.1,
    };
    grid.write(out, &pole, &weight)?;
    for index in 1..=knots.0 {
        out.write_real(bspline.u_knot(index)?);
        out.write_integer(bspline.u_multiplicity(index)?);
    }
    for index in 1..=knots.1 {
        out.write_real(bspline.v_knot(index)?);
        out.write_integer(bspline.v_multiplicity(index)?);
    }
    Ok(())
}

/// The pole loops both spline writers share.
struct PoleGrid {
    rows: i32,
    cols: i32,
    weighted: bool,
}

impl PoleGrid {
    fn write(
        &self,
        out: &mut BinOStream<'_>,
        pole: &dyn Fn(i32, i32) -> Result<Point3, Failure>,
        weight: &dyn Fn(i32, i32) -> Result<f64, Failure>,
    ) -> Result<(), Failure> {
        for row in 1..=self.rows {
            for col in 1..=self.cols {
                out.write_point3(pole(row, col)?);
                if self.weighted {
                    out.write_real(weight(row, col)?);
                }
            }
        }
        Ok(())
    }
}
