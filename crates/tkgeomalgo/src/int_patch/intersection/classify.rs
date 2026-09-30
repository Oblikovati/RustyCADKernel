// Rust translation of the classification of the surface pair at the start of the two
// IntPatch_Intersection::Perform overloads of two surfaces
// (ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Intersection.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The kind of intersection of a pair of surfaces: implicit-implicit, implicit-
//! parametric or parametric-parametric. A cone or a torus is treated as a parametric
//! surface when it is nearly degenerated, unless its axis is placed simply with respect
//! to the other surface.
use crate::int_start::TheTopolTool;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Axis1, Cone3, Line3, Plane3};

/// The two `Perform` overloads classify the tori differently and only the first one
/// sends a nearly degenerated cone and a plane to the implicit-parametric intersection.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Variant {
    /// `Perform(S1, D1, S2, D2, TolArc, TolTang, ...)`.
    Plain,
    /// `Perform(S1, D1, S2, D2, TolArc, TolTang, ListOfPnts, ...)`.
    WithPoints,
}

/// The surface types after the classification and the `bGeomGeom` flag.
pub(super) struct PairKind {
    pub(super) typs: (SurfaceType, SurfaceType),
    pub(super) geom_geom: bool,
}

impl PairKind {
    /// `ts1` and `ts2`: 1 for an implicit surface (a torus only with `bGeomGeom`).
    pub(super) fn implicit(&self) -> (bool, bool) {
        let ts = |t: SurfaceType| match t {
            SurfaceType::Plane
            | SurfaceType::Cylinder
            | SurfaceType::Sphere
            | SurfaceType::Cone => true,
            SurfaceType::Torus => self.geom_geom,
            _ => false,
        };
        (ts(self.typs.0), ts(self.typs.1))
    }
}

fn is_cone_or_torus(t: SurfaceType) -> bool {
    t == SurfaceType::Cone || t == SurfaceType::Torus
}

/// The state of the classification: the cone or torus surface, the other surface, the
/// axis of the first one and the flags.
struct Check<'a> {
    ct_surf: &'a Adaptor3dSurfaceHandle,
    geom_surf: &'a Adaptor3dSurfaceHandle,
    ct_type: SurfaceType,
    ct_ax: Axis1,
    to_check: bool,
    treat_as_bi_parametric: bool,
    geom_geom: bool,
}

impl Check<'_> {
    /// The cone part: a cone of a semi-angle below 0.02 or above 1.55 is checked, unless
    /// two such cones are coplanar quasi-planes.
    fn cone_part(&mut self, typs: (SurfaceType, SurfaceType)) -> Result<(), Failure> {
        let con1: Cone3 = if self.ct_type == SurfaceType::Cone {
            self.ct_surf.cone()?
        } else {
            self.geom_surf.cone()?
        };
        let a1: f64 = con1.semi_angle().abs();
        self.to_check = (a1 < 0.02) || (a1 > 1.55);
        if typs.0 == typs.1 {
            let con2: Cone3 = self.geom_surf.cone()?;
            let a2: f64 = con2.semi_angle().abs();
            self.to_check = self.to_check || (a2 < 0.02) || (a2 > 1.55);
            // Quasi-planes: if same domain, treat as canonic.
            if a1 > 1.55 && a2 > 1.55 && same_quasi_plane(&con1, &con2)? {
                self.to_check = false;
            }
        }
        self.treat_as_bi_parametric = self.to_check;
        if self.ct_type == SurfaceType::Cone {
            self.ct_ax = con1.axis();
        }
        Ok(())
    }

    /// The torus part: a torus is checked when its major radius is the larger one.
    fn torus_part(
        &mut self,
        typs: (SurfaceType, SurfaceType),
        variant: Variant,
        tol_tang: f64,
    ) -> Result<(), Failure> {
        let tor1 = if self.ct_type == SurfaceType::Torus {
            self.ct_surf.torus()?
        } else {
            self.geom_surf.torus()?
        };
        self.to_check = tor1.major_radius() > tor1.minor_radius();
        if typs.0 == typs.1 {
            let tor2 = self.geom_surf.torus()?;
            self.to_check = match variant {
                Variant::Plain => tor2.major_radius() > tor2.minor_radius(),
                Variant::WithPoints => {
                    (self.to_check && (tor2.major_radius() > tor2.minor_radius()))
                        || ((tor1.major_radius() - tor2.major_radius()).abs() < tol_tang
                            && (tor1.minor_radius() - tor2.minor_radius()).abs() < tol_tang)
                }
            };
        }
        if self.ct_type == SurfaceType::Torus {
            self.ct_ax = tor1.axis();
        }
        Ok(())
    }

    /// The position of the axis with respect to the other surface: `bGeomGeom` when it
    /// is simple (normal to a plane through it, through the centre of a sphere, coaxial).
    fn axis_part(&mut self) -> Result<(), Failure> {
        let l1 = Line3::from(self.ct_ax);
        let angular: f64 = precision::angular();
        let geom_ax: Axis1 = match self.geom_surf.get_type()? {
            SurfaceType::Plane => {
                self.plane_part(angular)?;
                return Ok(());
            }
            SurfaceType::Sphere => {
                if l1.distance(self.geom_surf.sphere()?.location()) < precision::confusion() {
                    self.geom_geom = true;
                }
                return Ok(());
            }
            SurfaceType::Cylinder => self.geom_surf.cylinder()?.axis(),
            SurfaceType::Cone => self.geom_surf.cone()?.axis(),
            SurfaceType::Torus => self.geom_surf.torus()?.axis(),
            _ => return Ok(()),
        };
        if self.ct_ax.is_parallel(geom_ax, angular)
            && l1.distance(geom_ax.location()) <= precision::confusion()
        {
            self.geom_geom = true;
        }
        Ok(())
    }

    /// The plane case of `axis_part`.
    fn plane_part(&mut self, angular: f64) -> Result<(), Failure> {
        let plane: Plane3 = self.geom_surf.plane()?;
        let geom_ax: Axis1 = plane.axis();
        if self.ct_type == SurfaceType::Cone {
            self.geom_geom = true;
            if self.ct_surf.cone()?.semi_angle().abs() < 0.02 {
                let ps: f64 = self.ct_ax.direction().dot(geom_ax.direction()).abs();
                if ps < 0.015 {
                    self.geom_geom = false;
                }
            }
            return Ok(());
        }
        if self.ct_ax.is_parallel(geom_ax, angular)
            || (self.ct_ax.is_normal(geom_ax, angular)
                && plane.distance(self.ct_ax.location()) < precision::confusion())
        {
            self.geom_geom = true;
        }
        Ok(())
    }
}

/// Whether two parallel quasi-plane cones have their apexes in one plane.
fn same_quasi_plane(con1: &Cone3, con2: &Cone3) -> Result<bool, Failure> {
    let (a1, a2) = (con1.axis(), con2.axis());
    if !a1.is_parallel(a2, precision::angular()) {
        return Ok(false);
    }
    let plan1 = Plane3::new(con1.apex(), a1.direction())?;
    Ok(plan1.distance(con2.apex()) <= precision::confusion())
}

/// The classification of the pair of surfaces of `Perform`.
pub(super) fn classify(
    (s1, d1): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
    (s2, d2): (&Adaptor3dSurfaceHandle, &(dyn TheTopolTool + 'static)),
    tol_tang: f64,
    variant: Variant,
) -> Result<PairKind, Failure> {
    let (mut typs1, mut typs2) = (s1.get_type()?, s2.get_type()?);
    let typs = (typs1, typs2);
    let mut treat_as_bi_parametric = false;
    let mut geom_geom = false;
    if is_cone_or_torus(typs1) || is_cone_or_torus(typs2) {
        let first_ct: bool = is_cone_or_torus(typs1);
        let (ct_surf, geom_surf) = if first_ct { (s1, s2) } else { (s2, s1) };
        let mut check = Check {
            ct_surf,
            geom_surf,
            ct_type: ct_surf.get_type()?,
            ct_ax: Axis1::default(),
            to_check: false,
            treat_as_bi_parametric: false,
            geom_geom: false,
        };
        if typs1 == SurfaceType::Cone || typs2 == SurfaceType::Cone {
            check.cone_part(typs)?;
        }
        if typs1 == SurfaceType::Torus || typs2 == SurfaceType::Torus {
            check.torus_part(typs, variant, tol_tang)?;
        }
        if check.to_check {
            check.axis_part()?;
            if check.geom_geom {
                check.treat_as_bi_parametric = false;
            }
        }
        treat_as_bi_parametric = check.treat_as_bi_parametric;
        geom_geom = check.geom_geom;
    }
    if d1.domain_is_infinite() || d2.domain_is_infinite() {
        treat_as_bi_parametric = false;
    }
    if treat_as_bi_parametric {
        (typs1, typs2) = bi_parametric_types((typs1, typs2), variant);
    }
    Ok(PairKind {
        typs: (typs1, typs2),
        geom_geom,
    })
}

/// The types that send a pair treated as bi-parametric to the parametric intersectors:
/// a cone and a plane go to the implicit-parametric one in the plain overload.
fn bi_parametric_types(
    (typs1, typs2): (SurfaceType, SurfaceType),
    variant: Variant,
) -> (SurfaceType, SurfaceType) {
    let bezier = SurfaceType::BezierSurface;
    if variant == Variant::Plain {
        if typs1 == SurfaceType::Cone && typs2 == SurfaceType::Plane {
            // Using Imp-Prm Intersector.
            return (bezier, typs2);
        } else if typs1 == SurfaceType::Plane && typs2 == SurfaceType::Cone {
            return (typs1, bezier);
        }
    }
    // Using Prm-Prm Intersector.
    (bezier, bezier)
}
