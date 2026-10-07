//! The cases of the native `RawMaterial` constructor (`Graphic3d_MaterialAspect.cxx`) as
//! data rows, in `Graphic3d_NameOfMaterial` order; material_table.rs builds them.
/// How a material case builds its BSDF from `CreateDiffuse(0)`.
#[derive(Clone, Copy)]
pub(super) enum BsdfSpec {
    /// `CreateDiffuse(0)` unchanged.
    Black,
    /// `Kd` and, when given, `Ks` set; then a dielectric 1.5 base and `Normalize()` on request.
    Layers {
        kd: [f32; 3],
        ks: Option<[f32; 4]>,
        normalize: bool,
        dielectric: bool,
    },
    /// `Kd = 0`, `Ks = (0.5, 0.5, 0.5, 0)`, `Le` and a dielectric 1.5 base (glowing).
    Glow([f32; 3]),
    /// `CreateMetallic(0.985, Schlick(color), 0.045)`.
    MetalSchlick([f32; 3]),
    /// `CreateMetallic(0.985, Conductor(n, k), 0.045)`.
    MetalConductor(f32, f32),
    /// `CreateMetallic(0.985, Conductor(n3, k3), 0.045)`.
    MetalConductorRgb([f32; 3], [f32; 3]),
    /// `CreateGlass(1, absorption, 0.05, RefractionIndex)`.
    Glass([f32; 3]),
    /// `Kd = 0.1`, `Kt = 0.9`, constant Fresnel 0.
    Transparent,
}

/// One `case` of the native constructor; unset members keep the member initialisers.
#[derive(Clone, Copy)]
pub(super) struct MaterialSpec {
    pub(super) physic: bool,
    pub(super) shininess: f32,
    pub(super) colors: [[f32; 3]; 4],
    pub(super) bsdf: BsdfSpec,
    /// `(RefractionIndex, TransparencyCoef)` of the transparent materials.
    pub(super) optics: Option<(f32, f32)>,
    /// `(AmbientCoef, DiffuseCoef)` when not `(0.25, 1)`.
    pub(super) coefs: Option<(f32, f32)>,
}

const fn spec(physic: bool, shininess: f32, colors: [[f32; 3]; 4], bsdf: BsdfSpec) -> MaterialSpec {
    MaterialSpec {
        physic,
        shininess,
        colors,
        bsdf,
        optics: None,
        coefs: None,
    }
}
const fn layers(kd: [f32; 3], ks: [f32; 4], normalize: bool, dielectric: bool) -> BsdfSpec {
    BsdfSpec::Layers {
        kd,
        ks: Some(ks),
        normalize,
        dielectric,
    }
}
/// A physical glass: refraction index, transparency 0.8, glass BSDF.
const fn glass(
    shininess: f32,
    colors: [[f32; 3]; 4],
    absorption: [f32; 3],
    index: f32,
) -> MaterialSpec {
    MaterialSpec {
        optics: Some((index, 0.80)),
        ..spec(true, shininess, colors, BsdfSpec::Glass(absorption))
    }
}
const fn gray(value: f32) -> [f32; 3] {
    [value, value, value]
}
pub(super) const BLACK: [f32; 3] = [0.0, 0.0, 0.0];

/// The cases of `RawMaterial::RawMaterial`, in `Graphic3d_NameOfMaterial` order.
pub(super) const MATERIAL_SPECS: [MaterialSpec; 26] = {
    use BsdfSpec::{Black, Glow, MetalConductor, MetalConductorRgb, MetalSchlick};
    [
        // Brass
        spec(
            true,
            0.65,
            [
                [0.088428, 0.041081, 0.002090],
                [0.570482, 0.283555, 0.012335],
                [0.992, 0.941, 0.808],
                BLACK,
            ],
            MetalSchlick([0.58, 0.42, 0.20]),
        ),
        // Bronze
        spec(
            true,
            0.65,
            [
                [0.037301, 0.014931, 0.004305],
                [0.468185, 0.153344, 0.027491],
                [0.590, 0.408, 0.250],
                BLACK,
            ],
            MetalSchlick([0.65, 0.35, 0.15]),
        ),
        // Copper
        spec(
            true,
            0.65,
            [
                [0.030370, 0.006451, 0.001780],
                [0.323236, 0.059254, 0.007584],
                [0.950, 0.640, 0.540],
                BLACK,
            ],
            MetalSchlick([0.955008, 0.637427, 0.538163]),
        ),
        // Gold
        spec(
            true,
            0.80,
            [
                [0.073239, 0.043234, 0.009264],
                [0.525643, 0.295700, 0.010023],
                [1.000, 0.710, 0.290],
                BLACK,
            ],
            MetalSchlick([1.000000, 0.765557, 0.336057]),
        ),
        // Pewter
        spec(
            true,
            0.50,
            [
                [0.010979, 0.004795, 0.012335],
                [0.152583, 0.188174, 0.253972],
                [0.333, 0.333, 0.522],
                BLACK,
            ],
            MetalConductor(1.8800, 3.4900),
        ),
        // Plastered: only `Kd` changes.
        spec(
            false,
            0.01,
            [gray(0.13), gray(0.75), gray(0.003936), BLACK],
            BsdfSpec::Layers {
                kd: gray(0.482353),
                ks: None,
                normalize: false,
                dielectric: false,
            },
        ),
        // Plastified
        spec(
            false,
            0.0078125,
            [gray(0.25), gray(0.24), gray(0.004896), BLACK],
            layers(
                gray(0.2),
                [0.00784314, 0.00784314, 0.00784314, 0.25],
                true,
                false,
            ),
        ),
        // Silver
        spec(
            true,
            0.75,
            [
                [0.061465, 0.061465, 0.050876],
                [0.354692, 0.354692, 0.354692],
                [0.950, 0.930, 0.880],
                BLACK,
            ],
            MetalSchlick([0.971519, 0.959915, 0.915324]),
        ),
        // Steel
        spec(
            true,
            0.90,
            [
                [0.019607, 0.019607, 0.027212],
                [0.214041, 0.223414, 0.233022],
                [0.560, 0.570, 0.580],
                BLACK,
            ],
            MetalConductorRgb([2.90, 2.80, 2.53], [3.08, 2.90, 2.74]),
        ),
        // Stone: the specular color is `NCollection_Vec3<float>(0.98, 1.0, 0.60) * 0.08f`.
        MaterialSpec {
            coefs: Some((0.19 * 0.25, 0.75)),
            ..spec(
                true,
                0.17,
                [
                    [0.030074, 0.020069, 0.013011],
                    [0.522522, 0.318547, 0.183064],
                    [0.98 * 0.08, 1.0 * 0.08, 0.60 * 0.08],
                    BLACK,
                ],
                layers(
                    gray(0.243137),
                    [0.00392157, 0.00392157, 0.00392157, 0.5],
                    false,
                    false,
                ),
            )
        },
        // Shiny_plastified
        spec(
            false,
            1.00,
            [gray(0.22), gray(0.50), gray(1.0), BLACK],
            layers(gray(0.2), [0.145, 0.145, 0.145, 0.17], true, false),
        ),
        // Satined
        spec(
            false,
            0.09375,
            [gray(0.165), gray(0.40), gray(0.162647), BLACK],
            layers(gray(0.2), [0.6, 0.6, 0.6, 0.6], false, false),
        ),
        // Metalized
        spec(
            false,
            0.13,
            [BLACK, gray(0.47), gray(0.170645), BLACK],
            MetalSchlick(gray(0.2)),
        ),
        // Ionized: `Le` is the diffuse color.
        spec(
            false,
            0.05,
            [BLACK, gray(1.0), gray(0.342392), gray(1.0)],
            Glow(gray(1.0)),
        ),
        // Chrome
        spec(
            true,
            0.90,
            [
                [0.033105, 0.033105, 0.041436],
                [0.263273, 0.263273, 0.263273],
                [0.975, 0.975, 0.975],
                BLACK,
            ],
            MetalSchlick([0.549585, 0.556114, 0.554256]),
        ),
        // Aluminium
        spec(
            true,
            0.75,
            [
                [0.073239, 0.073239, 0.073239],
                [0.318547, 0.318547, 0.318547],
                [0.910, 0.920, 0.920],
                BLACK,
            ],
            MetalSchlick([0.913183, 0.921494, 0.924524]),
        ),
        // Obsidian
        spec(
            true,
            0.3,
            [
                [0.004305, 0.003936, 0.005532],
                [0.028053, 0.024515, 0.041436],
                [0.333, 0.329, 0.346],
                BLACK,
            ],
            layers(
                [0.023, 0.0, 0.023],
                [0.0156863, 0.0156863, 0.0156863, 0.1],
                false,
                false,
            ),
        ),
        // Neon
        spec(
            true,
            0.05,
            [BLACK, BLACK, gray(0.62), [0.0, 0.787412, 0.142892]],
            Glow([0.0, 1.0, 0.46]),
        ),
        // Jade
        spec(
            true,
            0.10,
            [
                [0.016338, 0.040729, 0.021493],
                [0.252950, 0.767769, 0.354692],
                [0.316, 0.316, 0.316],
                BLACK,
            ],
            layers(
                [0.208658, 0.415686, 0.218401],
                [0.611765, 0.611765, 0.611765, 0.06],
                false,
                true,
            ),
        ),
        // Charcoal
        spec(
            true,
            0.01,
            [gray(0.003936), gray(0.019607), BLACK, BLACK],
            layers(gray(0.02), [0.1, 0.1, 0.1, 0.3], false, false),
        ),
        // Water
        glass(
            0.90,
            [
                [0.170645, 0.170645, 0.191627],
                [0.003936, 0.003936, 0.006571],
                [0.380, 0.380, 0.380],
                BLACK,
            ],
            [0.7, 0.75, 0.85],
            1.33,
        ),
        // Glass
        glass(
            0.50,
            [
                [0.263273, 0.290143, 0.290143],
                [0.003936, 0.006571, 0.006571],
                [0.920, 0.920, 0.920],
                BLACK,
            ],
            [0.75, 0.95, 0.9],
            1.62,
        ),
        // Diamond
        glass(
            0.90,
            [gray(0.263273), gray(0.010023), gray(0.970), BLACK],
            [0.95, 0.95, 0.95],
            2.42,
        ),
        // Transparent
        MaterialSpec {
            optics: Some((1.0, 0.80)),
            ..spec(
                true,
                0.90,
                [gray(0.263273), gray(0.010023), gray(0.970), BLACK],
                BsdfSpec::Transparent,
            )
        },
        // Default
        spec(false, 0.039, [gray(0.15), gray(0.65), BLACK, BLACK], Black),
        // UserDefined
        spec(true, 0.039, [gray(0.1), gray(0.6), gray(0.2), BLACK], Black),
    ]
};
