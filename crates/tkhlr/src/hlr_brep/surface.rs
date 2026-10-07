//! HLRBRep_Surface.hxx/.lxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::curve::null_projector;
use super::{BSurfaceTool, Curve};
use crate::hlr_algo::Projector;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::geom::{GeomSurfaceD1, GeomSurfaceD2, GeomSurfaceD3};
use rustycad_tkgeombase::gprop::PEquation;
use rustycad_tkmath::geom_abs::{CurveType, Shape, SurfaceType};
use rustycad_tkmath::gp::{
    Axis1, Cone3, Cylinder3, Direction3, Plane3, Point3, Sphere3, Torus3, Vector3,
};
use std::sync::Arc;

/// HLRBRep_Surface: a face's surface seen through the projector of the hidden line
/// data structure. The type is the adaptor's for the elementary surfaces, Plane for a
/// bilinear Bezier, Bezier for another Bezier and OtherSurface for any other surface
/// (B-splines, swept and offset surfaces included). The projector is shared as in
/// [`Curve`] (docs/hlr-design.md section 6).
///
/// ```
/// use rustycad_tkhlr::hlr_brep::Surface;
/// let surface = Surface::new();
/// assert!(surface.is_side(1e-7, 1e-7).is_err()); // no projector yet
/// ```
pub struct Surface {
    surf: BRepAdaptorSurface,
    surf_type: SurfaceType,
    projector: Option<Arc<Projector>>,
}

impl Default for Surface {
    fn default() -> Self {
        Self {
            surf: BRepAdaptorSurface::default(),
            surf_type: SurfaceType::OtherSurface,
            projector: None,
        }
    }
}

impl Surface {
    /// HLRBRep_Surface(): no face, type OtherSurface, no projector; `Surface::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Projector(Proj): the projector shared with the data structure;
    /// `surface.set_projector(Some(projector))`.
    pub fn set_projector(&mut self, projector: Option<Arc<Projector>>) {
        self.projector = projector;
    }

    // The shared projector, compared by identity (see `Curve::projector_handle`).
    pub(super) fn projector_handle(&self) -> Option<&Arc<Projector>> {
        self.projector.as_ref()
    }

    // The native copy: a new adaptor on the same face (restricted, as `Surface(F)`).
    pub(super) fn try_clone(&self) -> Result<Self, Failure> {
        let surf = BRepAdaptorSurface::default();
        let face = self.surf.face();
        if !face.is_null() {
            surf.initialize(&face, true)?;
        }
        Ok(Self {
            surf,
            surf_type: self.surf_type,
            projector: self.projector.clone(),
        })
    }

    fn projector(&self) -> Result<&Projector, Failure> {
        self.projector
            .as_deref()
            .ok_or_else(|| null_projector("HLRBRep_Surface"))
    }

    /// Surface(): the surface adaptor (interior mutability, as the native reference);
    /// `surface.surface()`.
    pub fn surface(&self) -> &BRepAdaptorSurface {
        &self.surf
    }

    /// Surface(F): loads the face with its restriction and classifies its type;
    /// `surface.set_surface(&face)?`.
    pub fn set_surface(&mut self, face: &Face) -> Result<(), Failure> {
        // mySurf.Initialize(F,false);
        self.surf.initialize(face, true)?;
        let typ = BSurfaceTool::get_type(&self.surf)?;
        self.surf_type = match typ {
            SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Cone
            | SurfaceType::Sphere
            | SurfaceType::Torus => typ,
            SurfaceType::BezierSurface if self.is_bilinear()? => SurfaceType::Plane,
            SurfaceType::BezierSurface => typ,
            _ => SurfaceType::OtherSurface,
        };
        Ok(())
    }

    fn is_bilinear(&self) -> Result<bool, Failure> {
        Ok(BSurfaceTool::u_degree(&self.surf)? == 1 && BSurfaceTool::v_degree(&self.surf)? == 1)
    }

    /// IsSide(tolf, toler): whether the face is seen edge-on (a plane through the
    /// sight, a cylinder along it, a cone with its apex at the eye, a spline whose
    /// pole rows project to points or whose poles lie in a side plane);
    /// `surface.is_side(1e-7, 1e-7)?`.
    pub fn is_side(&self, tol_f: f64, toler: f64) -> Result<bool, Failure> {
        let projector = self.projector()?;
        match self.surf_type {
            SurfaceType::Plane => self.plane_is_side(projector, toler),
            SurfaceType::Cylinder if projector.perspective() => Ok(false),
            SurfaceType::Cylinder => {
                let mut d = Vector3::from(BSurfaceTool::cylinder(&self.surf)?.axis().direction());
                d.transform(projector.transformation());
                Ok((d.x() * d.x() + d.y() * d.y()).sqrt() < toler)
            }
            SurfaceType::Cone if !projector.perspective() => Ok(false),
            SurfaceType::Cone => {
                let mut pt = BSurfaceTool::cone(&self.surf)?.apex();
                pt.transform(projector.transformation());
                let tol = 0.001;
                Ok(pt.is_equal(&Point3::new(0., 0., projector.focus()?), tol))
            }
            SurfaceType::BezierSurface | SurfaceType::BSplineSurface if projector.perspective() => {
                Ok(false)
            }
            SurfaceType::BezierSurface => self.poles_are_side(tol_f, false),
            SurfaceType::BSplineSurface => self.poles_are_side(tol_f, true),
            _ => Ok(false),
        }
    }

    fn plane_is_side(&self, projector: &Projector, toler: f64) -> Result<bool, Failure> {
        let a: Axis1 = self.plane()?.axis();
        let mut pt = a.location();
        let mut d = Vector3::from(a.direction());
        pt.transform(projector.transformation());
        d.transform(projector.transformation());
        let r = if projector.perspective() {
            d.z() * projector.focus()? - (d.x() * pt.x() + d.y() * pt.y() + d.z() * pt.z())
        } else {
            d.z()
        };
        Ok(r.abs() < toler)
    }

    // The pole grid of the Bezier (or B-spline) surface, then SideRowsOfPoles.
    fn poles_are_side(&self, tol_f: f64, bspline: bool) -> Result<bool, Failure> {
        let nu = BSurfaceTool::nb_u_poles(&self.surf)?;
        let nv = BSurfaceTool::nb_v_poles(&self.surf)?;
        let source = if bspline {
            BSurfaceTool::bspline(&self.surf)?.poles()
        } else {
            BSurfaceTool::bezier(&self.surf)?.poles()
        };
        let mut grid = PoleGrid {
            nv,
            poles: Vec::new(),
        };
        for iu in 1..=nu {
            for iv in 1..=nv {
                let pole = source
                    .value(iu, iv)
                    .map_err(|e| e.into_failure("HLRBRep_Surface::IsSide"))?;
                grid.poles.push(*pole);
            }
        }
        self.side_rows_of_poles(tol_f, nu, nv, &mut grid)
    }

    // SideRowsOfPoles: the poles in the viewing frame; true when every iso-u (or
    // iso-v) row projects to one point within `tol`, or when the poles are planar
    // in a plane whose normal is perpendicular to the sight.
    fn side_rows_of_poles(
        &self,
        tol: f64,
        nbu_poles: i32,
        nbv_poles: i32,
        grid: &mut PoleGrid,
    ) -> Result<bool, Failure> {
        let t = self.projector()?.transformation();
        grid.poles.iter_mut().for_each(|p| p.transform(t));
        // Side iso u ?
        if grid.rows_collapse(nbu_poles, tol, false) {
            return Ok(true);
        }
        // Side iso v ?
        if grid.rows_collapse(nbu_poles, tol, true) {
            return Ok(true);
        }
        // Are the Poles in a Side Plane ?
        let mut p = Array1::with_bounds(1, nbu_poles * nbv_poles)
            .map_err(|e| e.into_failure("HLRBRep_Surface::SideRowsOfPoles"))?;
        for (i, pole) in (1..).zip(&grid.poles) {
            p[i] = *pole;
        }
        let pl = PEquation::new(&p, tol);
        if !pl.is_planar() {
            // The native result is still false here: the iso-v loop left it so.
            return Ok(false);
        }
        Ok(pl.plane()?.axis().direction().z().abs() < 0.0001)
    }

    /// IsAbove(back, A, tolC): whether the curve lies above the plane of the face
    /// (below with `back`) within `tol_c`, at its ends and, if it is not a line, 30
    /// inner samples; false for a face that is not planar;
    /// `surface.is_above(false, &curve, 1e-7)?`.
    pub fn is_above(&self, back: bool, a: &Curve, tol: f64) -> Result<bool, Failure> {
        if self.surf_type != SurfaceType::Plane {
            return Ok(false);
        }
        let [pa, pb, pc, pd] = self.plane()?.coefficients()?;
        let below = |u: f64| -> Result<bool, Failure> {
            let p = a.d0_3d(u)?;
            let mut dd = pa * p.x() + pb * p.y() + pc * p.z() + pd;
            if back {
                dd = -dd;
            }
            Ok(dd < -tol)
        };
        let u1 = a.parameter_3d(a.first_parameter()?)?;
        let u2 = a.parameter_3d(a.last_parameter()?)?;
        let mut u = u1;
        if below(u)? {
            return Ok(false);
        }
        if a.get_type() != CurveType::Line {
            let nb_pnt = 30;
            let step = (u2 - u1) / f64::from(nb_pnt + 1);
            for _ in 1..=nb_pnt {
                u += step;
                if below(u)? {
                    return Ok(false);
                }
            }
        }
        Ok(!below(u2)?)
    }

    /// FirstUParameter(); `surface.first_u_parameter()?`.
    pub fn first_u_parameter(&self) -> Result<f64, Failure> {
        BSurfaceTool::first_u_parameter(&self.surf)
    }
    /// LastUParameter(); `surface.last_u_parameter()?`.
    pub fn last_u_parameter(&self) -> Result<f64, Failure> {
        BSurfaceTool::last_u_parameter(&self.surf)
    }
    /// FirstVParameter(); `surface.first_v_parameter()?`.
    pub fn first_v_parameter(&self) -> Result<f64, Failure> {
        BSurfaceTool::first_v_parameter(&self.surf)
    }
    /// LastVParameter(); `surface.last_v_parameter()?`.
    pub fn last_v_parameter(&self) -> Result<f64, Failure> {
        BSurfaceTool::last_v_parameter(&self.surf)
    }
    /// UContinuity(); `surface.u_continuity()?`.
    pub fn u_continuity(&self) -> Result<Shape, Failure> {
        BSurfaceTool::u_continuity(&self.surf)
    }
    /// VContinuity(); `surface.v_continuity()?`.
    pub fn v_continuity(&self) -> Result<Shape, Failure> {
        BSurfaceTool::v_continuity(&self.surf)
    }
    /// NbUIntervals(S); `surface.nb_u_intervals(Shape::C2)?`.
    pub fn nb_u_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        BSurfaceTool::nb_u_intervals(&self.surf, shape)
    }
    /// NbVIntervals(S); `surface.nb_v_intervals(Shape::C2)?`.
    pub fn nb_v_intervals(&self, shape: Shape) -> Result<i32, Failure> {
        BSurfaceTool::nb_v_intervals(&self.surf, shape)
    }
    /// IsUClosed(); `surface.is_u_closed()?`.
    pub fn is_u_closed(&self) -> Result<bool, Failure> {
        BSurfaceTool::is_u_closed(&self.surf)
    }
    /// IsVClosed(); `surface.is_v_closed()?`.
    pub fn is_v_closed(&self) -> Result<bool, Failure> {
        BSurfaceTool::is_v_closed(&self.surf)
    }
    /// IsUPeriodic(); `surface.is_u_periodic()?`.
    pub fn is_u_periodic(&self) -> Result<bool, Failure> {
        BSurfaceTool::is_u_periodic(&self.surf)
    }
    /// UPeriod(); `surface.u_period()?`.
    pub fn u_period(&self) -> Result<f64, Failure> {
        BSurfaceTool::u_period(&self.surf)
    }
    /// IsVPeriodic(); `surface.is_v_periodic()?`.
    pub fn is_v_periodic(&self) -> Result<bool, Failure> {
        BSurfaceTool::is_v_periodic(&self.surf)
    }
    /// VPeriod(); `surface.v_period()?`.
    pub fn v_period(&self) -> Result<f64, Failure> {
        BSurfaceTool::v_period(&self.surf)
    }
    /// Value(U, V); `surface.value(0.5, 0.5)?`.
    pub fn value(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        self.d0(u, v)
    }
    /// D0(U, V, P); `surface.d0(0.5, 0.5)?`.
    pub fn d0(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        BSurfaceTool::d0(&self.surf, u, v)
    }
    /// D1(U, V, P, D1U, D1V); `surface.d1(0.5, 0.5)?`.
    pub fn d1(&self, u: f64, v: f64) -> Result<GeomSurfaceD1, Failure> {
        BSurfaceTool::d1(&self.surf, u, v)
    }
    /// D2(U, V, P, ...); `surface.d2(0.5, 0.5)?`.
    pub fn d2(&self, u: f64, v: f64) -> Result<GeomSurfaceD2, Failure> {
        BSurfaceTool::d2(&self.surf, u, v)
    }
    /// D3(U, V, P, ...); `surface.d3(0.5, 0.5)?`.
    pub fn d3(&self, u: f64, v: f64) -> Result<GeomSurfaceD3, Failure> {
        BSurfaceTool::d3(&self.surf, u, v)
    }
    /// DN(U, V, Nu, Nv); `surface.dn(0.5, 0.5, 1, 1)?`.
    pub fn dn(&self, u: f64, v: f64, nu: i32, nv: i32) -> Result<Vector3, Failure> {
        BSurfaceTool::dn(&self.surf, u, v, nu, nv)
    }
    /// GetType(): the type classified by `Surface(F)`; `surface.get_type()`.
    pub fn get_type(&self) -> SurfaceType {
        self.surf_type
    }
    /// Plane(): the adaptor's plane, or for a Bezier the plane through its point at
    /// (0.5, 0.5) normal to `D1U ^ D1V`; `surface.plane()?`.
    pub fn plane(&self) -> Result<Plane3, Failure> {
        if BSurfaceTool::get_type(&self.surf)? != SurfaceType::BezierSurface {
            return BSurfaceTool::plane(&self.surf);
        }
        let d1 = self.d1(0.5, 0.5)?;
        Plane3::new(d1.point, Direction3::from_vector(d1.d1u.crossed(d1.d1v))?)
    }
    /// Cylinder(); `surface.cylinder()?`.
    pub fn cylinder(&self) -> Result<Cylinder3, Failure> {
        BSurfaceTool::cylinder(&self.surf)
    }
    /// Cone(); `surface.cone()?`.
    pub fn cone(&self) -> Result<Cone3, Failure> {
        BSurfaceTool::cone(&self.surf)
    }
    /// Sphere(); `surface.sphere()?`.
    pub fn sphere(&self) -> Result<Sphere3, Failure> {
        BSurfaceTool::sphere(&self.surf)
    }
    /// Torus(); `surface.torus()?`.
    pub fn torus(&self) -> Result<Torus3, Failure> {
        BSurfaceTool::torus(&self.surf)
    }
    /// UDegree(); `surface.u_degree()?`.
    pub fn u_degree(&self) -> Result<i32, Failure> {
        BSurfaceTool::u_degree(&self.surf)
    }
    /// NbUPoles(); `surface.nb_u_poles()?`.
    pub fn nb_u_poles(&self) -> Result<i32, Failure> {
        BSurfaceTool::nb_u_poles(&self.surf)
    }
    /// VDegree(); `surface.v_degree()?`.
    pub fn v_degree(&self) -> Result<i32, Failure> {
        BSurfaceTool::v_degree(&self.surf)
    }
    /// NbVPoles(); `surface.nb_v_poles()?`.
    pub fn nb_v_poles(&self) -> Result<i32, Failure> {
        BSurfaceTool::nb_v_poles(&self.surf)
    }
    /// NbUKnots(); `surface.nb_u_knots()?`.
    pub fn nb_u_knots(&self) -> Result<i32, Failure> {
        BSurfaceTool::nb_u_knots(&self.surf)
    }
    /// NbVKnots(); `surface.nb_v_knots()?`.
    pub fn nb_v_knots(&self) -> Result<i32, Failure> {
        BSurfaceTool::nb_v_knots(&self.surf)
    }
    /// Axis(): the axis of revolution; `surface.axis()?`.
    pub fn axis(&self) -> Result<Axis1, Failure> {
        BSurfaceTool::axe_of_revolution(&self.surf)
    }
}

// The poles of a spline surface, row-major (`Pnt(iu, iv)` at `(iu - 1) * nv + iv - 1`).
struct PoleGrid {
    nv: i32,
    poles: Vec<Point3>,
}

impl PoleGrid {
    fn at(&self, iu: i32, iv: i32) -> Point3 {
        self.poles[((iu - 1) * self.nv + iv - 1) as usize]
    }

    // Whether every row (iso-u when `along_u` is false, iso-v otherwise) projects to
    // one point within `tol`; the native loops stop at the first failure, and NaN
    // coordinates fail the test.
    fn rows_collapse(&self, nbu: i32, tol: f64, along_u: bool) -> bool {
        let (rows, columns) = if along_u {
            (self.nv, nbu)
        } else {
            (nbu, self.nv)
        };
        let at = |row: i32, column: i32| {
            if along_u {
                self.at(column, row)
            } else {
                self.at(row, column)
            }
        };
        (1..=rows).all(|row| {
            let first = at(row, 1);
            (2..=columns).all(|column| {
                let p = at(row, column);
                (p.x() - first.x()).abs() < tol && (p.y() - first.y()).abs() < tol
            })
        })
    }
}
