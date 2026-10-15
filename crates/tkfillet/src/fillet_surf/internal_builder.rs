// Rust translation of FilletSurf_InternalBuilder.hxx and the members of
// FilletSurf_InternalBuilder.cxx other than Add at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `FilletSurf_InternalBuilder`: a fillet builder that only computes the fillet
//! surfaces of one contour of constant radius (no corner, no solid); it overrides the
//! two-surface `PerformSurf` with a simulation walking followed by the approximation.
use super::StatusType;
use super::internal_add::compute_edge_parameter;
use crate::chfi_ds::{StripeHandle, SurfDataHandle, lock, shared};
use crate::chfi3d::{ChFi3dBuilderOps, ChFi3dFilBuilder, FilletShape, surf_data_at};
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomSurface, GeomTrimmedCurve};
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// The fillet builder of `FilletSurf_Builder` (`FilletSurf_InternalBuilder`).
///
/// ```
/// use rustycad_tkfillet::chfi3d::FilletShape;
/// use rustycad_tkfillet::fillet_surf::FilletSurfInternalBuilder;
/// use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
/// use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
/// let solid = BRepPrimAPIMakeBox::from_sizes(1., 1., 1.)?.shape()?.clone();
/// let mut b = FilletSurfInternalBuilder::new(&solid, FilletShape::Polynomial, 1e-2, 1e-4, 1e-5)?;
/// assert_eq!(b.add(&[], 0.1)?, 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct FilletSurfInternalBuilder {
    pub(crate) fil: ChFi3dFilBuilder,
}

/// The failure of a missing contour or surface data.
fn no_such(what: &str) -> Failure {
    Failure::with_message("Standard_NoSuchObject", what)
}

impl FilletSurfInternalBuilder {
    /// `FilletSurf_InternalBuilder(S, FShape, Ta, Tapp3d, Tapp2d)` (the native defaults
    /// are Polynomial, 1e-2, 1e-4 and 1e-5): the 3d and approximation tolerances
    /// `tapp3d`, the 2d ones `tapp2d`, and a C2 elementary spine.
    pub fn new(
        s: &Shape,
        fshape: FilletShape,
        ta: f64,
        tapp3d: f64,
        tapp2d: f64,
    ) -> Result<Self, Failure> {
        let mut fil = ChFi3dFilBuilder::new(s, fshape, ta)?;
        fil.base
            .set_params(ta, tapp3d, tapp2d, tapp3d, tapp2d, 1.0e-3);
        fil.base.set_continuity(Continuity::C2, ta);
        Ok(Self { fil })
    }

    /// The contour (the first stripe).
    fn stripe(&self) -> Result<StripeHandle, Failure> {
        self.fil
            .base
            .my_list_stripe
            .first()
            .cloned()
            .ok_or_else(|| no_such("FilletSurf_InternalBuilder: no contour"))
    }

    /// The contour prepared for `PerformSetOfKGen`: a new sequence of surface data, and
    /// the orientations and the side of the fillet.
    fn prepare(&mut self) -> Result<StripeHandle, Failure> {
        let stripe = self.stripe()?;
        *lock(&stripe).change_set_of_surf_data() = Some(shared(Vec::new()));
        let spine = lock(&stripe)
            .spine()
            .cloned()
            .ok_or_else(|| no_such("FilletSurf_InternalBuilder: no spine"))?;
        let (ref_or1, ref_or2, ref_choix) = self.fil.base.stripe_orientations(&spine)?;
        let mut s = lock(&stripe);
        s.set_orientation_on_face1(ref_or1);
        s.set_orientation_on_face2(ref_or2);
        s.set_choix(ref_choix);
        drop(s);
        Ok(stripe)
    }

    /// `Perform()`: the fillet surfaces walked along the elementary spine.
    pub fn perform(&mut self) -> Result<(), Failure> {
        let stripe = self.prepare()?;
        (self as &mut dyn ChFi3dBuilderOps).perform_set_of_kgen(&stripe, false)
    }

    /// `Simulate()`: only the sections of the fillet.
    pub fn simulate(&mut self) -> Result<(), Failure> {
        let stripe = self.prepare()?;
        (self as &mut dyn ChFi3dBuilderOps).perform_set_of_kgen(&stripe, true)
    }

    /// `Done()`.
    pub fn done(&self) -> bool {
        self.fil.base.done
    }

    /// `NbSurface()`: the number of surface data of the contour.
    pub fn nb_surface(&self) -> Result<i32, Failure> {
        let seq = lock(&self.stripe()?)
            .set_of_surf_data()
            .cloned()
            .ok_or_else(|| no_such("NbSurface"))?;
        Ok(lock(&seq).len() as i32)
    }

    /// The `index`-th surface data of the contour.
    fn sd(&self, index: i32) -> Result<SurfDataHandle, Failure> {
        surf_data_at(&self.stripe()?, index)
    }

    /// `SurfaceFillet(Index)`: the fillet surface of the `index`-th surface data.
    pub fn surface_fillet(&self, index: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
        let isurf = lock(&self.sd(index)?).surf();
        Ok(self.fil.base.ds.surface(isurf).surface().clone())
    }

    /// `TolApp3d(Index)`: the tolerance of the approximation of that surface.
    pub fn tol_app3d(&self, index: i32) -> Result<f64, Failure> {
        let isurf = lock(&self.sd(index)?).surf();
        Ok(self.fil.base.ds.surface(isurf).tolerance())
    }

    /// The support face of the side `on_s` of the `index`-th surface data.
    fn support_face(&self, index: i32, on_s: i32) -> Result<Face, Failure> {
        let ishape = lock(&self.sd(index)?).index(on_s);
        topods::face(self.fil.base.ds.shape(ishape, true))
    }

    /// `SupportFace1(Index)`.
    pub fn support_face1(&self, index: i32) -> Result<Face, Failure> {
        self.support_face(index, 1)
    }

    /// `SupportFace2(Index)`.
    pub fn support_face2(&self, index: i32) -> Result<Face, Failure> {
        self.support_face(index, 2)
    }

    /// The 3d curve of the fillet on its support of the side `on_s`.
    fn curve_on_face(&self, index: i32, on_s: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        let icurv = lock(&self.sd(index)?).interference(on_s).line_index();
        Ok(self.fil.base.ds.curve(icurv).curve().clone())
    }

    /// `CurveOnFace1(Index)`.
    pub fn curve_on_face1(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.curve_on_face(index, 1)
    }

    /// `CurveOnFace2(Index)`.
    pub fn curve_on_face2(&self, index: i32) -> Result<Handle<dyn GeomCurve>, Failure> {
        self.curve_on_face(index, 2)
    }

    /// A pcurve of the side `on_s`: on the support face, or on the fillet.
    fn pcurve(
        &self,
        index: i32,
        on_s: i32,
        on_fillet: bool,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let sd = self.sd(index)?;
        let g = lock(&sd);
        let fi = g.interference(on_s);
        Ok(if on_fillet {
            fi.pcurve_on_surf().clone()
        } else {
            fi.pcurve_on_face().clone()
        })
    }

    /// `PCurveOnFace1(Index)`.
    pub fn pcurve_on_face1(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        self.pcurve(index, 1, false)
    }

    /// `PCurve1OnFillet(Index)`.
    pub fn pcurve1_on_fillet(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        self.pcurve(index, 1, true)
    }

    /// `PCurveOnFace2(Index)`.
    pub fn pcurve_on_face2(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        self.pcurve(index, 2, false)
    }

    /// `PCurve2OnFillet(Index)`.
    pub fn pcurve2_on_fillet(&self, index: i32) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        self.pcurve(index, 2, true)
    }

    /// The parameter on the edge of an end of the fillet (0 when the normal plane of the
    /// guide misses the edge).
    fn end_parameter(&self, first: bool) -> Result<f64, Failure> {
        let sp = lock(&self.stripe()?)
            .spine()
            .cloned()
            .ok_or_else(|| no_such("no spine"))?;
        let index = if first { 1 } else { self.nb_surface()? };
        let sd = self.sd(index)?;
        let p = if first {
            lock(&sd).first_spine_param()
        } else {
            lock(&sd).last_spine_param()
        };
        let mut ind = if first { 1 } else { lock(&sp).nb_edges() };
        if lock(&sp).is_periodic() {
            ind = lock(&sp).index_at(p, true)?;
        }
        Ok(compute_edge_parameter(&sp, ind, p, self.fil.base.tolapp3d)?.unwrap_or(0.0))
    }

    /// `FirstParameter()`: the parameter of the start of the fillet on the first edge.
    pub fn first_parameter(&self) -> Result<f64, Failure> {
        self.end_parameter(true)
    }

    /// `LastParameter()`: the parameter of the end of the fillet on the last edge.
    pub fn last_parameter(&self) -> Result<f64, Failure> {
        self.end_parameter(false)
    }

    /// The status of an end section from whether its two ends are on edges.
    fn section_status(&self, first: bool) -> Result<StatusType, Failure> {
        let index = if first { 1 } else { self.nb_surface()? };
        let sd = self.sd(index)?;
        let g = lock(&sd);
        let (on1, on2) = (
            g.vertex(first, 1).is_on_arc(),
            g.vertex(first, 2).is_on_arc(),
        );
        Ok(match (on1, on2) {
            (true, true) => StatusType::TwoExtremityOnEdge,
            (false, false) => StatusType::NoExtremityOnEdge,
            _ => StatusType::OneExtremityOnEdge,
        })
    }

    /// `StartSectionStatus()`.
    pub fn start_section_status(&self) -> Result<StatusType, Failure> {
        self.section_status(true)
    }

    /// `EndSectionStatus()`.
    pub fn end_section_status(&self) -> Result<StatusType, Failure> {
        self.section_status(false)
    }

    /// `NbSection(IndexSurf)`: the number of sections of the simulation of that surface.
    pub fn nb_section(&self, index_surf: i32) -> Result<i32, Failure> {
        let sect = self.fil.sect(1, index_surf)?;
        let arr = sect
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "NbSection"))?;
        Ok(arr.array1().len() as i32)
    }

    /// `Section(IndexSurf, IndexSec)`: the arc of circle of that section.
    pub fn section(
        &self,
        index_surf: i32,
        index_sec: i32,
    ) -> Result<Handle<GeomTrimmedCurve>, Failure> {
        let sect = self.fil.sect(1, index_surf)?;
        let arr = sect
            .get()
            .ok_or_else(|| Failure::with_message("Standard_NullObject", "Section"))?;
        let (c, deb, fin) = arr
            .array1()
            .value(index_sec)
            .map_err(|_| Failure::out_of_range("Section"))?
            .get_circle();
        let gc: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(c)).cast();
        Ok(Handle::new(GeomTrimmedCurve::new(
            &gc, deb, fin, true, true,
        )?))
    }
}
