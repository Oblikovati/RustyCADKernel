//! Real primitive-array detection with the native point-only, near-tangent ray guard.
use super::AisViewCubeOwner;
use crate::{select_basics::*, select_mgr::*, select3d::*, v3d::V3d};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::{
    gp::{GeneralTransform3, Point3},
    toploc::Location,
};
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
/// Native sensitive ViewCube facets; `AisViewCubeSensitive::new(&owner,&triangles)?`.
pub struct AisViewCubeSensitive {
    transient: StandardTransient,
    /// Retained real sensitive primitive array; its buffer and owner identities are shared.
    pub primitive: Select3DSensitivePrimitiveArray,
}
rustycad_tkernel::impl_transient!(AisViewCubeSensitive,"AIS_ViewCubeSensitive",Select3DSensitivePrimitiveArray,transient,
    interfaces:[dyn Select3DSensitiveEntity]);
impl AisViewCubeSensitive {
    /// Initialize native indexed triangles without copying their buffers; `AisViewCubeSensitive::new(&owner,&triangles)?`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        triangles: &Handle<Graphic3dArrayOfTriangles>,
    ) -> Result<Self, Failure> {
        let triangles = triangles.get().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::null_object(
                "AIS_ViewCubeSensitive triangles=null; expected a real triangle array",
            )
        })?;
        let primitive = Select3DSensitivePrimitiveArray::new(owner)?;
        primitive.init_triangulation_all(
            &triangles.attributes(),
            &triangles.indices(),
            &Location::default(),
            true,
            1,
        )?;
        Ok(Self {
            transient: StandardTransient::default(),
            primitive,
        })
    }
    /// Reject rectangular selection and rays within ten degrees of a part tangent; `entity.is_valid_ray(&manager)?`.
    pub fn is_valid_ray(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<bool, Failure> {
        if manager.get_active_selection_type() != SelectMgrSelectionType::Point {
            return Ok(false);
        }
        let owner = self.owner_id().cast::<AisViewCubeOwner>();
        let Some(owner) = owner.get() else {
            return Ok(true);
        };
        let ray = manager.get_view_ray_direction()?;
        Ok(!ray.is_normal(
            V3d::get_proj_axis(owner.main_orientation()),
            10.0_f64.to_radians(),
        ))
    }
}
impl Select3DSensitiveEntity for AisViewCubeSensitive {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        self.primitive.sensitive_base()
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if !self.is_valid_ray(manager)? {
            return Ok(false);
        }
        self.primitive.matches(manager, pick)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.primitive.nb_sub_elements()
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.primitive.bounding_box()
    }
    fn center_of_geometry(&self) -> Point3 {
        self.primitive.center_of_geometry()
    }
    fn set(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
        self.primitive.set(owner);
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        // Native inherits the base method: a connected copy intentionally loses ViewCube's ray filter.
        self.primitive.get_connected()
    }
    fn bvh(&self) -> Result<(), Failure> {
        self.primitive.bvh()
    }
    fn to_build_bvh(&self) -> bool {
        self.primitive.to_build_bvh()
    }
    fn clear(&self) {
        self.primitive.clear();
    }
    fn has_init_location(&self) -> bool {
        self.primitive.has_init_location()
    }
    fn inv_init_location(&self) -> GeneralTransform3 {
        self.primitive.inv_init_location()
    }
}
