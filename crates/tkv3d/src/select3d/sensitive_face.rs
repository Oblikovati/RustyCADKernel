//! `Select3D_SensitiveFace`: a sensitive polygon, its interior (planar parts) or its boundary
//! (a poly with BVH).
use super::{
    Select3DBndBox3d, Select3DInteriorSensitivePointSet, Select3DSensitiveEntity,
    Select3DSensitiveEntityBase, Select3DSensitiveEntityMembers as _, Select3DSensitivePoly,
    Select3DTypeOfSensitivity, select3d_sensitive_entity_dump_json,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::json_dump::{base_class, class_begin, field_dumped, field_integer};
use std::io;

/// The sensitive set of a face (`myFacePoints`), by sensitivity.
#[derive(Clone)]
enum FacePoints {
    Interior(Handle<Select3DInteriorSensitivePointSet>),
    Boundary(Handle<Select3DSensitivePoly>),
}

impl FacePoints {
    /// The set as a sensitive entity.
    fn entity(&self) -> Handle<dyn Select3DSensitiveEntity> {
        match self {
            Self::Interior(set) => set.clone().cast(),
            Self::Boundary(poly) => poly.clone().cast(),
        }
    }
}

/// A sensitive face.
///
/// Example: `let face = Select3DSensitiveFace::new(&owner, &points, Select3DTypeOfSensitivity::Interior)?;`.
pub struct Select3DSensitiveFace {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    sens_type: Select3DTypeOfSensitivity,
    face_points: FacePoints,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveFace, "Select3D_SensitiveFace", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl Select3DSensitiveFace {
    /// A face of an owner: an interior point set or a boundary poly with BVH; no point raises
    /// `Standard_ConstructionError` for a boundary. Example: `Select3DSensitiveFace::new(&owner, &points, kind)?`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        points: &[Point3],
        sens_type: Select3DTypeOfSensitivity,
    ) -> Result<Self, Failure> {
        let face_points = match sens_type {
            Select3DTypeOfSensitivity::Interior => FacePoints::Interior(Handle::new(
                Select3DInteriorSensitivePointSet::new(owner, points)?,
            )),
            Select3DTypeOfSensitivity::Boundary => FacePoints::Boundary(Handle::new(
                Select3DSensitivePoly::new(owner, points, true)?,
            )),
        };
        Ok(Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            sens_type,
            face_points,
        })
    }
    /// `GetPoints`: the points of the boundary poly or of the interior set; `face.get_points()`.
    pub fn get_points(&self) -> Vec<Point3> {
        match &self.face_points {
            FacePoints::Interior(set) => set.get_points(),
            FacePoints::Boundary(poly) => poly.points_3d(),
        }
    }
    /// The inner set as an entity.
    fn inner(&self) -> Handle<dyn Select3DSensitiveEntity> {
        self.face_points.entity()
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveFace {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.inner()
            .get()
            .map_or(Ok(false), |e| e.matches(manager, pick_result))
    }
    fn nb_sub_elements(&self) -> i32 {
        self.inner().get().map_or(0, |e| e.nb_sub_elements())
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        self.inner()
            .get()
            .map(|e| e.bounding_box())
            .unwrap_or_default()
    }
    fn center_of_geometry(&self) -> Point3 {
        self.inner()
            .get()
            .map(|e| e.center_of_geometry())
            .unwrap_or_default()
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        Ok(Handle::new(Self::new(
            &self.owner_id(),
            &self.get_points(),
            self.sens_type,
        )?)
        .cast())
    }
    fn bvh(&self) -> Result<(), Failure> {
        self.inner().get().map_or(Ok(()), |e| e.bvh())
    }
    fn to_build_bvh(&self) -> bool {
        self.inner().get().is_some_and(|e| e.to_build_bvh())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitiveFace")?;
        base_class(stream, depth, &|s, d| {
            select3d_sensitive_entity_dump_json(self, s, d)
        })?;
        field_integer(stream, "mySensType", self.sens_type as i32)?;
        let inner = self.inner();
        if let Some(entity) = inner.get() {
            field_dumped(stream, depth, "myFacePoints.get()", &|s, d| {
                entity.dump_json(s, d)
            })?;
        }
        Ok(())
    }
}
