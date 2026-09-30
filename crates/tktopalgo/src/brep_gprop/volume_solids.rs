// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepGProp/BRepGProp.cxx (volumeProperties
// with its cache of shared solids, and the GPropTransformer helper) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `volumeProperties`: the volume of a shape, computing each solid shared by several
//! instances (the same TShape) once and moving its properties to the other instances.
use super::package::rough_bary_center;
use super::volume::{VolumeFlags, volume_faces};
use crate::brep_check::explore::{sub_shapes, sub_shapes_avoiding};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{Point3, Transform3, TransformForm};
use rustycad_tkmath::toploc::Location;
use std::collections::{HashMap, HashSet};

/// `SolidCacheEntry`: the properties of the first instance of a solid, its location and
/// orientation.
struct SolidCacheEntry {
    props: GProps,
    location: Location,
    orientation: Orientation,
}

/// `volumeProperties(S, Props, Eps, SkipShared, UseTriangulation)`: the faces of the
/// shape about its rough barycentre when no solid is shared, else solid by solid with
/// the cache, then the faces outside every solid; the largest error.
pub(super) fn volume_props(
    shape: &Shape,
    props: &mut GProps,
    eps: f64,
    flags: VolumeFlags,
) -> Result<f64, Failure> {
    let centre = rough_bary_center(shape)?;
    let solids = sub_shapes(shape, ShapeEnum::Solid)?;
    // The cache is looked up by TShape and never iterated, so the addresses decide
    // nothing but identity.
    let mut seen = HashSet::new();
    if solids
        .iter()
        .all(|solid| seen.insert(solid.tshape_address()))
    {
        return volume_faces(shape, props, centre, eps, flags);
    }
    let mut cache: HashMap<usize, SolidCacheEntry> = HashMap::new();
    let mut error_max = 0.0;
    for solid in &solids {
        let error = match cache.get(&solid.tshape_address()) {
            Some(cached) => add_instance(solid, cached, props, (centre, eps, flags))?,
            None => {
                let (solid_props, error) = solid_volume(solid, centre, eps, flags)?;
                let entry = SolidCacheEntry {
                    props: solid_props,
                    location: solid.location().clone(),
                    orientation: solid.orientation(),
                };
                cache.insert(solid.tshape_address(), entry);
                props.add(&solid_props, 1.0)?;
                error
            }
        };
        if error_max < error {
            error_max = error;
        }
    }
    let free_error = add_free_faces(shape, props, centre, (eps, flags))?;
    if error_max < free_error {
        error_max = free_error;
    }
    Ok(error_max)
}

/// The properties of one solid computed face by face, in a framework at `centre`.
fn solid_volume(
    solid: &Shape,
    centre: Point3,
    eps: f64,
    flags: VolumeFlags,
) -> Result<(GProps, f64), Failure> {
    let mut solid_props = GProps::with_location(centre);
    let error = volume_faces(solid, &mut solid_props, centre, eps, flags)?;
    Ok((solid_props, error))
}

/// A further instance of a cached solid: skipped when it is an exact duplicate with
/// `skip_shared`, computed face by face when its relative location is not rigid, else
/// the cached properties moved (and negated for the other orientation); the error.
fn add_instance(
    solid: &Shape,
    cached: &SolidCacheEntry,
    props: &mut GProps,
    (centre, eps, flags): (Point3, f64, VolumeFlags),
) -> Result<f64, Failure> {
    let relative = solid.location().multiplied(&cached.location.inverted()?)?;
    if flags.skip_shared && relative.is_identity() && solid.orientation() == cached.orientation {
        return Ok(0.0);
    }
    let transform = relative.transformation();
    if (transform.scale_factor().abs() - 1.0).abs() > Location::scale_prec()
        || transform.is_negative()
    {
        let (solid_props, error) = solid_volume(solid, centre, eps, flags)?;
        props.add(&solid_props, 1.0)?;
        return Ok(error);
    }
    let mut moved = cached.props;
    if !relative.is_identity() {
        apply_transform(&mut moved, transform);
    }
    if solid.orientation() != cached.orientation {
        moved.dim = -moved.dim;
        moved.inertia *= -1.0;
    }
    props.add(&moved, 1.0)?;
    Ok(0.0)
}

/// `GPropTransformer::ApplyTransform`: the reference point moved, the relative centre
/// and the inertia rotated (not for an identity or a translation).
fn apply_transform(props: &mut GProps, transform: Transform3) {
    props.loc.transform(transform);
    if matches!(
        transform.form(),
        TransformForm::Identity | TransformForm::Translation
    ) {
        return;
    }
    let rotation = transform.vectorial_part();
    props.g = Point3::from(rotation * props.g.xyz());
    props.inertia = (rotation * props.inertia) * rotation.transposed();
}

/// The faces outside every solid, as one compound about `centre`; the error.
fn add_free_faces(
    shape: &Shape,
    props: &mut GProps,
    centre: Point3,
    (eps, flags): (f64, VolumeFlags),
) -> Result<f64, Failure> {
    let free = sub_shapes_avoiding(shape, ShapeEnum::Face, ShapeEnum::Solid)?;
    if free.is_empty() {
        return Ok(0.0);
    }
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for face in &free {
        builder.add(&mut compound, face)?;
    }
    let (free_props, error) = solid_volume(&compound.into_shape(), centre, eps, flags)?;
    props.add(&free_props, 1.0)?;
    Ok(error)
}
