// Rust translation of ModelingData/TKGeomBase/GeomConvert/GeomConvert_SurfToAnaSurf.hxx/.cxx
// (constructors, accessors, ConvertToAnalytical(InitialToler), CheckVTrimForRevSurf,
// IsSame, IsCanonical) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomConvert_SurfToAnaSurf`: the recognition of a plane, a cylinder, a cone, a
//! sphere or a torus in an arbitrary surface, within a tolerance.
use super::ConvType;
use crate::extrema::ExtElC;
use crate::geom_lib::live;
use crate::int_ana::{QuadQuadGeo, ResultType};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{
    GeomConicalSurface, GeomCylindricalSurface, GeomElementarySurface, GeomLine, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfRevolution, GeomSweptSurface,
    GeomToroidalSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::Line3;

/// `GeomConvert_SurfToAnaSurf`: converts a surface into the analytic surface
/// (plane, cylinder, cone, sphere, torus) chosen by the conversion type among the
/// candidates within the tolerance.
///
/// `let mut conv = SurfToAnaSurf::with_surface(&surface);`
#[derive(Clone)]
pub struct SurfToAnaSurf {
    pub(super) my_surf: Handle<dyn GeomSurface>,
    pub(super) my_gap: f64,
    pub(super) my_conv_type: ConvType,
    pub(super) my_target: SurfaceType,
}

impl Default for SurfToAnaSurf {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfToAnaSurf {
    /// The empty converter: gap -1, Simplest, target plane; `SurfToAnaSurf::new()`.
    pub fn new() -> Self {
        Self {
            my_surf: Handle::default(),
            my_gap: -1.0,
            my_conv_type: ConvType::Simplest,
            my_target: SurfaceType::Plane,
        }
    }

    /// The converter of `surface`; `SurfToAnaSurf::with_surface(&surface)`.
    pub fn with_surface(surface: &Handle<dyn GeomSurface>) -> Self {
        let mut conv = Self::new();
        conv.init(surface);
        conv
    }

    /// `Init(S)`: a new surface (the gap is kept); `conv.init(&surface)`.
    pub fn init(&mut self, surface: &Handle<dyn GeomSurface>) {
        self.my_surf = surface.clone();
    }

    /// `SetConvType(theConvType = GeomConvert_Simplest)`;
    /// `conv.set_conv_type(ConvType::MinGap)`.
    pub fn set_conv_type(&mut self, conv_type: ConvType) {
        self.my_conv_type = conv_type;
    }

    /// `SetTarget(theSurfType = GeomAbs_Plane)`: the surface type of
    /// ConvType::Target; `conv.set_target(SurfaceType::Cylinder)`.
    pub fn set_target(&mut self, target: SurfaceType) {
        self.my_target = target;
    }

    /// `Gap()`: the deviation of the last conversion (-1 before any);
    /// `conv.gap()`.
    pub fn gap(&self) -> f64 {
        self.my_gap
    }

    /// `ConvertToAnalytical(InitialToler)`: the conversion on the surface bounds,
    /// an infinite pair of bounds replaced by [-1, 1] (the V range of a surface of
    /// revolution of a line cut at the axis); null on failure. A null surface is
    /// Standard_NullObject; `conv.convert_to_analytical(1e-7)?`.
    pub fn convert_to_analytical(
        &mut self,
        initial_toler: f64,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        let surface: &dyn GeomSurface = live(
            &self.my_surf,
            "GeomConvert_SurfToAnaSurf::ConvertToAnalytical",
        )?;
        let (mut u1, mut u2, mut v1, mut v2) = surface.bounds();
        if precision::is_infinite(u1) && precision::is_infinite(u2) {
            (u1, u2) = (-1.0, 1.0);
        }
        if precision::is_infinite(v1) && precision::is_infinite(v2) {
            (v1, v2) = (-1.0, 1.0);
            if let Some(revolution) = self.my_surf.cast::<GeomSurfaceOfRevolution>().get() {
                (v1, v2) = check_v_trim_for_rev_surf(revolution, (v1, v2))?;
            }
        }
        self.convert_to_analytical_in(initial_toler, (u1, u2), (v1, v2))
    }

    /// `IsSame(S1, S2, tol)`: true when two elementary surfaces of the same type
    /// coincide within `tol` (IntAna_QuadQuadGeo gives IntAna_Same); a null surface
    /// is Standard_NullObject; `SurfToAnaSurf::is_same(&a, &b, 1e-7)?`.
    pub fn is_same(
        s1: &Handle<dyn GeomSurface>,
        s2: &Handle<dyn GeomSurface>,
        tol: f64,
    ) -> Result<bool, Failure> {
        // Only elementary surfaces are processed.
        live(s1, "GeomConvert_SurfToAnaSurf::IsSame")?;
        if s1.cast::<dyn GeomElementarySurface>().is_null() {
            return Ok(false);
        }
        live(s2, "GeomConvert_SurfToAnaSurf::IsSame")?;
        if s2.cast::<dyn GeomElementarySurface>().is_null() {
            return Ok(false);
        }
        let adaptor1 = GeomAdaptorSurface::with_surface(s1)?;
        let adaptor2 = GeomAdaptorSurface::with_surface(s2)?;
        let st1: SurfaceType = adaptor1.get_type()?;
        if st1 != adaptor2.get_type()? {
            return Ok(false);
        }
        let mut interii = QuadQuadGeo::new();
        match st1 {
            SurfaceType::Plane => {
                interii.perform_planes(adaptor1.plane()?, adaptor2.plane()?, tol, tol)?
            }
            SurfaceType::Cylinder => {
                interii.perform_cylinders(adaptor1.cylinder()?, adaptor2.cylinder()?, tol)?
            }
            SurfaceType::Cone => interii.perform_cones(adaptor1.cone()?, adaptor2.cone()?, tol)?,
            SurfaceType::Sphere => {
                interii.perform_spheres(adaptor1.sphere()?, adaptor2.sphere()?, tol)?
            }
            SurfaceType::Torus => {
                interii.perform_tori(adaptor1.torus()?, adaptor2.torus()?, tol)?
            }
            _ => {}
        }
        if !interii.is_done() {
            return Ok(false);
        }
        Ok(interii.type_inter()? == ResultType::Same)
    }

    /// `IsCanonical(S)`: true for a plane, a cylinder, a cone, a sphere or a torus
    /// (false for a null surface); `SurfToAnaSurf::is_canonical(&surface)`.
    pub fn is_canonical(surface: &Handle<dyn GeomSurface>) -> bool {
        if surface.is_null() {
            return false;
        }
        surface.cast::<GeomPlane>().is_some()
            || surface.cast::<GeomCylindricalSurface>().is_some()
            || surface.cast::<GeomConicalSurface>().is_some()
            || surface.cast::<GeomSphericalSurface>().is_some()
            || surface.cast::<GeomToroidalSurface>().is_some()
    }
}

// `CheckVTrimForRevSurf(aRevSurf, V1, V2)`: for the revolution of a line crossing
// the axis inside (V1, V2), the longer of the two parts, so that the result is not
// a two-part cone.
fn check_v_trim_for_rev_surf(
    rev_surf: &GeomSurfaceOfRevolution,
    (v1, v2): (f64, f64),
) -> Result<(f64, f64), Failure> {
    let basis = rev_surf.basis_curve();
    let line_handle: Handle<GeomLine> = basis.cast::<GeomLine>();
    let Some(line) = line_handle.get() else {
        return Ok((v1, v2));
    };
    let axis_line = Line3::from_axis(rev_surf.axis());
    let ext_ll = ExtElC::lines(line.lin(), axis_line, precision::angular());
    if !ext_ll.is_done() || ext_ll.is_parallel()? {
        return Ok((v1, v2));
    }
    let nb_ext: i32 = ext_ll.nb_ext()?;
    let mut imin: i32 = 0;
    for i in 1..=nb_ext {
        if ext_ll.square_distance(i)? < precision::square_confusion() {
            imin = i;
            break;
        }
    }
    if imin == 0 {
        return Ok((v1, v2));
    }
    let v_ext: f64 = ext_ll.points(imin)?.0.parameter();
    if v_ext <= v1 || v_ext >= v2 {
        return Ok((v1, v2));
    }
    if v_ext - v1 > v2 - v_ext {
        return Ok((v1, v_ext));
    }
    Ok((v_ext, v2))
}
