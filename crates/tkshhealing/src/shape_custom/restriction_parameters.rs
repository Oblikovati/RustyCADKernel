// Rust translation of ModelingAlgorithms/TKShHealing/ShapeCustom/ShapeCustom_RestrictionParameters.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ShapeCustom_RestrictionParameters`: the flags of the B-spline restriction
//! (docs/shhealing-design.md section 7).
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{SharedState, StandardTransient};

/// The native fields, under their native names.
#[derive(Clone, Copy, Debug)]
struct RestrictionFields {
    g_max_degree: i32,
    g_max_seg: i32,
    conv_plane: bool,
    conv_conical_surf: bool,
    conv_spherical_surf: bool,
    conv_cylindrical_surf: bool,
    conv_toroidal_surf: bool,
    conv_bezier_surf: bool,
    conv_revol_surf: bool,
    conv_extr_surf: bool,
    conv_offset_surf: bool,
    segment_surface_mode: bool,
    conv_curve3d: bool,
    conv_offset_curv3d: bool,
    conv_curve2d: bool,
    conv_offset_curv2d: bool,
}

/// `ShapeCustom_RestrictionParameters`: which geometries the B-spline restriction
/// converts, with the global degree and segment limits. Each native accessor returns a
/// reference to its field; here it is a getter and a `set_` setter.
///
/// Quirk preserved: `ConvertToroidalSurf()` returns the cylindrical field and
/// `ConvertCylindricalSurf()` the toroidal one; both start false, so only the field
/// behind each name differs.
///
/// ```
/// use rustycad_tkshhealing::shape_custom::ShapeCustomRestrictionParameters;
/// let parameters = ShapeCustomRestrictionParameters::new();
/// assert_eq!(parameters.g_max_degree(), 15);
/// parameters.set_g_max_seg(20);
/// assert_eq!(parameters.g_max_seg(), 20);
/// ```
pub struct ShapeCustomRestrictionParameters {
    parent: StandardTransient,
    fields: SharedState<RestrictionFields>,
}
impl_transient!(
    ShapeCustomRestrictionParameters,
    "ShapeCustom_RestrictionParameters",
    StandardTransient,
    parent
);

impl Default for ShapeCustomRestrictionParameters {
    fn default() -> Self {
        Self::new()
    }
}

/// A getter and a setter over one field.
macro_rules! restriction_flag {
    ($(#[$doc:meta])* $get:ident, $set:ident, $field:ident, $kind:ty) => {
        $(#[$doc])*
        pub fn $get(&self) -> $kind {
            self.fields.read().$field
        }
        #[doc = concat!("Writes the field `", stringify!($get), "` reads; `parameters.", stringify!($set), "(value)`.")]
        pub fn $set(&self, value: $kind) {
            self.fields.write().$field = value;
        }
    };
}

impl ShapeCustomRestrictionParameters {
    /// `ShapeCustom_RestrictionParameters()`: degree 15, 10000 segments; planes, Bezier
    /// surfaces and the cones, spheres, cylinders and tori are kept, every other flag
    /// is set; `ShapeCustomRestrictionParameters::new()`.
    pub fn new() -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(RestrictionFields {
                g_max_degree: 15,
                g_max_seg: 10000,
                conv_plane: false,
                conv_conical_surf: false,
                conv_spherical_surf: false,
                conv_cylindrical_surf: false,
                conv_toroidal_surf: false,
                conv_bezier_surf: false,
                conv_revol_surf: true,
                conv_extr_surf: true,
                conv_offset_surf: true,
                segment_surface_mode: true,
                conv_curve3d: true,
                conv_offset_curv3d: true,
                conv_curve2d: true,
                conv_offset_curv2d: true,
            }),
        }
    }

    restriction_flag!(
        /// `GMaxDegree()`: the maximal degree; `parameters.g_max_degree()`.
        g_max_degree, set_g_max_degree, g_max_degree, i32
    );
    restriction_flag!(
        /// `GMaxSeg()`: the maximal number of segments; `parameters.g_max_seg()`.
        g_max_seg, set_g_max_seg, g_max_seg, i32
    );
    restriction_flag!(
        /// `ConvertPlane()`; `parameters.convert_plane()`.
        convert_plane, set_convert_plane, conv_plane, bool
    );
    restriction_flag!(
        /// `ConvertBezierSurf()`; `parameters.convert_bezier_surf()`.
        convert_bezier_surf, set_convert_bezier_surf, conv_bezier_surf, bool
    );
    restriction_flag!(
        /// `ConvertRevolutionSurf()`; `parameters.convert_revolution_surf()`.
        convert_revolution_surf, set_convert_revolution_surf, conv_revol_surf, bool
    );
    restriction_flag!(
        /// `ConvertExtrusionSurf()`; `parameters.convert_extrusion_surf()`.
        convert_extrusion_surf, set_convert_extrusion_surf, conv_extr_surf, bool
    );
    restriction_flag!(
        /// `ConvertOffsetSurf()`; `parameters.convert_offset_surf()`.
        convert_offset_surf, set_convert_offset_surf, conv_offset_surf, bool
    );
    restriction_flag!(
        /// `ConvertCylindricalSurf()`: natively a reference to `myConvToroidalSurf`;
        /// `parameters.convert_cylindrical_surf()`.
        convert_cylindrical_surf, set_convert_cylindrical_surf, conv_toroidal_surf, bool
    );
    restriction_flag!(
        /// `ConvertConicalSurf()`; `parameters.convert_conical_surf()`.
        convert_conical_surf, set_convert_conical_surf, conv_conical_surf, bool
    );
    restriction_flag!(
        /// `ConvertToroidalSurf()`: natively a reference to `myConvCylindricalSurf`;
        /// `parameters.convert_toroidal_surf()`.
        convert_toroidal_surf, set_convert_toroidal_surf, conv_cylindrical_surf, bool
    );
    restriction_flag!(
        /// `ConvertSphericalSurf()`; `parameters.convert_spherical_surf()`.
        convert_spherical_surf, set_convert_spherical_surf, conv_spherical_surf, bool
    );
    restriction_flag!(
        /// `SegmentSurfaceMode()`; `parameters.segment_surface_mode()`.
        segment_surface_mode, set_segment_surface_mode, segment_surface_mode, bool
    );
    restriction_flag!(
        /// `ConvertCurve3d()`; `parameters.convert_curve3d()`.
        convert_curve3d, set_convert_curve3d, conv_curve3d, bool
    );
    restriction_flag!(
        /// `ConvertOffsetCurv3d()`; `parameters.convert_offset_curv3d()`.
        convert_offset_curv3d, set_convert_offset_curv3d, conv_offset_curv3d, bool
    );
    restriction_flag!(
        /// `ConvertCurve2d()`; `parameters.convert_curve2d()`.
        convert_curve2d, set_convert_curve2d, conv_curve2d, bool
    );
    restriction_flag!(
        /// `ConvertOffsetCurv2d()`; `parameters.convert_offset_curv2d()`.
        convert_offset_curv2d, set_convert_offset_curv2d, conv_offset_curv2d, bool
    );
}
