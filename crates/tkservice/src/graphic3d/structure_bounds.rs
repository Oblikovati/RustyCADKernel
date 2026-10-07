//! The bounds of `Graphic3d_Structure`: `CalculateBoundBox`, `MinMaxValues` and the private
//! `minMaxCoord`, `getBox` and `addTransformed`.
use super::{
    Graphic3dBndBox3d, Graphic3dBndBox4f, Graphic3dCStructureMembers as _,
    Graphic3dGroupMembers as _, Graphic3dStructureMembers as _, Graphic3dStructureSource,
};
use rustycad_tkernel::ncollection::{Mat4, Vec3};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::bnd::BoundingBox3;

/// `0.5 * 1e6` squared: the diagonal from which an infinite structure counts by its center.
const INFINITE_DIAGONAL2: f64 = 500000.0 * 500000.0; // tol:metric native threshold

/// The native `CalculateBoundBox`: the transformed box (ignoring the infinite flag) stored in
/// the driver structure.
pub(crate) fn calculate_bound_box<S: Graphic3dStructureSource + ?Sized>(structure: &S) {
    let mut bounds = Graphic3dBndBox3d::default();
    add_transformed(structure, &mut bounds, true);
    structure.with_cstructure(|c| c.set_bounding_box(bounds));
}

/// `minMaxCoord`: the union of the group boxes without transform persistence.
fn min_max_coord<S: Graphic3dStructureSource + ?Sized>(structure: &S) -> Graphic3dBndBox4f {
    let mut bounds = Graphic3dBndBox4f::default();
    for group in structure.groups().iter().filter_map(Handle::get) {
        if group.transform_persistence().is_null() {
            bounds.combine(&group.bounding_box());
        }
    }
    bounds
}

/// `getBox`: the group box in double precision; an infinite structure gives its center
/// (large diagonal) or the whole real range, unless the flag is ignored.
fn get_box<S: Graphic3dStructureSource + ?Sized>(
    structure: &S,
    bounds: &mut Graphic3dBndBox3d,
    ignore_infinite: bool,
) {
    let box_f = min_max_coord(structure);
    if !box_f.is_valid() {
        return;
    }
    let widen = |v: &rustycad_tkernel::ncollection::Vec4<f32>| {
        Vec3::new(f64::from(v.x), f64::from(v.y), f64::from(v.z))
    };
    *bounds = Graphic3dBndBox3d::from_corners(widen(box_f.corner_min()), widen(box_f.corner_max()));
    if !structure.is_infinite() || ignore_infinite {
        return;
    }
    let diagonal = *bounds.corner_max() - *bounds.corner_min();
    *bounds = if diagonal.square_modulus() >= INFINITE_DIAGONAL2 {
        Graphic3dBndBox3d::from_point((*bounds.corner_min() + *bounds.corner_max()) * 0.5)
    } else {
        Graphic3dBndBox3d::from_corners(
            Vec3::new(f64::MIN, f64::MIN, f64::MIN),
            Vec3::new(f64::MAX, f64::MAX, f64::MAX),
        )
    };
}

/// `addTransformed`: combines the boxes of the structure and its descendants, transforms
/// them by the structure transformation and adds them to `bounds` (untransformed when the
/// transformation makes them invalid).
/// Example: `add_transformed(&structure, &mut bounds, true)`.
pub(crate) fn add_transformed<S: Graphic3dStructureSource + ?Sized>(
    structure: &S,
    bounds: &mut Graphic3dBndBox3d,
    ignore_infinite: bool,
) {
    let mut combined = Graphic3dBndBox3d::default();
    get_box(structure, &mut combined, ignore_infinite);
    let descendants = structure.structure_base().fields.read().descendants.clone();
    // Quirk kept: the child box is reused across the descendants, so a descendant without
    // a valid box contributes the box of the previous one again.
    let mut child = Graphic3dBndBox3d::default();
    for descendant in descendants.iter().map(|d| d.upgrade()) {
        if let Some(descendant) = descendant.get() {
            get_box(descendant, &mut child, ignore_infinite);
        }
        combined.combine(&child);
    }
    if !combined.is_valid() {
        return;
    }
    let mut transformed = combined;
    if let Some(trsf) = structure.transformation().get() {
        let mut matrix = Mat4::<f64>::identity();
        trsf.trsf().get_mat4(&mut matrix);
        transformed.transform(&matrix);
    }
    bounds.combine(if transformed.is_valid() {
        &transformed
    } else {
        &combined
    });
}

/// The structure bounds as a `Bnd_Box`, for every structure.
pub trait Graphic3dStructureBounds: Graphic3dStructureSource {
    /// `MinMaxValues`: the transformed box, void when invalid, whole when it covers the
    /// short-real range; `structure.min_max_values(false)`.
    fn min_max_values(&self, ignore_infinite: bool) -> BoundingBox3 {
        let mut bounds = Graphic3dBndBox3d::default();
        add_transformed(self, &mut bounds, ignore_infinite);
        let mut result = BoundingBox3::default();
        if !bounds.is_valid() {
            return result;
        }
        let (min, max) = (bounds.corner_min(), bounds.corner_max());
        result.update_bounds([min.x, min.y, min.z], [max.x, max.y, max.z]);
        let (lim_min, lim_max) = (f64::from(f32::MIN) + 1.0, f64::from(f32::MAX) - 1.0);
        let (Ok(low), Ok(high)) = (result.corner_min(), result.corner_max()) else {
            return result;
        };
        let below = [low.x(), low.y(), low.z()].iter().all(|v| *v < lim_min);
        let above = [high.x(), high.y(), high.z()].iter().all(|v| *v > lim_max);
        if below && above {
            result.set_whole();
        }
        result
    }
}
impl<T: Graphic3dStructureSource + ?Sized> Graphic3dStructureBounds for T {}
