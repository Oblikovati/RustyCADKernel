//! The bounds of `Graphic3d_Layer`: `BoundingBox` and `considerZoomPersistenceObjects`.
use super::layer::Graphic3dLayer;
use super::{
    Graphic3dBndBox3d, Graphic3dCStructure, Graphic3dCStructureMembers as _, Graphic3dCamera,
    Graphic3dGroupMembers as _, Graphic3dTransformPersApply as _,
    Graphic3dTransformPersMembers as _,
};
use crate::graphic3d::frame_stats_data::{native_max, native_min};
use rustycad_tkernel::ncollection::{Mat4, Vec3};
use rustycad_tkernel::standard::{Failure, Handle, WeakHandle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::Point3;

/// The view parameters of a layer box: camera, its matrices and the window size.
struct LayerView<'a> {
    camera: &'a Handle<Graphic3dCamera>,
    projection: Mat4<f64>,
    world_view: Mat4<f64>,
    size: [i32; 2],
}

impl LayerView<'_> {
    /// Moves `bounds` by the transform persistence of a structure or a group.
    fn apply(
        &self,
        pers: &Handle<dyn super::Graphic3dTransformPersSource>,
        bounds: &mut Graphic3dBndBox3d,
    ) -> Result<(), Failure> {
        match pers.get() {
            Some(pers) => pers.apply_bvh(
                self.camera,
                &self.projection,
                &self.world_view,
                self.size,
                bounds,
            ),
            None => Ok(()),
        }
    }
}

/// `centerOfinfiniteBndBox`: the center of a box with a large diagonal, else a void box.
fn center_of_infinite(bounds: &Graphic3dBndBox3d) -> Graphic3dBndBox3d {
    let diagonal = *bounds.corner_max() - *bounds.corner_min();
    if diagonal.square_modulus() >= 500000.0 * 500000.0 {
        return Graphic3dBndBox3d::from_point((*bounds.corner_min() + *bounds.corner_max()) * 0.5);
    }
    Graphic3dBndBox3d::default()
}

/// `isInfiniteBndBox`: a corner coordinate at or beyond the short-real range.
fn is_infinite(bounds: &Graphic3dBndBox3d) -> bool {
    let limit = f64::from(f32::MAX);
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    [max.x, max.y, max.z, min.x, min.y, min.z]
        .iter()
        .any(|v| v.abs() >= limit)
}

/// `addBox3dToBndBox`: adds the corners of a valid finite box.
fn add_box(result: &mut BoundingBox3, bounds: &Graphic3dBndBox3d) {
    if !bounds.is_valid() || is_infinite(bounds) {
        return;
    }
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    result.add_point(Point3::new(min.x, min.y, min.z));
    result.add_point(Point3::new(max.x, max.y, max.z));
}

/// The structures of every priority, from Bottom to Topmost.
fn all_structures(layer: &Graphic3dLayer) -> Vec<Handle<dyn Graphic3dCStructure>> {
    let state = layer.state.read();
    state
        .array
        .iter()
        .flat_map(|map| map.iter().map(WeakHandle::upgrade).collect::<Vec<_>>())
        .filter(|s| !s.is_null())
        .collect()
}

/// The contribution of one structure to the cached box.
fn add_structure_box(
    view: &LayerView<'_>,
    structure: &dyn Graphic3dCStructure,
    include_aux: bool,
    result: &mut BoundingBox3,
) -> Result<(), Failure> {
    let pers = structure.transform_persistence();
    if let Some(pers_ref) = pers.get() {
        if !include_aux && pers_ref.is_zoom_or_rotate() {
            result.add_point(pers_ref.anchor_point()?);
            return Ok(());
        }
        if !include_aux || pers_ref.is_trihedron_or_2d() {
            return Ok(());
        }
    }
    if !include_aux && structure.has_group_transform_persistence() {
        for group in structure.groups().iter().filter_map(Handle::get) {
            let group_pers = group.transform_persistence();
            if let Some(group_pers) = group_pers.get().filter(|p| p.is_zoom_or_rotate()) {
                result.add_point(group_pers.anchor_point()?);
            }
        }
    }
    let mut bounds = structure.bounding_box();
    if !bounds.is_valid() {
        return Ok(());
    }
    if structure.cstructure_base().fields.read().is_infinite && !include_aux {
        bounds = center_of_infinite(&bounds);
    }
    view.apply(&pers, &mut bounds)?;
    add_box(result, &bounds);
    Ok(())
}

/// The auxiliary boxes of an always rendered structure: its persistent groups and, for a
/// trihedron or 2D persistence, its own box.
fn add_auxiliary_box(
    view: &LayerView<'_>,
    structure: &dyn Graphic3dCStructure,
    result: &mut BoundingBox3,
) -> Result<(), Failure> {
    if structure.has_group_transform_persistence() {
        for group in structure.groups().iter().filter_map(Handle::get) {
            let (pers, box_f) = (group.transform_persistence(), group.bounding_box());
            if pers.is_null() || !box_f.is_valid() {
                continue;
            }
            let widen = |v: &rustycad_tkernel::ncollection::Vec4<f32>| {
                Vec3::new(f64::from(v.x), f64::from(v.y), f64::from(v.z))
            };
            let mut copy = Graphic3dBndBox3d::from_corners(
                widen(box_f.corner_min()),
                widen(box_f.corner_max()),
            );
            view.apply(&pers, &mut copy)?;
            add_box(result, &copy);
        }
    }
    let pers = structure.transform_persistence();
    let bounds = structure.bounding_box();
    if !bounds.is_valid() || !pers.get().is_some_and(|p| p.is_trihedron_or_2d()) {
        return Ok(());
    }
    let mut copy = bounds;
    view.apply(&pers, &mut copy)?;
    add_box(result, &copy);
    Ok(())
}

impl Graphic3dLayer {
    /// `BoundingBox`: the cached box of the visible structures (recomputed after an
    /// invalidation), plus the auxiliary boxes of the always rendered structures when asked.
    /// Example: `layer.bounding_box(0, &camera, [800, 600], false)?`.
    pub fn bounding_box(
        &self,
        view_id: i32,
        camera: &Handle<Graphic3dCamera>,
        size: [i32; 2],
        include_aux: bool,
    ) -> Result<BoundingBox3, Failure> {
        self.update_bvh();
        let Some(cam) = camera.get() else {
            return Err(rustycad_tkernel::standard::exceptions::program_error(
                "Graphic3d_Layer::BoundingBox: null camera; expected a camera",
            ));
        };
        let view = LayerView {
            camera,
            projection: cam.projection_matrix(),
            world_view: cam.orientation_matrix(),
            size,
        };
        let box_id = usize::from(include_aux);
        if self.state.read().is_bounding_box_needs_reset[box_id] {
            let mut cached = BoundingBox3::default();
            for structure in all_structures(self)
                .iter()
                .filter_map(Handle::get)
                .filter(|s| s.is_visible_in_view(view_id))
            {
                add_structure_box(&view, structure, include_aux, &mut cached)?;
            }
            let mut state = self.state.write();
            state.bounding_box[box_id] = cached;
            state.is_bounding_box_needs_reset[box_id] = false;
        }
        let (mut result, always) = {
            let state = self.state.read();
            (
                state.bounding_box[box_id].clone(),
                state.always_rendered.clone(),
            )
        };
        if !include_aux || always.extent() == 0 {
            return Ok(result);
        }
        for structure in always.iter().map(WeakHandle::upgrade) {
            if let Some(structure) = structure.get().filter(|s| s.is_visible_in_view(view_id)) {
                add_auxiliary_box(&view, structure, &mut result)?;
            }
        }
        Ok(result)
    }
    /// `considerZoomPersistenceObjects`: the scale that brings the zoom-persistent structures
    /// partly out of the screen back in (1 when none); `layer.consider_zoom_persistence_objects(0, &camera, [800, 600])?`.
    pub fn consider_zoom_persistence_objects(
        &self,
        view_id: i32,
        camera: &Handle<Graphic3dCamera>,
        size: [i32; 2],
    ) -> Result<f64, Failure> {
        if self.nb_of_transform_persistence_objects() == 0 {
            return Ok(1.0);
        }
        let Some(cam) = camera.get() else {
            return Err(rustycad_tkernel::standard::exceptions::program_error(
                "Graphic3d_Layer::considerZoomPersistenceObjects: null camera; expected a camera",
            ));
        };
        let view = LayerView {
            camera,
            projection: cam.projection_matrix(),
            world_view: cam.orientation_matrix(),
            size,
        };
        let mut max_coef = -f64::MAX;
        for structure in all_structures(self).iter().filter_map(Handle::get) {
            let pers = structure.transform_persistence();
            let Some(pers_ref) = pers
                .get()
                .filter(|p| p.is_zoom_or_rotate() && structure.is_visible_in_view(view_id))
            else {
                continue;
            };
            let mut bounds = structure.bounding_box();
            if !bounds.is_valid() {
                continue;
            }
            view.apply(&pers, &mut bounds)?;
            let anchor = cam.project(&pers_ref.anchor_point()?);
            max_coef = zoom_coefficient(cam, &bounds, anchor, max_coef);
        }
        Ok(if max_coef > 0.0 { max_coef } else { 1.0 })
    }
}

/// The projected extent `[min x, max x, min y, max y]` of the eight corners of a box.
fn projected_extent(camera: &Graphic3dCamera, bounds: &Graphic3dBndBox3d) -> [f64; 4] {
    let (min, max) = (bounds.corner_min(), bounds.corner_max());
    let mut extent = [f64::MAX, -f64::MAX, f64::MAX, -f64::MAX];
    for (x, y, z) in [
        (min.x, min.y, min.z),
        (min.x, min.y, max.z),
        (min.x, max.y, min.z),
        (min.x, max.y, max.z),
        (max.x, min.y, min.z),
        (max.x, min.y, max.z),
        (max.x, max.y, min.z),
        (max.x, max.y, max.z),
    ] {
        let point = camera.project(&Point3::new(x, y, z));
        extent = [
            native_min(extent[0], point.x()),
            native_max(extent[1], point.x()),
            native_min(extent[2], point.y()),
            native_max(extent[3], point.y()),
        ];
    }
    extent
}

/// The shift that brings a projected interval back into [-1, 1].
fn screen_shift(min: f64, max: f64) -> f64 {
    if min < -1.0 {
        return if max < -1.0 {
            -(1.0 + max) + (max - min)
        } else {
            -(1.0 + min)
        };
    }
    if max > 1.0 {
        return if min > 1.0 {
            (min - 1.0) + (max - min)
        } else {
            max - 1.0
        };
    }
    0.0
}

/// The native coefficient update of one zoom-persistent structure box.
fn zoom_coefficient(
    camera: &Graphic3dCamera,
    bounds: &Graphic3dBndBox3d,
    anchor: Point3,
    max_coef: f64,
) -> f64 {
    let [min_x, max_x, min_y, max_y] = projected_extent(camera, bounds);
    let is_big = (max_x - min_x).abs() > 2.0 || (max_y - min_y).abs() > 2.0;
    let inside = |v: f64| v > -1.0 && v < 1.0;
    let in_screen = inside(min_x) && inside(max_x) && inside(min_y) && inside(max_y);
    let confusion = rustycad_tkernel::precision::confusion();
    let (ax, ay) = (anchor.x(), anchor.y());
    if is_big || in_screen || (ax * ax + ay * ay).sqrt() < confusion {
        return max_coef;
    }
    let mut coef = max_coef;
    let dif_x = ax.abs() - screen_shift(min_x, max_x);
    let dif_y = ay.abs() - screen_shift(min_y, max_y);
    if dif_x > confusion {
        coef = native_max(coef, ax.abs() / dif_x);
    }
    if dif_y > confusion {
        coef = native_max(coef, ay.abs() / dif_y);
    }
    coef
}
