//! Martensite's overridable AIS_PointCloud geometry contract at OCCT 3d097a0.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::{prs_mgr::*, prs3d::Prs3dPointAspect};
use rustycad_tkernel::{
    ncollection::HArray1,
    quantity::{Color, NamedColor},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkmath::{
    bnd::BoundingBox3,
    gp::{Direction3, Point3},
};
use rustycad_tkservice::{aspect::AspectTypeOfMarker, graphic3d::*};
/// Native display modes; `AisPointCloudDisplayMode::Points as i32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AisPointCloudDisplayMode {
    /// Normal point presentation.
    Points = 0,
    /// Bounding box presentation used for whole-object highlighting.
    BndBox = 2,
}
/// Native selection modes; `AisPointCloudSelectionMode::SubsetOfPoints as i32`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AisPointCloudSelectionMode {
    /// Point detection with a whole-object owner.
    Points = 0,
    /// Point detection with a forced subset owner.
    SubsetOfPoints = 1,
    /// Bounding box detection.
    BndBox = 2,
}
/// Native retained buffer and cached bounds; `cloud.point_cloud_base().fields.read().bounds`.
#[derive(Clone, Default)]
pub struct AisPointCloudFields {
    /// Aliased Graphic3d point buffer.
    pub points: Handle<Graphic3dArrayOfPoints>,
    /// Bounds cached on SetPoints, independent of subsequent buffer edits.
    pub bounds: BoundingBox3,
}
/// State embedded by derived point clouds; `AisPointCloudBase::new()?`.
pub struct AisPointCloudBase {
    /// Inherited interactive state.
    pub interactive: AisInteractiveObjectBase,
    /// Native buffer storage and cached bounds.
    pub fields: SharedState<AisPointCloudFields>,
}
impl AisPointCloudBase {
    /// Initialize retained geometry and native shading defaults; `AisPointCloudBase::new()?`.
    pub fn new() -> Result<Self, Failure> {
        let interactive = AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView);
        initialize_cloud_drawers(&interactive.presentable)?;
        Ok(Self {
            interactive,
            fields: SharedState::default(),
        })
    }
    /// Call the native base buffer setter from a derived virtual override; `base.set_points(&points)?`.
    pub fn set_points(&self, points: &Handle<Graphic3dArrayOfPoints>) -> Result<(), Failure> {
        self.fields.write().points = points.clone();
        let bounds = super::point_cloud_geometry::point_cloud_bounds(points)?;
        self.fields.write().bounds = bounds;
        Ok(())
    }
    /// Call the native base copying setter from a derived override; `base.set_points_arrays(&coords, &colors, &normals)?`.
    pub fn set_points_arrays(
        &self,
        coords: &Handle<HArray1<Point3>>,
        colors: &Handle<HArray1<Color>>,
        normals: &Handle<HArray1<Direction3>>,
    ) -> Result<(), Failure> {
        *self.fields.write() = AisPointCloudFields::default();
        let points = super::point_cloud_geometry::copy_point_cloud_arrays(coords, colors, normals)?;
        let bounds = super::point_cloud_geometry::point_cloud_bounds(&points)?;
        *self.fields.write() = AisPointCloudFields { points, bounds };
        Ok(())
    }
}
/// Virtual geometry hooks for application-supplied clouds; `cloud.get_points()`.
pub trait AisPointCloudSource: AisInteractiveObject {
    /// Embedded native state; `cloud.point_cloud_base()`.
    fn point_cloud_base(&self) -> &AisPointCloudBase;
    /// Alias the supplied buffer and refresh cached bounds without scheduling updates; `cloud.set_points(&points)?`.
    fn set_points(&self, points: &Handle<Graphic3dArrayOfPoints>) -> Result<(), Failure> {
        self.point_cloud_base().set_points(points)
    }
    /// Copy coordinate, color and normal arrays by offset, silently clearing mismatched input; `cloud.set_points_arrays(&coords, &colors, &normals)?`.
    fn set_points_arrays(
        &self,
        coords: &Handle<HArray1<Point3>>,
        colors: &Handle<HArray1<Color>>,
        normals: &Handle<HArray1<Direction3>>,
    ) -> Result<(), Failure> {
        self.point_cloud_base()
            .set_points_arrays(coords, colors, normals)
    }
    /// Return the retained buffer or an application's live replacement; `cloud.get_points()`.
    fn get_points(&self) -> Handle<Graphic3dArrayOfPoints> {
        self.point_cloud_base().fields.read().points.clone()
    }
    /// Return cached bounds or an application's live bounds; `cloud.get_bounding_box()`.
    fn get_bounding_box(&self) -> BoundingBox3 {
        self.point_cloud_base().fields.read().bounds.clone()
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn AisPointCloudSource,
    "AIS_PointCloud",
    dyn AisInteractiveObject
);
/// A retained point cloud with native subset selection; `AisPointCloud::new()?`.
pub struct AisPointCloud {
    transient: StandardTransient,
    base: AisPointCloudBase,
}
rustycad_tkernel::impl_transient!(AisPointCloud,"AIS_PointCloud",dyn AisInteractiveObject,transient,
    interfaces:[dyn AisPointCloudSource,dyn AisInteractiveObject,dyn crate::select_mgr::SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisPointCloud {
    /// Construct and bind a cloud with native defaults; `AisPointCloud::new()?`.
    pub fn new() -> Result<Handle<Self>, Failure> {
        let cloud = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisPointCloudBase::new()?,
        });
        prs_mgr_presentable_bind(&cloud.cast());
        Ok(cloud)
    }
}
impl AisPointCloudSource for AisPointCloud {
    fn point_cloud_base(&self) -> &AisPointCloudBase {
        &self.base
    }
}
impl AisInteractiveObject for AisPointCloud {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.interactive
    }
}

fn initialize_cloud_drawers(base: &PrsMgrPresentableObjectBase) -> Result<(), Failure> {
    let drawer = base.read().drawer;
    drawer.setup_own_shading_aspect(&Handle::default())?;
    drawer
        .shading_aspect()
        .aspect()
        .set_marker_type(AspectTypeOfMarker::Point);
    drawer.set_display_mode(0);
    prs_mgr_initialize_hilight_mode(base, 2);
    let dynamic = Prs3dPointAspect::new(
        AspectTypeOfMarker::Plus,
        Color::named(NamedColor::CYAN1),
        1.,
    )?;
    base.read()
        .dyn_hilight_drawer
        .set_point_aspect(Handle::new(dynamic));
    Ok(())
}
