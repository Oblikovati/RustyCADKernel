// Rust translation of ShapeCustom_Surface::ConvertToAnalytical (ShapeCustom_Surface.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ConvertToAnalytical`: the plane, cylinder/sphere/cone and torus recognizers and the
//! 21 x 21 point verification of the recognized surface.
use super::ShapeCustomSurface;
use crate::shape_analysis::ShapeAnalysisGeom;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCurve, GeomCylindricalSurface,
    GeomPlane, GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
};
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{self, Axis3, Direction3, Plane3, Point3, Vector3};

/// The parameter box of the recognizers, swapped as the native code swaps it, and the
/// direction of the iso curves used by the verification.
#[derive(Clone, Copy)]
struct IsoFrame {
    u1: f64,
    u2: f64,
    v1: f64,
    v2: f64,
    u_closed: bool,
}

impl IsoFrame {
    fn swapped(self) -> Self {
        Self {
            u1: self.v1,
            u2: self.v2,
            v1: self.u1,
            v2: self.u2,
            ..self
        }
    }
}

/// The recognizer branch chosen from the pole counts, degrees and closure.
enum Shape {
    Planar,
    Revolved { u_closed: bool },
    Toroid,
    Other,
}

fn midpoint(a: Point3, b: Point3) -> Point3 {
    Point3::new(
        0.5 * (a.x() + b.x()),
        0.5 * (a.y() + b.y()),
        0.5 * (a.z() + b.z()),
    )
}

/// One native sample: the iso point at the start, at the middle, their midpoint and its
/// distance to the start (the radius when the iso is a closed circle).
struct IsoSample {
    p1: Point3,
    p3: Point3,
    r: f64,
}

fn iso_sample(iso: &Handle<dyn GeomCurve>, first: f64, last: f64) -> Result<IsoSample, Failure> {
    let p1 = iso.value(first)?;
    let p2 = iso.value(0.5 * (first + last))?;
    let p3 = midpoint(p1, p2);
    Ok(IsoSample {
        p1,
        p3,
        r: p3.distance(&p1),
    })
}

impl ShapeCustomSurface {
    /// `ConvertToAnalytical(tol, substitute)`: the analytic surface equivalent within `tol`
    /// to the B-spline or Bezier surface (a plane for a bilinear patch, a cylinder, cone or
    /// sphere for a surface closed in one direction, a torus for one closed in both),
    /// checked on 21 x 21 iso points; a null handle otherwise. With `substitute` the result
    /// replaces the surface. Native quirk (kept): after a rejected verification the null
    /// result still replaces the surface when `substitute` is set.
    /// Example: `let analytic = tool.convert_to_analytical(1e-6, false)?;`.
    pub fn convert_to_analytical(
        &mut self,
        tol: f64,
        substitute: bool,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let Some(counts) = self.pole_counts() else {
            return Ok(Handle::default());
        };
        let surface = self.surface.clone();
        let (u1, u2, v1, v2) = surface.bounds();
        let frame = IsoFrame {
            u1,
            u2,
            v1,
            v2,
            u_closed: true,
        };
        let (candidate, frame) = match self.shape_kind(counts)? {
            Shape::Planar => return self.planar(frame, tol, substitute),
            Shape::Revolved { u_closed } => revolved(&surface, frame, u_closed, counts, tol)?,
            Shape::Toroid => toroid(&surface, frame, tol)?,
            Shape::Other => (Handle::default(), frame),
        };
        if candidate.is_null() {
            return Ok(candidate);
        }
        let verified = self.verified(&surface, candidate, frame, tol)?;
        if substitute {
            self.init(&verified);
        }
        Ok(verified)
    }

    /// (NbUPoles, NbVPoles, UDegree, VDegree) of a B-spline or Bezier surface.
    fn pole_counts(&self) -> Option<(i32, i32, i32, i32)> {
        if let Some(spline) = self.surface.cast::<GeomBSplineSurface>().get() {
            return Some((
                spline.nb_u_poles(),
                spline.nb_v_poles(),
                spline.u_degree(),
                spline.v_degree(),
            ));
        }
        let bezier = self.surface.cast::<GeomBezierSurface>();
        let bezier = bezier.get()?;
        Some((
            bezier.nb_u_poles(),
            bezier.nb_v_poles(),
            bezier.u_degree(),
            bezier.v_degree(),
        ))
    }

    fn shape_kind(&self, (nu, nv, du, dv): (i32, i32, i32, i32)) -> Result<Shape, Failure> {
        if nu == 2 && nv == 2 {
            return Ok(if du == 1 && dv == 1 {
                Shape::Planar
            } else {
                Shape::Other
            });
        }
        let surface = &self.surface;
        if surface.is_u_closed()? {
            return Ok(if surface.is_v_closed()? {
                Shape::Toroid
            } else {
                Shape::Revolved { u_closed: true }
            });
        }
        Ok(if surface.is_v_closed()? {
            Shape::Revolved { u_closed: false }
        } else {
            Shape::Other
        })
    }

    /// The nearest plane of the four corners, oriented as the patch, when within `tol`.
    fn planar(
        &mut self,
        frame: IsoFrame,
        tol: f64,
        substitute: bool,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let s = self.surface.clone();
        let corners = [
            s.value(frame.u1, frame.v1)?,
            s.value(frame.u2, frame.v1)?,
            s.value(frame.u1, frame.v2)?,
            s.value(frame.u2, frame.v2)?,
        ];
        let points = Array1::from_shared(&corners, 1);
        let Some((plane, gap)) = ShapeAnalysisGeom::nearest_plane(&points)? else {
            return Ok(Handle::default());
        };
        self.gap = gap;
        if gap > tol {
            return Ok(Handle::default());
        }
        let result = oriented_plane(plane, &corners, frame)?;
        if substitute {
            self.init(&result);
        }
        Ok(result)
    }
}

// "IL RESTE a verifier l orientation": the normals of the patch and of the plane must
// agree, else the plane normal (not the whole plane) is reversed.
fn oriented_plane(
    plane: Plane3,
    corners: &[Point3; 4],
    frame: IsoFrame,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surface = GeomPlane::from_pln(plane);
    let old_normal =
        Vector3::between(corners[0], corners[1]).crossed(Vector3::between(corners[0], corners[2]));
    let origin = surface.value(frame.u1, frame.v1)?;
    let new_u = Vector3::between(origin, surface.value(frame.u2, frame.v1)?);
    let new_v = Vector3::between(origin, surface.value(frame.u1, frame.v2)?);
    if old_normal.dot(new_u.crossed(new_v)) >= 0.0 {
        return Ok(Handle::new(surface).cast());
    }
    let mut position = plane.position();
    position.z_reverse();
    position.x_reverse();
    Ok(Handle::new(GeomPlane::from_pln(Plane3::from_position(position))).cast())
}

fn iso_at(
    surface: &Handle<dyn GeomSurface>,
    value: f64,
    u_closed: bool,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    if u_closed {
        surface.v_iso(value)
    } else {
        surface.u_iso(value)
    }
}

/// The cylinder, sphere or cone of a surface closed in one direction (the parameters
/// swapped when it is V that is closed), from three iso circles.
fn revolved(
    surface: &Handle<dyn GeomSurface>,
    frame: IsoFrame,
    u_closed: bool,
    (nu, nv, _, _): (i32, i32, i32, i32),
    tol: f64,
) -> Result<(Handle<dyn GeomSurface>, IsoFrame), Failure> {
    let (frame, nv) = if u_closed {
        (frame, nv)
    } else {
        (
            IsoFrame {
                u_closed,
                ..frame.swapped()
            },
            nu,
        )
    };
    let mut samples = Vec::with_capacity(3);
    let mut iso = Handle::default();
    for v in [frame.v1, frame.v2, 0.5 * (frame.v1 + frame.v2)] {
        iso = iso_at(surface, v, u_closed)?;
        samples.push(iso_sample(&iso, frame.u1, frame.u2)?);
    }
    let orig_d1u = iso.d1(0.)?.1;
    let axis_vector = Vector3::between(samples[0].p3, samples[1].p3);
    let x_vector = Vector3::between(samples[2].p3, samples[2].p1);
    let mut position = Axis3::new(
        samples[0].p3,
        Direction3::from_vector(axis_vector)?,
        Direction3::from_vector(x_vector)?,
    )?;
    // "CKY 3-FEV-1997 : verification du sens de description".
    if position.y_direction().xyz().dot(*orig_d1u.xyz()) < 0. {
        position.y_reverse();
    }
    let surface = revolved_kind(&samples, position, axis_vector, nv, u_closed, tol)?;
    Ok((surface, frame))
}

fn revolved_kind(
    s: &[IsoSample],
    position: Axis3,
    axis_vector: Vector3,
    nv: i32,
    u_closed: bool,
    tol: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if nv > 2 {
        if s[0].r.abs() < tol && s[1].r.abs() < tol && s[2].r.abs() > tol {
            let sphere = GeomSphericalSurface::new(position, s[2].r)?;
            return reversed_unless(Handle::new(sphere).cast(), u_closed);
        }
        return Ok(Handle::default());
    }
    if nv != 2 {
        return Ok(Handle::default());
    }
    if (s[1].r - s[0].r).abs() < tol {
        let cylinder = GeomCylindricalSurface::new(position, s[0].r)?;
        return reversed_unless(Handle::new(cylinder).cast(), u_closed);
    }
    cone(s, position, axis_vector)
}

fn reversed_unless(
    surface: Handle<dyn GeomSurface>,
    u_closed: bool,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    if !u_closed {
        surface.u_reverse()?;
    }
    Ok(surface)
}

// The cone is always U-reversed natively ("if (!uClosed)" is commented out); with the
// larger radius first the cone is built from the second circle, axis reversed.
fn cone(
    s: &[IsoSample],
    position: Axis3,
    axis_vector: Vector3,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let angle = axis_vector.angle(Vector3::between(s[0].p1, s[1].p1))?;
    let cone = if s[0].r < s[1].r {
        GeomConicalSurface::new(position, angle, s[0].r)?
    } else {
        let mut axis = Direction3::from_vector(axis_vector)?;
        axis.reverse();
        let x = Direction3::from_vector(Vector3::between(s[1].p3, s[1].p1))?;
        GeomConicalSurface::new(Axis3::new(s[1].p3, axis, x)?, angle, s[1].r)?
    };
    let cone: Handle<dyn GeomSurface> = Handle::new(cone).cast();
    cone.u_reverse()?;
    Ok(cone)
}

/// The torus of a surface closed in both directions, tried on U isos then (bounds
/// swapped) on V isos; the frame keeps the swap for the verification, as natively.
fn toroid(
    surface: &Handle<dyn GeomSurface>,
    frame: IsoFrame,
    tol: f64,
) -> Result<(Handle<dyn GeomSurface>, IsoFrame), Failure> {
    let mut frame = frame;
    for attempt in 1..=2 {
        if attempt == 2 {
            frame = frame.swapped();
        }
        let mut samples = Vec::with_capacity(3);
        for u in [
            frame.u1,
            0.5 * (frame.u1 + frame.u2),
            0.25 * (frame.u1 + frame.u2),
        ] {
            samples.push(iso_sample(&surface.u_iso(u)?, frame.v1, frame.v2)?);
        }
        let (r1, r2, r3) = (samples[0].r, samples[1].r, samples[2].r);
        if (r1 - r2).abs() < tol && (r1 - r3).abs() < tol {
            return Ok((torus(&samples, attempt == 2)?, frame));
        }
    }
    Ok((Handle::default(), frame))
}

// "modif empirique (pourtant NON DEMONTREE) : inverser roles RR1,RR2" (CKY, 24-JAN-1997).
fn torus(s: &[IsoSample], swapped: bool) -> Result<Handle<dyn GeomSurface>, Failure> {
    let centre = midpoint(s[0].p3, s[1].p3);
    let mut normal = Vector3::between(centre, s[0].p3).crossed(Vector3::between(centre, s[2].p3));
    if normal.magnitude() <= gp::resolution() {
        normal = Vector3::new(0., 0., 1.);
    }
    let position = Axis3::from_normal(centre, Direction3::from_vector(normal)?)?;
    let (mut major, mut minor) = (centre.distance(&s[0].p3), s[1].r);
    if major < minor {
        std::mem::swap(&mut major, &mut minor);
    }
    let torus: Handle<dyn GeomSurface> =
        Handle::new(GeomToroidalSurface::new(position, major, minor)?).cast();
    if swapped {
        torus.u_reverse()?;
    }
    Ok(torus)
}

impl ShapeCustomSurface {
    /// The native verification: 21 x 21 points of the iso curves projected on the
    /// candidate (ElSLib parameters); the largest distance is the gap, and a distance
    /// above `tol` rejects the candidate (a null result).
    fn verified(
        &mut self,
        surface: &Handle<dyn GeomSurface>,
        candidate: Handle<dyn GeomSurface>,
        frame: IsoFrame,
        tol: f64,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        const NP: i32 = 21;
        self.gap = 0.;
        let du = (frame.u2 - frame.u1) / f64::from(NP - 1);
        let dv = (frame.v2 - frame.v1) / f64::from(NP - 1);
        let mut st = (0., 0.);
        for j in 1..=NP {
            let iso = iso_at(surface, frame.v1 + dv * f64::from(j - 1), frame.u_closed)?;
            for i in 1..=NP {
                let point = iso.value(frame.u1 + du * f64::from(i - 1))?;
                st = parameters_on(&candidate, point, st)?;
                let distance = point.distance(&candidate.value(st.0, st.1)?);
                if distance > self.gap {
                    self.gap = distance;
                }
                if distance > tol {
                    return Ok(Handle::default());
                }
            }
        }
        Ok(candidate)
    }
}

/// `ElSLib::Parameters` on the candidate's gp surface; another kind keeps the previous
/// parameters (the native `default: break`).
fn parameters_on(
    candidate: &Handle<dyn GeomSurface>,
    point: Point3,
    previous: (f64, f64),
) -> Result<(f64, f64), Failure> {
    if let Some(c) = candidate.cast::<GeomCylindricalSurface>().get() {
        return Ok(elslib::parameters(&c.cylinder()?, point));
    }
    if let Some(c) = candidate.cast::<GeomConicalSurface>().get() {
        return Ok(elslib::parameters(&c.cone()?, point));
    }
    if let Some(s) = candidate.cast::<GeomSphericalSurface>().get() {
        return Ok(elslib::parameters(&s.sphere()?, point));
    }
    if let Some(t) = candidate.cast::<GeomToroidalSurface>().get() {
        return Ok(elslib::parameters(&t.torus()?, point));
    }
    Ok(previous)
}
