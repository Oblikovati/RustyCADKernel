//! The camera-dependent members of `Graphic3d_TransformPers`: the persistent scale,
//! `ComputeApply`, the world-view `Apply`, `Compute` and the box `Apply` members.
use super::bnd_box::{apply_on_bnd_box, transform_box_corners};
use super::transform_pers::{Graphic3dTransformPersBase, Graphic3dTransformPersMembers};
use super::transform_utils::{scale, translate};
use super::{
    Graphic3dCamera, Graphic3dCameraFocusType, Graphic3dCameraProjection,
    Graphic3dTransModeFlags as Flags, Graphic3dTransformPersSource, Graphic3dTransformScalar,
};
use crate::aspect::AspectTypeOfTriedronPosition as Corner;
use rustycad_tkernel::ncollection::{ElementCast, Mat4, Vec3, Vec4};
use rustycad_tkernel::standard::exceptions::program_error;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::bvh::{BvhBox, BvhVector};
use rustycad_tkmath::gp::{Point3, Xyz};
use std::ops::{Div, Mul};

/// The sub-pixel shift of the corner placements (`aJitterComp`).
const JITTER_COMP: f64 = 0.001;

/// The body of the default `persistentScale`: the view height per pixel at the depth of the
/// anchor point. Example: `graphic3d_transform_pers_persistent_scale(base, &camera, 800, 600)`.
pub fn graphic3d_transform_pers_persistent_scale(
    base: &Graphic3dTransformPersBase,
    camera: &Graphic3dCamera,
    _viewport_width: i32,
    viewport_height: i32,
) -> f64 {
    let tile = camera.tile();
    let vp_size_y = if tile.is_valid() {
        tile.total_size.y
    } else {
        viewport_height
    };
    let [px, py, pz] = base.state.read().params_3d();
    let (to_eye, eye) = (Xyz::from(camera.direction()), camera.eye());
    let to_obj = Xyz::new(px - eye.x(), py - eye.y(), pz - eye.z());
    let focus = to_obj.x() * to_eye.x() + to_obj.y() * to_eye.y() + to_obj.z() * to_eye.z();
    camera.view_dimensions_at(focus).y().abs() / f64::from(vp_size_y)
}

/// The focus distance of the corner placements.
fn corner_focus(camera: &Graphic3dCamera) -> f64 {
    if camera.is_orthographic() {
        camera.distance()
    } else if camera.z_focus_type() == Graphic3dCameraFocusType::Relative {
        camera.z_focus() * camera.distance()
    } else {
        camera.z_focus()
    }
}

/// The camera `ComputeApply` works with: an orthographic copy for ortho persistence.
fn proxy_camera(
    ortho_pers: bool,
    camera: &Handle<Graphic3dCamera>,
) -> Result<Handle<Graphic3dCamera>, Failure> {
    let Some(source) = camera.get() else {
        return Err(program_error(
            "Graphic3d_TransformPers::ComputeApply() - null camera; expected a camera",
        ));
    };
    if !ortho_pers || source.is_orthographic() {
        return Ok(camera.clone());
    }
    let copy = source.copy_members();
    copy.set_projection_type(Graphic3dCameraProjection::Orthographic);
    Ok(Handle::new(copy))
}

/// The view height per pixel of the corner placements and the view dimensions.
fn pixel_scale(camera: &Graphic3dCamera, focus: f64, viewport_height: i32) -> (f64, Xyz) {
    let tile = camera.tile();
    let vp_size_y = if tile.is_valid() {
        tile.total_size.y
    } else {
        viewport_height
    };
    let dims = camera.view_dimensions_at(focus);
    (dims.y().abs() / f64::from(vp_size_y), dims)
}

/// `Scale(aWorldView, 1 / AxialScale)`.
fn unscale_axial(world_view: &mut Mat4<f64>, camera: &Graphic3dCamera) {
    let axial = camera.axial_scale();
    scale(
        world_view,
        [1.0 / axial.x(), 1.0 / axial.y(), 1.0 / axial.z()],
    );
}

/// The trihedron branch of `ComputeApply`.
fn trihedron_world_view(
    world_view: &mut Mat4<f64>,
    proxy: &Graphic3dCamera,
    camera: &Graphic3dCamera,
    [offset_x, offset_y, corner]: [i32; 3],
    viewport_height: i32,
) -> Result<(), Failure> {
    let focus = corner_focus(proxy);
    let (pixel, dims) = pixel_scale(proxy, focus, viewport_height);
    let forward = proxy.direction();
    let mut center = Xyz::from(proxy.center()) + Xyz::from(forward) * (focus - proxy.distance());
    if corner & (Corner::LEFT.0 | Corner::RIGHT.0) != 0 {
        let offset = (f64::from(offset_x) + JITTER_COMP) * pixel;
        let side = forward.crossed(proxy.up())?;
        let delta = Xyz::from(side) * (dims.x().abs() * proxy.ndc_2d_offset_x() - offset);
        shift(&mut center, delta, corner & Corner::RIGHT.0 != 0);
    }
    if corner & (Corner::TOP.0 | Corner::BOTTOM.0) != 0 {
        let offset = (f64::from(offset_y) + JITTER_COMP) * pixel;
        let delta = Xyz::from(proxy.up()) * (dims.y().abs() * proxy.ndc_2d_offset_y() - offset);
        shift(&mut center, delta, corner & Corner::TOP.0 != 0);
    }
    unscale_axial(world_view, camera);
    translate(world_view, [center.x(), center.y(), center.z()]);
    scale(world_view, [pixel; 3]);
    Ok(())
}

fn shift(center: &mut Xyz, delta: Xyz, forward: bool) {
    if forward {
        *center += delta;
    } else {
        *center -= delta;
    }
}

/// The 2D branch of `ComputeApply`.
fn screen_world_view(
    world_view: &mut Mat4<f64>,
    proxy: &Graphic3dCamera,
    [offset_x, offset_y, corner]: [i32; 3],
    viewport_height: i32,
) {
    let focus = corner_focus(proxy);
    let (pixel, dims) = pixel_scale(proxy, focus, viewport_height);
    let mut center = [0.0, 0.0, -focus];
    if corner & (Corner::LEFT.0 | Corner::RIGHT.0) != 0 {
        let x = -dims.x() * proxy.ndc_2d_offset_x() + (f64::from(offset_x) + JITTER_COMP) * pixel;
        center[0] = if corner & Corner::RIGHT.0 != 0 { -x } else { x };
    }
    if corner & (Corner::TOP.0 | Corner::BOTTOM.0) != 0 {
        let y = -dims.y() * proxy.ndc_2d_offset_y() + (f64::from(offset_y) + JITTER_COMP) * pixel;
        center[1] = if corner & Corner::TOP.0 != 0 { -y } else { y };
    }
    world_view.init_identity();
    translate(world_view, center);
    scale(world_view, [pixel; 3]);
}

/// The anchored (zoom, rotate, axial) branch of `ComputeApply`.
fn anchored_world_view<P: Graphic3dTransformPersSource + ?Sized>(
    pers: &P,
    world_view: &mut Mat4<f64>,
    (proxy, camera): (&Graphic3dCamera, &Graphic3dCamera),
    anchor: Option<&Point3>,
    [width, height]: [i32; 2],
) {
    let mode = pers.mode();
    let [px, py, pz] = pers.transform_pers_base().state.read().params_3d();
    let point = anchor.map_or([px, py, pz], |p| [p.x(), p.y(), p.z()]);
    translate(world_view, point);
    if mode.0 & Flags::ROTATE_PERS.0 != 0 {
        let rotation = pers.persistent_rotation_matrix(camera, width, height);
        for column in 0..3 {
            let values = rotation.get_column(column);
            (0..3).for_each(|row| {
                world_view.set_value(row, column, [values.x, values.y, values.z][row])
            });
        }
    }
    if pers.is_axial() {
        unscale_axial(world_view, camera);
    }
    if mode.0 & Flags::ZOOM_PERS.0 != 0 {
        let factor = pers.persistent_scale(proxy, width, height);
        scale(world_view, [factor; 3]);
    }
}

/// The body of the virtual `ComputeApply`: the world-view matrix of the persistence for the
/// camera (replaced by its orthographic copy for ortho persistence).
/// Example: `graphic3d_transform_pers_compute_apply(&pers, &mut camera, 800, 600, None)?`.
pub fn graphic3d_transform_pers_compute_apply<P: Graphic3dTransformPersSource + ?Sized>(
    pers: &P,
    camera: &mut Handle<Graphic3dCamera>,
    viewport_width: i32,
    viewport_height: i32,
    anchor: Option<&Point3>,
) -> Result<Mat4<f64>, Failure> {
    let proxy = proxy_camera(pers.is_ortho_pers(), camera)?;
    let (source, view) = (&**camera, &*proxy);
    let mut world_view = view.orientation_matrix();
    let state = *pers.transform_pers_base().state.read();
    let mode = state.mode.0;
    if mode & Flags::TRIEDRON_PERS.0 != 0 {
        trihedron_world_view(
            &mut world_view,
            view,
            source,
            state.params_2d(),
            viewport_height,
        )?;
    } else if mode & Flags::TMF_2D.0 != 0 {
        screen_world_view(&mut world_view, view, state.params_2d(), viewport_height);
    } else if mode & Flags::CAMERA_PERS.0 != 0 {
        world_view.init_identity();
    } else {
        let size = [viewport_width, viewport_height];
        anchored_world_view(pers, &mut world_view, (view, source), anchor, size);
    }
    *camera = proxy;
    Ok(world_view)
}

/// The templated `Apply` and `Compute` members of `Graphic3d_TransformPers`, over float and
/// double matrices, for every implementer.
pub trait Graphic3dTransformPersApply: Graphic3dTransformPersSource {
    /// `Apply(camera, projection, worldView&, width, height, anchor, toApplyProjPers)`: replaces
    /// the world-view matrix by the persistent one (nothing for no persistence or a zero
    /// height); a singular projection with ortho persistence on a perspective camera raises
    /// the `Standard_ConstructionError` of `Inverted()`.
    /// Example: `pers.apply_world_view(&camera, &projection, &mut world_view, [800, 600], None, true)?`.
    fn apply_world_view<T>(
        &self,
        camera: &Handle<Graphic3dCamera>,
        projection: &Mat4<T>,
        world_view: &mut Mat4<T>,
        [width, height]: [i32; 2],
        anchor: Option<&Point3>,
        to_apply_proj_pers: bool,
    ) -> Result<(), Failure>
    where
        T: Graphic3dTransformScalar + ElementCast<f64>,
        f64: ElementCast<T>,
    {
        if self.mode() == Flags::NONE || height == 0 {
            return Ok(());
        }
        let Some(source) = camera.get() else {
            return Err(program_error(
                "Graphic3d_TransformPers::Apply() - null camera; expected a camera",
            ));
        };
        let mut copy = Handle::new(source.copy_members());
        let mut result = self.compute_apply(&mut copy, width, height, anchor)?;
        if !source.is_orthographic() && self.is_ortho_pers() && to_apply_proj_pers {
            let proj_inv = Mat4::<f64>::from_other(&projection.inverted()?);
            result = (proj_inv * copy.projection_matrix()) * result;
        }
        world_view.convert_from(&result);
        Ok(())
    }
    /// `Compute`: the persistent transformation relative to the world-view matrix (identity
    /// for no persistence or a singular world view).
    /// Example: `pers.compute(&camera, &projection, &world_view, [800, 600], false)?`.
    fn compute<T>(
        &self,
        camera: &Handle<Graphic3dCamera>,
        projection: &Mat4<T>,
        world_view: &Mat4<T>,
        size: [i32; 2],
        to_apply_proj_pers: bool,
    ) -> Result<Mat4<T>, Failure>
    where
        T: Graphic3dTransformScalar + ElementCast<f64>,
        f64: ElementCast<T>,
        Mat4<T>: Mul<Output = Mat4<T>>,
    {
        let mut unview = Mat4::<T>::identity();
        if self.mode() == Flags::NONE || !world_view.inverted_to(&mut unview) {
            return Ok(Mat4::identity());
        }
        let mut persistent = *world_view;
        self.apply_world_view(
            camera,
            projection,
            &mut persistent,
            size,
            None,
            to_apply_proj_pers,
        )?;
        Ok(unview * persistent)
    }
    /// `Apply(..., BVH_Box<T, 3>&)`: the box of the corners moved by [`Self::compute`].
    /// Example: `pers.apply_bvh(&camera, &projection, &world_view, [800, 600], &mut bounds)?`.
    fn apply_bvh<T>(
        &self,
        camera: &Handle<Graphic3dCamera>,
        projection: &Mat4<T>,
        world_view: &Mat4<T>,
        size: [i32; 2],
        bounds: &mut BvhBox<Vec3<T>>,
    ) -> Result<(), Failure>
    where
        T: Graphic3dTransformScalar + ElementCast<f64>,
        f64: ElementCast<T>,
        Mat4<T>: Mul<Output = Mat4<T>>,
        Vec4<T>: Div<T, Output = Vec4<T>>,
        Vec3<T>: BvhVector,
    {
        let persistent = self.compute(camera, projection, world_view, size, false)?;
        if persistent.is_identity() || !bounds.is_valid() {
            return Ok(());
        }
        transform_box_corners(&persistent, bounds);
        Ok(())
    }
    /// `Apply(..., Bnd_Box&)`: the same on a non-void `Bnd_Box` (only the double instantiation
    /// compiles natively). Example: `pers.apply_bnd(&camera, &projection, &world_view, [800, 600], &mut b)?`.
    fn apply_bnd(
        &self,
        camera: &Handle<Graphic3dCamera>,
        projection: &Mat4<f64>,
        world_view: &Mat4<f64>,
        size: [i32; 2],
        bounds: &mut BoundingBox3,
    ) -> Result<(), Failure> {
        apply_on_bnd_box(bounds, |bvh| {
            self.apply_bvh(camera, projection, world_view, size, bvh)
        })
    }
}
impl<P: Graphic3dTransformPersSource + ?Sized> Graphic3dTransformPersApply for P {}
