// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_DummySensitiveEntity`, `MeshVS_SensitiveFace` and `MeshVS_SensitiveSegment` at
//! OCCT 3d097a0.
//!
//! The face and the segment derive natively from the concrete `Select3D_SensitiveFace` and
//! `Select3D_SensitiveSegment` without overriding any virtual member: they embed that base
//! (also as their runtime-type parent, so a down cast to the base type finds it) and forward
//! every member to it.
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Axis2, GeneralTransform3, Point3, Transform3, Xyz};
use rustycad_tkservice::graphic3d::Graphic3dTransformPersSource;
use rustycad_tkv3d::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkv3d::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkv3d::select3d::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase, Select3DSensitiveFace,
    Select3DSensitiveSegment, Select3DTypeOfSensitivity,
};
use std::io;

/// The entity of an owner that must not be detected; `MeshVSDummySensitiveEntity::new(&owner)`.
pub struct MeshVSDummySensitiveEntity {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
}
rustycad_tkernel::impl_transient!(MeshVSDummySensitiveEntity, "MeshVS_DummySensitiveEntity", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl MeshVSDummySensitiveEntity {
    /// `MeshVS_DummySensitiveEntity(theOwnerId)`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
        }
    }
}

impl Select3DSensitiveEntity for MeshVSDummySensitiveEntity {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    /// `Matches`: never.
    fn matches(
        &self,
        _: &dyn SelectBasicsSelectingVolumeManager,
        _: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    /// `NbSubElements`: -1.
    fn nb_sub_elements(&self) -> i32 {
        -1
    }
    /// `BoundingBox`: the invalid box.
    fn bounding_box(&self) -> Select3DBndBox3d {
        Select3DBndBox3d::default()
    }
    /// `CenterOfGeometry`: the origin.
    fn center_of_geometry(&self) -> Point3 {
        Point3::default()
    }
    /// `BVH`: nothing.
    fn bvh(&self) -> Result<(), Failure> {
        Ok(())
    }
    /// `ToBuildBVH`: false.
    fn to_build_bvh(&self) -> bool {
        false
    }
    /// `Clear`: nothing (the owner stays).
    fn clear(&self) {}
    /// `HasInitLocation`: false.
    fn has_init_location(&self) -> bool {
        false
    }
    /// `InvInitLocation`: the identity.
    fn inv_init_location(&self) -> GeneralTransform3 {
        GeneralTransform3::from(Transform3::default())
    }
}

/// Forwards every member of `Select3DSensitiveEntity` to the embedded base entity `$field`.
macro_rules! delegate_sensitive_entity {
    ($type:ty, $field:ident) => {
        impl Select3DSensitiveEntity for $type {
            fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
                self.$field.sensitive_base()
            }
            fn matches(
                &self,
                manager: &dyn SelectBasicsSelectingVolumeManager,
                pick: &mut SelectBasicsPickResult,
            ) -> Result<bool, Failure> {
                self.$field.matches(manager, pick)
            }
            fn nb_sub_elements(&self) -> i32 {
                self.$field.nb_sub_elements()
            }
            fn bounding_box(&self) -> Select3DBndBox3d {
                self.$field.bounding_box()
            }
            fn center_of_geometry(&self) -> Point3 {
                self.$field.center_of_geometry()
            }
            fn set(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
                self.$field.set(owner);
            }
            fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
                self.$field.get_connected()
            }
            fn bvh(&self) -> Result<(), Failure> {
                self.$field.bvh()
            }
            fn to_build_bvh(&self) -> bool {
                self.$field.to_build_bvh()
            }
            fn clear(&self) {
                self.$field.clear();
            }
            fn has_init_location(&self) -> bool {
                self.$field.has_init_location()
            }
            fn inv_init_location(&self) -> GeneralTransform3 {
                self.$field.inv_init_location()
            }
            fn set_transform_persistence(&self, pers: &Handle<dyn Graphic3dTransformPersSource>) {
                self.$field.set_transform_persistence(pers);
            }
            fn set_flipping_options(&self, is_enabled: bool, ref_plane: &Axis2) {
                self.$field.set_flipping_options(is_enabled, ref_plane);
            }
            fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
                self.$field.dump_json(stream, depth)
            }
        }
    };
}

/// A polygon face of a mesh; `MeshVSSensitiveFace::new(&owner, &points, Select3DTypeOfSensitivity::Interior)?`.
pub struct MeshVSSensitiveFace {
    face: Select3DSensitiveFace,
    center: Point3,
}
rustycad_tkernel::impl_transient!(MeshVSSensitiveFace, "MeshVS_SensitiveFace", Select3DSensitiveFace, face,
    interfaces: [dyn Select3DSensitiveEntity]);
delegate_sensitive_entity!(MeshVSSensitiveFace, face);

impl MeshVSSensitiveFace {
    /// `MeshVS_SensitiveFace(theOwner, thePoints, theSensType)`: the base face and the mean
    /// of the points (no point divides by zero).
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        points: &[Point3],
        sens_type: Select3DTypeOfSensitivity,
    ) -> Result<Self, Failure> {
        let mut sum = Xyz::new(0.0, 0.0, 0.0);
        for point in points {
            sum += point.xyz();
        }
        Ok(Self {
            face: Select3DSensitiveFace::new(owner, points, sens_type)?,
            center: Point3::from(sum / points.len() as f64),
        })
    }
    /// The protected `myCenter`; `face.center()`.
    pub fn center(&self) -> Point3 {
        self.center
    }
    /// The embedded `Select3D_SensitiveFace`; `face.base_face().get_points()`.
    pub fn base_face(&self) -> &Select3DSensitiveFace {
        &self.face
    }
}

/// A link of a mesh; `MeshVSSensitiveSegment::new(&owner, a, b)`.
pub struct MeshVSSensitiveSegment {
    segment: Select3DSensitiveSegment,
}
rustycad_tkernel::impl_transient!(MeshVSSensitiveSegment, "MeshVS_SensitiveSegment", Select3DSensitiveSegment, segment,
    interfaces: [dyn Select3DSensitiveEntity]);
delegate_sensitive_entity!(MeshVSSensitiveSegment, segment);

impl MeshVSSensitiveSegment {
    /// `MeshVS_SensitiveSegment(theOwner, theFirstPnt, theLastPnt)`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        first: Point3,
        last: Point3,
    ) -> Self {
        Self {
            segment: Select3DSensitiveSegment::new(owner, first, last),
        }
    }
    /// The embedded `Select3D_SensitiveSegment`; `segment.base_segment()`.
    pub fn base_segment(&self) -> &Select3DSensitiveSegment {
        &self.segment
    }
}
