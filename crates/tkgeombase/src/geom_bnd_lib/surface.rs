// Rust translation of GeomBndLib_Surface at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomBndLib_Surface`: detect the kind of a surface (adaptor or Geom handle) and
//! delegate to its evaluator.
use super::cylinder::{Cone, Cylinder};
use super::extrusion::SurfaceOfExtrusion;
use super::offset_surface::OffsetSurface;
use super::other_surface::OtherSurface;
use super::plane::Plane;
use super::revolution::SurfaceOfRevolution;
use super::sphere::{Sphere, Torus};
use super::spline_surface::{BSplineSurface, BezierSurface};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, exceptions};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierSurface, GeomConicalSurface, GeomCylindricalSurface,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSphericalSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorSurface, GeomAdaptorTransformedSurface};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::SurfaceType;

// The `std::variant` of evaluators; `Other` borrows the dispatcher's adaptor at call time.
enum Evaluator {
    Plane(Plane),
    Cylinder(Cylinder),
    Cone(Cone),
    Sphere(Sphere),
    Torus(Torus),
    Bezier(BezierSurface),
    BSpline(BSplineSurface),
    Revolution(SurfaceOfRevolution),
    Extrusion(SurfaceOfExtrusion),
    Offset(OffsetSurface),
    Other,
}

/// `GeomBndLib_Surface`: the box of any surface through the evaluator of its kind. An
/// adaptor or a rectangular trimmed surface supplies the default patch of `Box(Tol)`.
pub struct Surface<'a> {
    adaptor_ref: Option<&'a dyn Adaptor3dSurface>,
    adaptor_owned: Option<GeomAdaptorSurface>,
    evaluator: Evaluator,
    surface_type: SurfaceType,
}

// Run `$body` on the evaluator bound to `$eval` (an OtherSurface on the adaptor for the
// `Other` kind).
macro_rules! dispatch {
    ($surface:expr, $eval:ident => $body:expr) => {
        match &$surface.evaluator {
            Evaluator::Plane($eval) => $body,
            Evaluator::Cylinder($eval) => $body,
            Evaluator::Cone($eval) => $body,
            Evaluator::Sphere($eval) => $body,
            Evaluator::Torus($eval) => $body,
            Evaluator::Bezier($eval) => $body,
            Evaluator::BSpline($eval) => $body,
            Evaluator::Revolution($eval) => $body,
            Evaluator::Extrusion($eval) => $body,
            Evaluator::Offset($eval) => $body,
            Evaluator::Other => {
                let $eval = &OtherSurface::new($surface.required_adaptor()?);
                $body
            }
        }
    };
}

// `unwrapSurface`: the basis under any number of rectangular trimmed wrappers.
fn unwrap_surface(surface: Handle<dyn GeomSurface>) -> Handle<dyn GeomSurface> {
    let trimmed: Handle<GeomRectangularTrimmedSurface> = surface.cast();
    match trimmed.get() {
        Some(wrapper) => unwrap_surface(wrapper.basis_surface()),
        None => surface,
    }
}

// `initFromSurfaceType`: the evaluator of `kind` on `occ::down_cast<T>(surface)` (null
// when the kinds disagree, as natively), or None for the kinds without an evaluator.
fn from_type(kind: SurfaceType, s: &Handle<dyn GeomSurface>) -> Option<Evaluator> {
    Some(match kind {
        SurfaceType::Plane => Evaluator::Plane(Plane::new(s.cast())),
        SurfaceType::Cylinder => Evaluator::Cylinder(Cylinder::new(s.cast())),
        SurfaceType::Cone => Evaluator::Cone(Cone::new(s.cast())),
        SurfaceType::Sphere => Evaluator::Sphere(Sphere::new(s.cast())),
        SurfaceType::Torus => Evaluator::Torus(Torus::new(s.cast())),
        SurfaceType::BezierSurface => Evaluator::Bezier(BezierSurface::new(s.cast())),
        SurfaceType::BSplineSurface => Evaluator::BSpline(BSplineSurface::new(s.cast())),
        SurfaceType::SurfaceOfRevolution => {
            Evaluator::Revolution(SurfaceOfRevolution::new(s.cast()))
        }
        SurfaceType::SurfaceOfExtrusion => Evaluator::Extrusion(SurfaceOfExtrusion::new(s.cast())),
        SurfaceType::OffsetSurface => Evaluator::Offset(OffsetSurface::new(s.cast())),
        _ => return None,
    })
}

// `initFromSurface`: the exact dynamic type (`DynamicType() == STANDARD_TYPE(...)`,
// subclasses do not match) selects the kind.
fn exact_kind(surface: &Handle<dyn GeomSurface>) -> Option<SurfaceType> {
    fn is<T: RuntimeTyped>(surface: &Handle<dyn GeomSurface>) -> bool {
        surface.is_instance(T::static_type())
    }
    let kinds: [(bool, SurfaceType); 10] = [
        (is::<GeomPlane>(surface), SurfaceType::Plane),
        (is::<GeomCylindricalSurface>(surface), SurfaceType::Cylinder),
        (is::<GeomConicalSurface>(surface), SurfaceType::Cone),
        (is::<GeomSphericalSurface>(surface), SurfaceType::Sphere),
        (is::<GeomToroidalSurface>(surface), SurfaceType::Torus),
        (is::<GeomBezierSurface>(surface), SurfaceType::BezierSurface),
        (
            is::<GeomBSplineSurface>(surface),
            SurfaceType::BSplineSurface,
        ),
        (
            is::<GeomSurfaceOfRevolution>(surface),
            SurfaceType::SurfaceOfRevolution,
        ),
        (
            is::<GeomSurfaceOfLinearExtrusion>(surface),
            SurfaceType::SurfaceOfExtrusion,
        ),
        (is::<GeomOffsetSurface>(surface), SurfaceType::OffsetSurface),
    ];
    kinds
        .into_iter()
        .find(|(matched, _)| *matched)
        .map(|(_, kind)| kind)
}

// The Geom surface behind an adaptor: a GeomAdaptor_Surface's surface or a
// GeomAdaptor_TransformedSurface's transformed surface, else null.
fn adaptor_geometry(surface: &dyn Adaptor3dSurface) -> Result<Handle<dyn GeomSurface>, Failure> {
    if let Some(geom_adaptor) = GeomAdaptorSurface::project_from(surface) {
        return Ok(geom_adaptor.surface());
    }
    if let Some(transformed) = GeomAdaptorTransformedSurface::project_from(surface) {
        return transformed.geom_surface_transformed();
    }
    Ok(Handle::default())
}

// The adaptor constructor's switch: elementary kinds rebuilt from the adaptor's gp
// value, spline kinds from its handles, every other kind an OtherSurface.
fn from_adaptor_value(
    surface: &dyn Adaptor3dSurface,
    kind: SurfaceType,
) -> Result<Option<Evaluator>, Failure> {
    Ok(Some(match kind {
        SurfaceType::Plane => Evaluator::Plane(Plane::new(Handle::new(GeomPlane::from_pln(
            surface.plane()?,
        )))),
        SurfaceType::Cylinder => Evaluator::Cylinder(Cylinder::new(Handle::new(
            GeomCylindricalSurface::from_cylinder(surface.cylinder()?),
        ))),
        SurfaceType::Cone => Evaluator::Cone(Cone::new(Handle::new(
            GeomConicalSurface::from_cone(surface.cone()?),
        ))),
        SurfaceType::Sphere => Evaluator::Sphere(Sphere::new(Handle::new(
            GeomSphericalSurface::from_sphere(surface.sphere()?),
        ))),
        SurfaceType::Torus => Evaluator::Torus(Torus::new(Handle::new(
            GeomToroidalSurface::from_torus(surface.torus()?),
        ))),
        SurfaceType::BezierSurface => Evaluator::Bezier(BezierSurface::new(surface.bezier()?)),
        SurfaceType::BSplineSurface => Evaluator::BSpline(BSplineSurface::new(surface.bspline()?)),
        _ => return Ok(None),
    }))
}

impl<'a> Surface<'a> {
    /// `GeomBndLib_Surface(const Adaptor3d_Surface&)`: the Geom surface behind a
    /// GeomAdaptor_Surface (or the transformed surface of a
    /// GeomAdaptor_TransformedSurface), without trimming wrappers, selects the evaluator
    /// of the adaptor's `GetType`; otherwise the adaptor's gp value or spline handle;
    /// every other kind uses OtherSurface on the adaptor; `Surface::from_adaptor(&adaptor)?`.
    pub fn from_adaptor(surface: &'a dyn Adaptor3dSurface) -> Result<Self, Failure> {
        let kind: SurfaceType = surface.get_type()?;
        let base: Handle<dyn GeomSurface> = unwrap_surface(adaptor_geometry(surface)?);
        let found: Option<Evaluator> = if base.is_null() {
            None
        } else {
            from_type(kind, &base)
        };
        let (surface_type, evaluator) = match found {
            Some(evaluator) => (kind, evaluator),
            None => match from_adaptor_value(surface, kind)? {
                Some(evaluator) => (kind, evaluator),
                None => (SurfaceType::OtherSurface, Evaluator::Other),
            },
        };
        Ok(Self {
            adaptor_ref: Some(surface),
            adaptor_owned: None,
            evaluator,
            surface_type,
        })
    }
    /// `GetType()`: the detected kind; `surface.get_type()`.
    pub fn get_type(&self) -> SurfaceType {
        self.surface_type
    }
    fn adaptor(&self) -> Option<&dyn Adaptor3dSurface> {
        match (self.adaptor_ref, &self.adaptor_owned) {
            (Some(adaptor), _) => Some(adaptor),
            (None, Some(owned)) => Some(owned),
            (None, None) => None,
        }
    }
    fn required_adaptor(&self) -> Result<&dyn Adaptor3dSurface, Failure> {
        self.adaptor().ok_or_else(|| {
            exceptions::null_object("GeomBndLib_Surface: the OtherSurface evaluator has no adaptor")
        })
    }
    // `[FirstU, LastU, FirstV, LastV]` of the adaptor.
    fn adaptor_patch(adaptor: &dyn Adaptor3dSurface) -> Result<[f64; 4], Failure> {
        Ok([
            adaptor.first_u_parameter()?,
            adaptor.last_u_parameter()?,
            adaptor.first_v_parameter()?,
            adaptor.last_v_parameter()?,
        ])
    }
    /// `Box(Tol)`: the adaptor bounds when there is an adaptor, else the evaluator's
    /// natural bounds; `surface.bounding_box(0.0)?`.
    pub fn bounding_box(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        if let Some(adaptor) = self.adaptor() {
            let [u1, u2, v1, v2] = Self::adaptor_patch(adaptor)?;
            return self.bounding_box_range(u1, u2, v1, v2, tolerance);
        }
        dispatch!(self, eval => eval.bounding_box(tolerance))
    }
    /// `Box(UMin, UMax, VMin, VMax, Tol)`; `surface.bounding_box_range(0.0, 1.0, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        dispatch!(self, eval => eval.bounding_box_range(u_min, u_max, v_min, v_max, tolerance))
    }
    /// `BoxOptimal(Tol)`: as `Box(Tol)` with the optimal boxes; `surface.bounding_box_optimal(0.0)?`.
    pub fn bounding_box_optimal(&self, tolerance: f64) -> Result<BoundingBox3, Failure> {
        if let Some(adaptor) = self.adaptor() {
            let [u1, u2, v1, v2] = Self::adaptor_patch(adaptor)?;
            return self.bounding_box_optimal_range(u1, u2, v1, v2, tolerance);
        }
        dispatch!(self, eval => eval.bounding_box_optimal(tolerance))
    }
    /// `BoxOptimal(UMin, UMax, VMin, VMax, Tol)`;
    /// `surface.bounding_box_optimal_range(0.0, 1.0, 0.0, 1.0, 0.0)?`.
    pub fn bounding_box_optimal_range(
        &self,
        u_min: f64,
        u_max: f64,
        v_min: f64,
        v_max: f64,
        tolerance: f64,
    ) -> Result<BoundingBox3, Failure> {
        dispatch!(self, eval => eval.bounding_box_optimal_range(u_min, u_max, v_min, v_max, tolerance))
    }
    /// `Add(Tol, B)`: `B.Add(Box(Tol))`; `surface.add(0.0, &mut bounds)?`.
    pub fn add(&self, tolerance: f64, bounds: &mut BoundingBox3) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box(tolerance)?);
        Ok(())
    }
    /// `Add(UMin, UMax, VMin, VMax, Tol, B)`; `surface.add_range([0.0, 1.0, 0.0, 1.0], 0.0, &mut bounds)?`.
    pub fn add_range(
        &self,
        [u_min, u_max, v_min, v_max]: [f64; 4],
        tolerance: f64,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box_range(u_min, u_max, v_min, v_max, tolerance)?);
        Ok(())
    }
    /// `AddOptimal(Tol, B)`; `surface.add_optimal(0.0, &mut bounds)?`.
    pub fn add_optimal(&self, tolerance: f64, bounds: &mut BoundingBox3) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box_optimal(tolerance)?);
        Ok(())
    }
    /// `AddOptimal(UMin, UMax, VMin, VMax, Tol, B)`;
    /// `surface.add_optimal_range([0.0, 1.0, 0.0, 1.0], 0.0, &mut bounds)?`.
    pub fn add_optimal_range(
        &self,
        [u_min, u_max, v_min, v_max]: [f64; 4],
        tolerance: f64,
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        bounds.add_box(&self.bounding_box_optimal_range(u_min, u_max, v_min, v_max, tolerance)?);
        Ok(())
    }
}

impl Surface<'static> {
    /// `GeomBndLib_Surface(const occ::handle<Geom_Surface>&)`: the exact dynamic type
    /// selects the evaluator; a rectangular trimmed surface uses its basis (one level) with
    /// a GeomAdaptor_Surface on the trimmed surface for the patch of `Box(Tol)`; any
    /// other surface uses OtherSurface with the type OtherSurface;
    /// `Surface::from_surface(&handle)?`.
    pub fn from_surface(surface: &Handle<dyn GeomSurface>) -> Result<Self, Failure> {
        let trimmed: Handle<GeomRectangularTrimmedSurface> = surface.cast();
        if let Some(trimmed) = trimmed.get() {
            let owned: GeomAdaptorSurface = GeomAdaptorSurface::with_surface(surface)?;
            let basis: Handle<dyn GeomSurface> = trimmed.basis_surface();
            let found = exact_kind(&basis).and_then(|kind| Some((kind, from_type(kind, &basis)?)));
            let (surface_type, evaluator) =
                found.unwrap_or((SurfaceType::OtherSurface, Evaluator::Other));
            return Ok(Self {
                adaptor_ref: None,
                adaptor_owned: Some(owned),
                evaluator,
                surface_type,
            });
        }
        if let Some(kind) = exact_kind(surface)
            && let Some(evaluator) = from_type(kind, surface)
        {
            return Ok(Self {
                adaptor_ref: None,
                adaptor_owned: None,
                evaluator,
                surface_type: kind,
            });
        }
        let owned: GeomAdaptorSurface = GeomAdaptorSurface::with_surface(surface)?;
        Ok(Self {
            adaptor_ref: None,
            adaptor_owned: Some(owned),
            evaluator: Evaluator::Other,
            surface_type: SurfaceType::OtherSurface,
        })
    }
}
