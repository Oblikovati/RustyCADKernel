// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_NSections.hxx/.cxx
// (the constructors, SetSurface and ComputeSurface) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_NSections`: the section law of several sections, through the surface
//! that skins them (docs/geomalgo-design.md, section 14).
use super::app_surf::GeomFillAppSurf;
use super::line::GeomFillLine;
use super::section_generator::GeomFillSectionGenerator;
use super::section_law::GeomFillSectionLaw;
use super::sweep_failures::live;
use crate::app_blend::AppBlendApprox;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::one_based;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomBSplineSurface, GeomCurve, GeomGeometry};
use rustycad_tkgeombase::geom_convert::curve_to_bspline_curve;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Transform3;

/// The surfaces of the law: the reference surface (`myRefSurf`) and the skinned
/// or segmented surface the evaluators read (`mySurface`).
#[derive(Clone, Default)]
pub(crate) struct NSectionsSurfaces {
    pub ref_surf: Handle<GeomBSplineSurface>,
    pub surface: Handle<GeomBSplineSurface>,
}

/// `GeomFill_NSections`: the sections (`mySections`), their parameters
/// (`myParams`), their transformations (`myTrsfs`), the section domain `[UFirst,
/// ULast]` and the path domain `[VFirst, VLast]`.
///
/// A B-spline section is shared with the caller, and `ComputeSurface`
/// reparameterises it in place, as natively.
/// `let law = GeomFillNSections::with_params(&[c1, c2], &[0.0, 1.0])?;`.
pub struct GeomFillNSections {
    parent: StandardTransient,
    pub(crate) my_sections: Vec<Handle<dyn GeomCurve>>,
    pub(crate) my_trsfs: Vec<Transform3>,
    pub(crate) my_params: Vec<f64>,
    pub(crate) u_first: f64,
    pub(crate) u_last: f64,
    pub(crate) v_first: f64,
    pub(crate) v_last: f64,
    pub(crate) surfaces: SharedState<NSectionsSurfaces>,
}

impl_transient!(
    GeomFillNSections,
    "GeomFill_NSections",
    dyn GeomFillSectionLaw,
    parent,
    interfaces: [dyn GeomFillSectionLaw]
);

impl GeomFillNSections {
    fn build(nc: &[Handle<dyn GeomCurve>], np: &[f64], u: (f64, f64), v: (f64, f64)) -> Self {
        Self {
            parent: StandardTransient::default(),
            my_sections: nc.to_vec(),
            my_trsfs: Vec::new(),
            my_params: np.to_vec(),
            u_first: u.0,
            u_last: u.1,
            v_first: v.0,
            v_last: v.1,
            surfaces: SharedState::new(NSectionsSurfaces::default()),
        }
    }

    /// `GeomFill_NSections(NC)`: sections on `[0, 1]` in both directions;
    /// `GeomFillNSections::new(&sections)?`.
    pub fn new(nc: &[Handle<dyn GeomCurve>]) -> Result<Self, Failure> {
        Self::with_domain(nc, &[], (0.0, 1.0), (0.0, 1.0))
    }

    /// `GeomFill_NSections(NC, NP)`; `GeomFillNSections::with_params(&sections, &params)?`.
    pub fn with_params(nc: &[Handle<dyn GeomCurve>], np: &[f64]) -> Result<Self, Failure> {
        Self::with_domain(nc, np, (0.0, 1.0), (0.0, 1.0))
    }

    /// `GeomFill_NSections(NC, NP, UF, UL)`;
    /// `GeomFillNSections::with_section_domain(&sections, &params, 0.0, 2.0)?`.
    pub fn with_section_domain(
        nc: &[Handle<dyn GeomCurve>],
        np: &[f64],
        uf: f64,
        ul: f64,
    ) -> Result<Self, Failure> {
        Self::with_domain(nc, np, (uf, ul), (0.0, 1.0))
    }

    /// `GeomFill_NSections(NC, NP, UF, UL, VF, VL)`;
    /// `GeomFillNSections::with_domain(&sections, &params, (0.0, 1.0), (0.0, 3.0))?`.
    pub fn with_domain(
        nc: &[Handle<dyn GeomCurve>],
        np: &[f64],
        u: (f64, f64),
        v: (f64, f64),
    ) -> Result<Self, Failure> {
        let law = Self::build(nc, np, u, v);
        law.compute_surface()?;
        Ok(law)
    }

    /// `GeomFill_NSections(NC, Trsfs, NP, UF, UL, VF, VL, Surf)`: the surface is a
    /// segment of the reference surface;
    /// `GeomFillNSections::with_reference(&sections, &trsfs, &params, (0.0, 1.0), (0.0, 1.0), &surf)?`.
    pub fn with_reference(
        nc: &[Handle<dyn GeomCurve>],
        trsfs: &[Transform3],
        np: &[f64],
        u: (f64, f64),
        v: (f64, f64),
        surf: &Handle<GeomBSplineSurface>,
    ) -> Result<Self, Failure> {
        let mut law = Self::build(nc, np, u, v);
        law.my_trsfs = trsfs.to_vec();
        law.surfaces.write().ref_surf = surf.clone();
        law.compute_surface()?;
        Ok(law)
    }

    /// `SetSurface(RefSurf)`: the reference surface of the next `ComputeSurface`;
    /// `law.set_surface(&surf)`.
    pub fn set_surface(&self, ref_surf: &Handle<GeomBSplineSurface>) {
        self.surfaces.write().ref_surf = ref_surf.clone();
    }

    pub(crate) fn surface(&self) -> Handle<GeomBSplineSurface> {
        self.surfaces.read().surface.clone()
    }

    /// `ComputeSurface()`: the surface skinning the sections (nothing below two
    /// sections), or the segment of the reference surface; its V degree is raised
    /// to 2 at least; `law.compute_surface()?`.
    pub fn compute_surface(&self) -> Result<(), Failure> {
        let ref_surf = self.surfaces.read().ref_surf.clone();
        let bs = if ref_surf.is_null() {
            match self.skinned_surface()? {
                Some(bs) => bs,
                None => return Ok(()),
            }
        } else {
            self.segmented_surface(&ref_surf)?
        };
        self.surfaces.write().surface = bs.clone();
        let s = live(&bs, "GeomFill_NSections::ComputeSurface", "mySurface")?;
        if s.v_degree() < 2 {
            s.increase_degree(s.u_degree(), 2)?;
        }
        Ok(())
    }

    /// Each section as a B-spline on `[UFirst, ULast]` (a B-spline section is
    /// reparameterised in place).
    fn section_generator(&self) -> Result<GeomFillSectionGenerator, Failure> {
        let mut section = GeomFillSectionGenerator::new();
        for curv in &self.my_sections {
            let mut curv_bs: Handle<GeomBSplineCurve> = curv.cast();
            if curv_bs.is_null() {
                curv_bs = curve_to_bspline_curve(curv, ParameterisationType::QuasiAngular)?;
            }
            let c = live(&curv_bs, "GeomFill_NSections::ComputeSurface", "curvBS")?;
            let mut bsk = c.knots();
            bsplclib::reparametrize(self.u_first, self.u_last, &mut bsk)?;
            c.set_knots(&bsk)?;
            section.add_curve(&curv_bs.cast())?;
        }
        if !self.my_params.is_empty() {
            let mut hpar: Array1<'static, f64> = one_based(self.my_params.len() as i32)?;
            hpar.as_mut_slice().copy_from_slice(&self.my_params);
            section.set_param(&Handle::new(HArray1::from_owned(hpar)))?;
        }
        section.perform(precision::p_confusion())?;
        Ok(section)
    }

    /// The skinning of the sections by `GeomFill_AppSurf` (degrees 2 to 6, C1,
    /// tolerance 1e-6), or `None` below two sections.
    fn skinned_surface(&self) -> Result<Option<Handle<GeomBSplineSurface>>, Failure> {
        if self.my_sections.len() <= 1 {
            return Ok(None);
        }
        let mut section = self.section_generator()?;
        let line = GeomFillLine::with_nb_points(self.my_sections.len() as i32);
        let my_pres3d = 1.0e-6;
        let known_p = !self.my_params.is_empty();
        let mut an_approx = GeomFillAppSurf::new(2, 6, my_pres3d, my_pres3d, 0, known_p);
        an_approx.set_continuity(Shape::C1);
        an_approx.perform(Some(&line), &mut section, true)?;
        let bs = GeomBSplineSurface::with_weights(
            an_approx.surf_poles()?,
            an_approx.surf_weights()?,
            (an_approx.surf_u_knots()?, an_approx.surf_u_mults()?),
            (an_approx.surf_v_knots()?, an_approx.surf_v_mults()?),
            (an_approx.u_degree()?, an_approx.v_degree()?),
            (section.is_periodic(), false),
        )?;
        Ok(Some(Handle::new(bs)))
    }

    /// A copy of the reference surface segmented to `[UFirst, ULast]` (snapped to
    /// the knots within PConfusion()) and to its whole V knot range.
    fn segmented_surface(
        &self,
        ref_surf: &Handle<GeomBSplineSurface>,
    ) -> Result<Handle<GeomBSplineSurface>, Failure> {
        let r = live(ref_surf, "GeomFill_NSections::ComputeSurface", "myRefSurf")?;
        let bs: Handle<GeomBSplineSurface> = r.copy()?.cast();
        let ui1 = snap_to_knots(r, self.u_first)?;
        let ui2 = snap_to_knots(r, self.u_last)?;
        let v0 = r.v_knot(r.first_v_knot_index()?)?;
        let v1 = r.v_knot(r.last_v_knot_index()?)?;
        let tol = precision::p_confusion();
        live(&bs, "GeomFill_NSections::ComputeSurface", "BS")?.check_and_segment(
            (ui1, ui2),
            (v0, v1),
            (tol, tol),
        )?;
        Ok(bs)
    }
}

/// `u` moved onto the knot of its interval within PConfusion() (the upper one
/// winning when both are).
fn snap_to_knots(r: &GeomBSplineSurface, mut u: f64) -> Result<f64, Failure> {
    let tol = precision::p_confusion();
    let (i1, i2) = r.locate_u(u, tol, false)?;
    if (u - r.u_knot(i1)?).abs() <= tol {
        u = r.u_knot(i1)?;
    }
    if (u - r.u_knot(i2)?).abs() <= tol {
        u = r.u_knot(i2)?;
    }
    Ok(u)
}
