// Rust translation of the IntPatch_Intersection constructor and Perform overload of one
// surface and the Perform overload from a start point
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The self-intersection of a surface (none for a quadric or an extrusion of a simple
//! curve) and the intersection from a start point.
use super::IntPatchIntersection;
use super::perform::{IntPatchIntersectionOptions, Pair};
use crate::geom2d_int::GInter;
use crate::int_patch::{IntPatchPrmPrmIntersection, IntPatchPrmPrmTolerances};
use crate::int_start::TheTopolTool;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkg3d::geom::GeomPlane;
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeombase::proj_lib::{self, ProjectOnPlane, ProjectedCurve};
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{self, Axis3};

/// Whether the extrusion has no self-intersection: its basis curve projected on the
/// plane normal to the direction does not cross itself (a closed projection may touch
/// itself once, at its ends).
fn is_simple_extrusion(s1: &Adaptor3dSurfaceHandle) -> Result<bool, Failure> {
    let an_axis = Axis3::from_normal(gp::origin(), s1.direction()?)?;
    let projector = ProjectOnPlane::with_plane(&an_axis);
    projector.load(&s1.basis_curve()?, precision::confusion(), false)?;
    let proj_curve = projector.get_result();
    let a_plane: Handle<GeomPlane> = Handle::new(GeomPlane::new(an_axis));
    let ga_hsurf: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&a_plane.cast())?).cast();
    let projected = ProjectedCurve::with_curve(&ga_hsurf, &proj_curve.cast())?;
    let pcurve: Handle<dyn Geom2dCurve> = proj_lib::make_pcurve_of_type(&projected)?;
    let ac = Geom2dAdaptorCurve::with_range(
        &pcurve,
        projected.first_parameter()?,
        projected.last_parameter()?,
    )?;
    let intersector = GInter::of_self(&ac, precision::confusion(), precision::confusion())?;
    Ok(intersector.is_done()
        && (intersector.is_empty()? || (ac.is_closed()? && intersector.nb_points()? == 1)))
}

impl IntPatchIntersection {
    /// `IntPatch_Intersection(S1, D1, TolArc, TolTang)`: the self-intersection of `s1`.
    ///
    /// ```ignore
    /// let inter = IntPatchIntersection::with_surface(&s1, &*d1, 1e-7, 1e-7)?;
    /// ```
    pub fn with_surface(
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        tol_arc: f64,
        tol_tang: f64,
    ) -> Result<Self, Failure> {
        let mut inter = Self::new();
        inter.my_tol_arc = tol_arc;
        inter.my_tol_tang = tol_tang;
        inter.perform_self(s1, d1, tol_arc, tol_tang)?;
        Ok(inter)
    }

    /// `Perform(S1, D1, TolArc, TolTang)`: the self-intersection of the surface; a
    /// quadric, or an extrusion of a curve whose projection does not cross itself, has
    /// none; another surface runs the parametric self-intersection.
    ///
    /// ```ignore
    /// inter.perform_self(&s1, &*d1, 1e-7, 1e-7)?;
    /// ```
    pub fn perform_self(
        &mut self,
        s1: &Adaptor3dSurfaceHandle,
        d1: &(dyn TheTopolTool + 'static),
        tol_arc: f64,
        tol_tang: f64,
    ) -> Result<(), Failure> {
        self.my_tol_arc = tol_arc;
        self.my_tol_tang = tol_tang;
        if self.my_fleche == 0.0 {
            self.my_fleche = 0.01;
        }
        if self.my_uv_max_step == 0.0 {
            self.my_uv_max_step = 0.01;
        }
        self.done = true;
        self.spnt.clear();
        self.slin.clear();
        (self.empt, self.tgte, self.oppo) = (true, false, false);
        let typ: SurfaceType = s1.get_type()?;
        let simple: bool = match typ {
            SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Sphere
            | SurfaceType::Cone
            | SurfaceType::Torus => true,
            SurfaceType::SurfaceOfExtrusion => is_simple_extrusion(s1)?,
            _ => false,
        };
        if simple {
            return Ok(());
        }
        let mut interpp = IntPatchPrmPrmIntersection::new();
        let tolerances = IntPatchPrmPrmTolerances {
            tol_tangency: tol_tang,
            epsilon: tol_arc,
            deflection: self.my_fleche,
            increment: self.my_uv_max_step,
        };
        interpp.perform_self(s1, d1, tolerances)?;
        if interpp.is_done() {
            self.done = true;
            self.tgte = false;
            self.empt = interpp.is_empty()?;
            for i in 1..=interpp.nb_lines()? {
                self.slin.push(interpp.line(i)?);
            }
        }
        Ok(())
    }

    /// `Perform(S1, D1, S2, D2, U1, V1, U2, V2, TolArc, TolTang)`: with a quadric, the
    /// intersection of two surfaces with the start point given to the
    /// implicit-parametric intersector; else the parametric line through the start
    /// point. The walking lines are purged.
    ///
    /// ```ignore
    /// inter.perform_from_start(&s1, &*d1, &s2, &*d2, [0.5, 0.5, 0.5, 0.5], (1e-7, 1e-7))?;
    /// ```
    pub fn perform_from_start(
        &mut self,
        (s1, d1): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
        (s2, d2): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
        start: [f64; 4],
        (tol_arc, tol_tang): (f64, f64),
    ) -> Result<(), Failure> {
        self.my_tol_arc = tol_arc;
        self.my_tol_tang = tol_tang;
        if self.my_fleche == 0.0 {
            self.my_fleche = 0.01;
        }
        if self.my_uv_max_step == 0.0 {
            self.my_uv_max_step = 0.01;
        }
        self.done = false;
        self.spnt.clear();
        self.slin.clear();
        (self.empt, self.tgte, self.oppo) = (true, false, false);
        let is_quadric = |t: SurfaceType| {
            matches!(
                t,
                SurfaceType::Plane
                    | SurfaceType::Cylinder
                    | SurfaceType::Sphere
                    | SurfaceType::Cone
            )
        };
        if is_quadric(s1.get_type()?) || is_quadric(s2.get_type()?) {
            self.my_is_start_pnt = true;
            self.my_start = start;
            self.perform(
                s1,
                d1,
                s2,
                d2,
                (tol_arc, tol_tang),
                IntPatchIntersectionOptions::default(),
            )?;
            self.my_is_start_pnt = false;
        } else {
            let mut interpp = IntPatchPrmPrmIntersection::new();
            let t = self.prm_tolerances((tol_arc, tol_tang));
            interpp.perform_from_start((s1, d1), (s2, d2), start, t)?;
            if interpp.is_done() {
                self.done = true;
                self.tgte = false;
                self.empt = interpp.is_empty()?;
                for i in 1..=interpp.nb_lines()? {
                    self.slin.push(interpp.line(i)?);
                }
            }
        }
        self.purge_wlines(Pair { s1, d1, s2, d2 })
    }
}
