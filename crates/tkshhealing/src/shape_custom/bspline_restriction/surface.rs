//! `NewSurface` and `ConvertSurface` of ShapeCustom_BSplineRestriction: the exact
//! conversions (planes, Beziers, extrusions) and the dispatch on wrapped surfaces.
use super::ShapeCustomBSplineRestriction;
use super::predicates::{Limits, is_convert_surface};
use crate::shape_construct::ShapeConstruct;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_tools::{NewSurface, uv_bounds};
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomBSplineSurface, GeomBezierSurface, GeomCurve, GeomOffsetCurve,
    GeomOffsetSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
    GeomSurfaceOfLinearExtrusion, GeomSurfaceOfRevolution, GeomSweptSurface,
};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Point3, Transform3};

/// The parameter box `[uf, ul] x [vf, vl]` of a surface conversion.
#[derive(Clone, Copy)]
pub struct ShapeCustomRestrictionBox {
    /// UF.
    pub uf: f64,
    /// UL.
    pub ul: f64,
    /// VF.
    pub vf: f64,
    /// VL.
    pub vl: f64,
}

impl ShapeCustomBSplineRestriction {
    pub(super) fn limits(&self) -> Limits {
        let state = self.state.read();
        Limits {
            degree: state.max_degree,
            segments: state.nb_max_seg,
            rational: state.rational,
        }
    }

    /// The `NewSurface` body: the face surface converted on its bounds (the face UV bounds
    /// in segment mode or in an infinite direction), with the messages MSG1 for a B-spline
    /// origin and MSG0 otherwise.
    pub(super) fn restricted_surface(
        &self,
        face: &Face,
        result: &mut NewSurface,
    ) -> Result<bool, Failure> {
        if !self.read().approx_surface {
            return Ok(false);
        }
        (result.rev_wires, result.rev_face) = (false, false);
        self.state.write().converted = false;
        let (surface, location) = BRepTool::surface_with_location(face)?;
        result.location = location;
        if surface.is_null() {
            return Ok(false);
        }
        let parameters = self.parameters()?;
        let is_of = !parameters.convert_offset_surf();
        let bounds = face_bounds(&surface, face, parameters.segment_surface_mode())?;
        let converted = self.convert_surface(&surface, &mut result.surface, bounds, is_of)?;
        result.tolerance = precision::confusion();
        if converted {
            let key: &[u8] = if was_bspline(&surface) {
                b"BSplineRestriction.NewSurface.MSG1"
            } else {
                b"BSplineRestriction.NewSurface.MSG0"
            };
            self.send(&face.clone().into_shape(), key);
        }
        Ok(converted)
    }

    /// `ConvertSurface(aSurface, S, UF, UL, VF, VL, IsOf)`: whether `S` received the
    /// conversion of `surface` on the box; with `is_of` an offset keeps its offset over the
    /// converted basis when that is not C0.
    /// Example: `restriction.convert_surface(&surface, &mut result, bounds, true)?`.
    pub fn convert_surface(
        &self,
        surface: &Handle<dyn GeomSurface>,
        result: &mut Handle<dyn GeomSurface>,
        bounds: ShapeCustomRestrictionBox,
        is_of: bool,
    ) -> Result<bool, Failure> {
        let parameters = self.parameters()?;
        if !is_convert_surface(surface, self.limits(), &parameters) {
            return Ok(false);
        }
        if !surface.cast::<GeomPlane>().is_null() && parameters.convert_plane() {
            *result = bilinear_plane(surface, bounds)?;
            return Ok(true);
        }
        let mut current = surface.clone();
        if let Some(outcome) = self.convert_revolution(&mut current, result, bounds, is_of)? {
            return Ok(outcome);
        }
        if let Some(outcome) = self.convert_extrusion(&current, result, bounds, is_of)? {
            return Ok(outcome);
        }
        if let Some(outcome) = self.convert_wrapped(&current, result, bounds, is_of)? {
            return Ok(outcome);
        }
        self.approximate_surface(surface, &current, result, bounds)
    }

    // A revolution with ConvertRevolutionSurf only gets its offset basis converted (and
    // goes on to the approximation); without it the basis curve is converted.
    fn convert_revolution(
        &self,
        current: &mut Handle<dyn GeomSurface>,
        result: &mut Handle<dyn GeomSurface>,
        bounds: ShapeCustomRestrictionBox,
        is_of: bool,
    ) -> Result<Option<bool>, Failure> {
        let revolution = current.cast::<GeomSurfaceOfRevolution>();
        let Some(revolution) = revolution.get() else {
            return Ok(None);
        };
        let basis = revolution.basis_curve();
        let range = (
            bounds.vf.max(basis.first_parameter()),
            bounds.vl.min(basis.last_parameter()),
        );
        let mut converted: Handle<dyn GeomCurve> = Handle::default();
        let mut tol = precision::confusion();
        if !self.parameters()?.convert_revolution_surf() {
            if !self.convert_curve(&basis, &mut converted, false, range, &mut tol, is_of)? {
                return Ok(Some(false));
            }
            *result =
                Handle::new(GeomSurfaceOfRevolution::new(&converted, revolution.axis())?).cast();
            return Ok(Some(true));
        }
        if !basis.cast::<GeomOffsetCurve>().is_null()
            && self.convert_curve(&basis, &mut converted, false, range, &mut tol, false)?
        {
            *current =
                Handle::new(GeomSurfaceOfRevolution::new(&converted, revolution.axis())?).cast();
        }
        Ok(None)
    }

    fn convert_extrusion(
        &self,
        current: &Handle<dyn GeomSurface>,
        result: &mut Handle<dyn GeomSurface>,
        bounds: ShapeCustomRestrictionBox,
        is_of: bool,
    ) -> Result<Option<bool>, Failure> {
        let extrusion = current.cast::<GeomSurfaceOfLinearExtrusion>();
        let Some(extrusion) = extrusion.get() else {
            return Ok(None);
        };
        let mut basis = extrusion.basis_curve();
        let mut converted: Handle<dyn GeomCurve> = Handle::default();
        let mut tol = precision::confusion();
        if !self.parameters()?.convert_extrusion_surf() {
            let range = (
                bounds.uf.max(basis.first_parameter()),
                bounds.ul.min(basis.last_parameter()),
            );
            if !self.convert_curve(&basis, &mut converted, false, range, &mut tol, is_of)? {
                return Ok(Some(false));
            }
            *result = Handle::new(GeomSurfaceOfLinearExtrusion::new(
                &converted,
                extrusion.direction(),
            )?)
            .cast();
            return Ok(Some(true));
        }
        let state = self.read();
        let continuity = current.continuity().min(Continuity::C2);
        basis = ShapeConstruct::convert_curve_to_bspline(
            &basis,
            bounds.uf,
            bounds.ul,
            tol,
            continuity,
            state.nb_max_seg,
            state.max_degree,
        )?
        .cast();
        let range = (
            bounds.uf.max(basis.first_parameter()),
            bounds.ul.min(basis.last_parameter()),
        );
        // The native result of this ConvertCurve is ignored.
        self.convert_curve(&basis, &mut converted, true, range, &mut tol, is_of)?;
        *result = swept_spline(&converted, extrusion, bounds)?;
        Ok(Some(true))
    }

    fn convert_wrapped(
        &self,
        current: &Handle<dyn GeomSurface>,
        result: &mut Handle<dyn GeomSurface>,
        bounds: ShapeCustomRestrictionBox,
        is_of: bool,
    ) -> Result<Option<bool>, Failure> {
        if let Some(trimmed) = current.cast::<GeomRectangularTrimmedSurface>().get() {
            let (u1, u2, v1, v2) = trimmed.bounds();
            let own = ShapeCustomRestrictionBox {
                uf: u1,
                ul: u2,
                vf: v1,
                vl: v2,
            };
            return Ok(Some(self.convert_surface(
                &trimmed.basis_surface(),
                result,
                own,
                is_of,
            )?));
        }
        let offset = current.cast::<GeomOffsetSurface>();
        if let Some(offset) = offset.get().filter(|_| is_of) {
            return Ok(Some(
                self.convert_offset_surface(current, offset, result, bounds)?,
            ));
        }
        let bezier = current.cast::<GeomBezierSurface>();
        if let Some(bezier) = bezier.get().filter(|_| self.convert_bezier()) {
            let spline: Handle<dyn GeomSurface> = Handle::new(bezier_as_bspline(bezier)?).cast();
            if !self.convert_surface(&spline, result, bounds, is_of)? {
                *result = spline;
            }
            return Ok(Some(true));
        }
        Ok(None)
    }

    fn convert_bezier(&self) -> bool {
        self.parameters().is_ok_and(|p| p.convert_bezier_surf())
    }

    fn convert_offset_surface(
        &self,
        current: &Handle<dyn GeomSurface>,
        offset: &GeomOffsetSurface,
        result: &mut Handle<dyn GeomSurface>,
        bounds: ShapeCustomRestrictionBox,
    ) -> Result<bool, Failure> {
        let mut basis: Handle<dyn GeomSurface> = Handle::default();
        if !self.convert_surface(&offset.basis_surface(), &mut basis, bounds, true)? {
            return Ok(false);
        }
        if basis.continuity() != Continuity::C0 {
            *result = Handle::new(GeomOffsetSurface::new(&basis, offset.offset(), false)?).cast();
            return Ok(true);
        }
        self.convert_surface(current, result, bounds, false)
    }
}

fn face_bounds(
    surface: &Handle<dyn GeomSurface>,
    face: &Face,
    segment_mode: bool,
) -> Result<ShapeCustomRestrictionBox, Failure> {
    let (mut uf, mut ul, mut vf, mut vl) = surface.bounds();
    let (umin, umax, vmin, vmax) = uv_bounds(face)?;
    if segment_mode || precision::is_infinite(uf) || precision::is_infinite(ul) {
        (uf, ul) = (umin, umax);
    }
    if segment_mode || precision::is_infinite(vf) || precision::is_infinite(vl) {
        (vf, vl) = (vmin, vmax);
    }
    Ok(ShapeCustomRestrictionBox { uf, ul, vf, vl })
}

fn was_bspline(surface: &Handle<dyn GeomSurface>) -> bool {
    if let Some(trimmed) = surface.cast::<GeomRectangularTrimmedSurface>().get() {
        return !trimmed
            .basis_surface()
            .cast::<GeomBSplineSurface>()
            .is_null();
    }
    !surface.cast::<GeomBSplineSurface>().is_null()
}

fn bilinear_plane(
    surface: &Handle<dyn GeomSurface>,
    b: ShapeCustomRestrictionBox,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let poles = [
        surface.value(b.uf, b.vf)?,
        surface.value(b.uf, b.vl)?,
        surface.value(b.ul, b.vf)?,
        surface.value(b.ul, b.vl)?,
    ];
    let poles = Array2::from_shared(&poles, 1, 2, 1, 2)
        .map_err(|e| e.into_failure("ShapeCustom_BSplineRestriction::ConvertSurface"))?;
    let (uk, vk, mults) = ([b.uf, b.ul], [b.vf, b.vl], [2, 2]);
    let (uk, vk, mults) = (
        Array1::from_shared(&uk, 1),
        Array1::from_shared(&vk, 1),
        Array1::from_shared(&mults, 1),
    );
    let spline =
        GeomBSplineSurface::new(&poles, (&uk, &mults), (&vk, &mults), (1, 1), (false, false))?;
    Ok(Handle::new(spline).cast())
}

fn bezier_as_bspline(bezier: &GeomBezierSurface) -> Result<GeomBSplineSurface, Failure> {
    let (ud, vd) = (bezier.u_degree(), bezier.v_degree());
    let poles = bezier.poles();
    let weights = bezier.weights_array(|w| w.clone());
    let knots = [0., 1.];
    let (um, vm) = ([ud + 1, ud + 1], [vd + 1, vd + 1]);
    let knots = Array1::from_shared(&knots, 1);
    let (um, vm) = (Array1::from_shared(&um, 1), Array1::from_shared(&vm, 1));
    GeomBSplineSurface::with_weights(
        &poles,
        &weights,
        (&knots, &um),
        (&knots, &vm),
        (ud, vd),
        (false, false),
    )
}

fn shift(from: Point3, to: Point3) -> Transform3 {
    let mut transform = Transform3::default();
    transform.set_translation_between(from, to);
    transform
}

/// The static `ConvertExtrusion`: the converted basis swept between the V bounds.
fn swept_spline(
    curve: &Handle<dyn GeomCurve>,
    extrusion: &GeomSurfaceOfLinearExtrusion,
    b: ShapeCustomRestrictionBox,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let spline = curve.cast::<GeomBSplineCurve>();
    // Natively the down-cast B-spline is dereferenced unchecked.
    let spline = spline.get().ok_or_else(|| {
        Failure::with_type("Standard_NullObject", "ShapeCustom_BSplineRestriction::ConvertExtrusion: the converted basis is not a Geom_BSplineCurve", "")
    })?;
    let origin = extrusion.value(b.uf, 0.)?;
    let first = shift(origin, extrusion.value(b.uf, b.vf)?);
    let last = shift(origin, extrusion.value(b.uf, b.vl)?);
    let n = spline.nb_poles();
    let mut poles = Vec::with_capacity(2 * n as usize);
    let mut weights = Vec::with_capacity(2 * n as usize);
    for j in 1..=n {
        let pole = spline.pole(j)?;
        poles.extend([pole.transformed(first), pole.transformed(last)]);
        let w = spline.weight(j)?;
        weights.extend([w, w]);
    }
    let label = "ShapeCustom_BSplineRestriction::ConvertExtrusion";
    let poles = Array2::from_shared(&poles, 1, n, 1, 2).map_err(|e| e.into_failure(label))?;
    let weights = Array2::from_shared(&weights, 1, n, 1, 2).map_err(|e| e.into_failure(label))?;
    let knots: Vec<f64> = (1..=spline.nb_knots())
        .map(|i| spline.knot(i))
        .collect::<Result<_, _>>()?;
    let mults: Vec<i32> = (1..=spline.nb_knots())
        .map(|i| spline.multiplicity(i))
        .collect::<Result<_, _>>()?;
    let (vk, vm) = ([b.vf, b.vl], [2, 2]);
    let surface = GeomBSplineSurface::with_weights(
        &poles,
        &weights,
        (
            &Array1::from_shared(&knots, 1),
            &Array1::from_shared(&mults, 1),
        ),
        (&Array1::from_shared(&vk, 1), &Array1::from_shared(&vm, 1)),
        (spline.degree(), 1),
        (spline.is_periodic(), false),
    )?;
    Ok(Handle::new(surface).cast())
}
