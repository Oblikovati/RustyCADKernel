// Rust translation of BRepFill_MultiLine::Curves
// (ModelingAlgorithms/TKBool/BRepFill/BRepFill_MultiLine.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The exact curves of the particular cases: an iso of the first face (case 1) or the
//! bisector projected on the planes of both faces (case 2).
use super::BRepFillMultiLine;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{
    GeomCurve, GeomLine, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface, GeomTrimmedCurve,
};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3};

/// The curves of the multi-line: the 3d curve and the p-curves on both faces.
pub type MultiLineCurves = (
    Handle<dyn GeomCurve>,
    Handle<dyn Geom2dCurve>,
    Handle<dyn Geom2dCurve>,
);

/// The surface of a face transformed by its location.
fn located_surface(face: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let (s, l) = BRepTool::surface_with_location(face)?;
    if l.is_identity() {
        return Ok(s);
    }
    Ok(s.get()
        .ok_or_else(null_geometry)?
        .transformed(l.transformation())?
        .cast())
}

/// The plane of a face (the basis of a rectangular trimmed surface).
fn face_plane(face: &Face) -> Result<Handle<dyn GeomSurface>, Failure> {
    let s = located_surface(face)?;
    let trimmed: Handle<GeomRectangularTrimmedSurface> = s.cast();
    let basis = match trimmed.get() {
        Some(t) => t.basis_surface(),
        None => s,
    };
    Ok(basis.cast::<GeomPlane>().cast())
}

/// The line of a p-curve through the middle iso with the 3d parameters, trimmed.
fn iso_pcurve(
    along_y: bool,
    (p1, pmil): (Point2, Point2),
    sens: bool,
    (f, l): (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    let line = match (along_y, sens) {
        (true, true) => Geom2dLine::from_point(Point2::new(pmil.x(), p1.y() - f), Direction2::Y),
        (true, false) => {
            Geom2dLine::from_point(Point2::new(pmil.x(), p1.y() + f), Direction2::Y.reversed())
        }
        (false, true) => Geom2dLine::from_point(Point2::new(p1.x() - f, pmil.y()), Direction2::X),
        (false, false) => {
            Geom2dLine::from_point(Point2::new(p1.x() + f, pmil.y()), Direction2::X.reversed())
        }
    };
    let line: Handle<dyn Geom2dCurve> = Handle::new(line).cast();
    Ok(Handle::new(Geom2dTrimmedCurve::new(&line, f, l, true, true)?).cast())
}

impl BRepFillMultiLine {
    /// `Curves(Curve, PCurve1, PCurve2)`: the exact curves of a particular case (null
    /// otherwise); `let (c, pc1, pc2) = ml.curves()?;`.
    pub fn curves(&self) -> Result<MultiLineCurves, Failure> {
        match self.my_k_part {
            1 => self.iso_curves(),
            2 => self.plane_curves(),
            _ => Ok((Handle::default(), Handle::default(), Handle::default())),
        }
    }

    /// The ends and the middle of the bisector on a face.
    fn ends_on(&self, first_face: bool) -> Result<(Point2, Point2, Point2), Failure> {
        let (bf, bl) = (
            self.my_bis.first_parameter()?,
            self.my_bis.last_parameter()?,
        );
        let on = |u: f64| {
            if first_face {
                self.value_on_f1(u)
            } else {
                self.value_on_f2(u)
            }
        };
        Ok((on(bf)?, on(bl)?, on(0.5 * (bf + bl))?))
    }

    /// Case 1: the iso of the first face through the middle point between the ends,
    /// and the lines of the p-curves on both faces.
    fn iso_curves(&self) -> Result<MultiLineCurves, Failure> {
        let (p1, p2, pmil) = self.ends_on(true)?;
        let s = located_surface(&self.my_face1)?;
        let surface = s.get().ok_or_else(null_geometry)?;
        let (iso, a, b) = if !self.my_iso_u1 {
            (surface.u_iso(pmil.x())?, p1.y(), p2.y())
        } else {
            (surface.v_iso(pmil.y())?, p1.x(), p2.x())
        };
        let sens: bool = a < b;
        let (lo, hi) = if sens { (a, b) } else { (b, a) };
        let curve: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&iso, lo, hi, sens, true)?).cast();
        let c = curve.get().ok_or_else(null_geometry)?;
        let (f, l) = (c.first_parameter(), c.last_parameter());
        let pcurve1 = iso_pcurve(!self.my_iso_u1, (p1, pmil), sens, (f, l))?;
        let (q1, q2, qmil) = self.ends_on(false)?;
        let sens2: bool = if !self.my_iso_u2 {
            q1.y() < q2.y()
        } else {
            q1.x() < q2.x()
        };
        let pcurve2 = iso_pcurve(!self.my_iso_u2, (q1, qmil), sens2, (f, l))?;
        Ok((curve, pcurve1, pcurve2))
    }

    /// Case 2: the line bisector in 3d projected on the plane of the first face along
    /// Z, and its p-curves on both planes.
    fn plane_curves(&self) -> Result<MultiLineCurves, Failure> {
        let plane = face_plane(&self.my_face1)?;
        let bis_line = self.my_bis.line()?;
        let (p, d) = (bis_line.location(), bis_line.direction());
        let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
            Point3::new(p.x(), p.y(), 0.),
            Direction3::new(d.x(), d.y(), 0.)?,
        ))
        .cast();
        let (bf, bl) = (
            self.my_bis.first_parameter()?,
            self.my_bis.last_parameter()?,
        );
        let t_line: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&line, bf, bl, true, true)?).cast();
        let curve = geom_proj_lib::project_on_plane(
            &t_line,
            &plane.cast::<GeomPlane>(),
            Direction3::Z,
            false,
        )?;
        let pcurve1 = geom_proj_lib::curve2d(&curve, &plane)?;
        let plane2 = face_plane(&self.my_face2)?;
        let pcurve2 = geom_proj_lib::curve2d(&curve, &plane2)?;
        Ok((curve, pcurve1, pcurve2))
    }
}

/// Standard_NullObject of a null surface or curve (natively a crash).
fn null_geometry() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "BRepFill_MultiLine: null surface or curve; expected a geometry",
        "BRepFill_MultiLine",
    )
}
