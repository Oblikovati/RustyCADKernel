// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp.cxx (the volume
// properties by Gauss-Kronrod integration: VolumePropertiesGK and volumePropertiesGK) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The volume properties of `BRepGProp` by adaptive Gauss-Kronrod integration.
use super::domain::BRepGPropDomain;
use super::face::BRepGPropFace;
use super::package::{BRepGProp, located_origin, rough_bary_center};
use super::vinert_gk::{BRepGPropVinertGK, VinertGKOptions};
use super::volume::{OrientedFaces, is_oriented};
use crate::brep_check::BRepCheckShell;
use crate::brep_check::BRepCheckStatus;
use crate::brep_check::explore::sub_shapes;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::MapOfShape;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Plane3;

/// The options of `VolumePropertiesGK` besides the tolerance and the closed flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VolumeGKFlags {
    /// `IsUseSpan`: integrate B-spline faces span by span.
    pub is_use_span: bool,
    /// `CGFlag`: compute the centre of mass.
    pub cg_flag: bool,
    /// `IFlag`: compute the inertia (and the centre).
    pub i_flag: bool,
    /// `SkipShared`: count each shared face once per orientation.
    pub skip_shared: bool,
}

impl BRepGProp {
    /// `VolumePropertiesGK(S, VProps, Eps, OnlyClosed, IsUseSpan, CGFlag, IFlag,
    /// SkipShared)`: `props` becomes a framework at the located origin holding the
    /// volume of `shape` between its faces and its rough barycentre (of each shell
    /// that BRepCheck_Shell finds closed, with `only_closed`: -1 when none is); the sum
    /// of the absolute errors, divided by the volume above `Epsilon(1)`, or a negative
    /// value on failure; `BRepGProp::volume_properties_gk(&shape, &mut props, 0.001, false, flags)?`.
    pub fn volume_properties_gk(
        shape: &Shape,
        props: &mut GProps,
        eps: f64,
        only_closed: bool,
        flags: VolumeGKFlags,
    ) -> Result<f64, Failure> {
        gk_by_shells(shape, props, (eps, only_closed, flags), None)
    }

    /// `VolumePropertiesGK(S, VProps, thePln, Eps, OnlyClosed, IsUseSpan, CGFlag, IFlag,
    /// SkipShared)`: the same between the faces and the plane `plane`;
    /// `BRepGProp::volume_properties_gk_plane(&shape, &mut props, plane, 0.001, false, flags)?`.
    pub fn volume_properties_gk_plane(
        shape: &Shape,
        props: &mut GProps,
        plane: Plane3,
        eps: f64,
        only_closed: bool,
        flags: VolumeGKFlags,
    ) -> Result<f64, Failure> {
        gk_by_shells(shape, props, (eps, only_closed, flags), Some(plane))
    }
}

/// Both VolumePropertiesGK: the shape or its closed shells, then the error divided by
/// the volume.
fn gk_by_shells(
    shape: &Shape,
    props: &mut GProps,
    (eps, only_closed, flags): (f64, bool, VolumeGKFlags),
    plane: Option<Plane3>,
) -> Result<f64, Failure> {
    *props = GProps::with_location(located_origin(shape));
    let error = if only_closed {
        let shells = closed_shells(shape, flags.skip_shared)?;
        if shells.is_empty() {
            return Ok(-1.);
        }
        let mut error = 0.;
        for shell in &shells {
            let local = volume_gk(shell, props, plane, eps, flags)?;
            if local < 0. {
                return Ok(local);
            }
            error += local;
        }
        error
    } else {
        volume_gk(shape, props, plane, eps, flags)?
    };
    let volume = props.mass();
    Ok(if volume > epsilon(1.) {
        error / volume
    } else {
        error
    })
}

/// The shells of `shape` that `BRepCheck_Shell::Closed(false)` accepts (each shared
/// shell once with `skip_shared`).
fn closed_shells(shape: &Shape, skip_shared: bool) -> Result<Vec<Shape>, Failure> {
    let mut seen = MapOfShape::default();
    let mut closed = Vec::new();
    for shell in sub_shapes(shape, ShapeEnum::Shell)? {
        if skip_shared && !seen.add(shell.clone()) {
            continue;
        }
        let checker = BRepCheckShell::new(&topods::shell(&shell)?)?;
        if checker.closed(false)? == BRepCheckStatus::NoError {
            closed.push(shell);
        }
    }
    Ok(closed)
}

/// `volumePropertiesGK`: the FORWARD and REVERSED faces with a surface, one VinertGK
/// framework located at the rough barycentre reused for every face (its centre and
/// inertia keep their previous values when not requested); the sum of the errors, or
/// the first negative one.
fn volume_gk(
    shape: &Shape,
    props: &mut GProps,
    plane: Option<Plane3>,
    tolerance: f64,
    flags: VolumeGKFlags,
) -> Result<f64, Failure> {
    let mut volume = BRepGPropVinertGK::new();
    volume.set_location(rough_bary_center(shape)?);
    let (mut tool, mut domain) = (
        BRepGPropFace::new(flags.is_use_span),
        BRepGPropDomain::new(),
    );
    let options = VinertGKOptions {
        tolerance,
        cg_flag: flags.cg_flag,
        i_flag: flags.i_flag,
    };
    let mut seen = OrientedFaces::default();
    let mut error = 0.;
    for shape in sub_shapes(shape, ShapeEnum::Face)? {
        let face = topods::face(&shape)?;
        if flags.skip_shared && seen.seen(&face) {
            continue;
        }
        if BRepTool::surface_with_location(&face)?.0.is_null() || !is_oriented(&face) {
            continue;
        }
        let local = gk_face(&face, (&mut tool, &mut domain), &mut volume, plane, options)?;
        if local < 0. {
            return Ok(local);
        }
        error += local;
        props.add(&volume, 1.0)?;
    }
    Ok(error)
}

/// The VinertGK integration of one face, naturally bounded or along its domain, by
/// point (the location) or by plane.
fn gk_face(
    face: &Face,
    (tool, domain): (&mut BRepGPropFace, &mut BRepGPropDomain),
    volume: &mut BRepGPropVinertGK,
    plane: Option<Plane3>,
    options: VinertGKOptions,
) -> Result<f64, Failure> {
    tool.load_face(face)?;
    if face.nb_children() == 0 {
        return match plane {
            Some(plane) => volume.perform_plane(tool, plane, options),
            None => volume.perform(tool, options),
        };
    }
    domain.init_face(face)?;
    match plane {
        Some(plane) => volume.perform_domain_plane(tool, domain, plane, options),
        None => volume.perform_domain(tool, domain, options),
    }
}
