// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_SensitiveQuad` at OCCT 3d097a0: a quadrangle detected as the triangles (0,1,2)
//! and (0,2,3); an inclusion test asks every corner (or, for a polyline, both triangles).
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Point3, Xyz};
use rustycad_tkv3d::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkv3d::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkv3d::select3d::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers, Select3DTypeOfSensitivity,
};

/// A sensitive quadrangle; `MeshVSSensitiveQuad::new(&owner, [a, b, c, d])`.
pub struct MeshVSSensitiveQuad {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    vertices: [Point3; 4],
}
rustycad_tkernel::impl_transient!(MeshVSSensitiveQuad, "MeshVS_SensitiveQuad", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl MeshVSSensitiveQuad {
    /// `MeshVS_SensitiveQuad(theOwner, thePnt1, thePnt2, thePnt3, thePnt4)`.
    pub fn new(owner: &Handle<dyn SelectMgrEntityOwnerSource>, vertices: [Point3; 4]) -> Self {
        Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            vertices,
        }
    }
    /// `MeshVS_SensitiveQuad(theOwner, theQuadVerts)`: the first four points of the array.
    /// Example: `MeshVSSensitiveQuad::from_points(&owner, &points)?`.
    pub fn from_points(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        points: &[Point3],
    ) -> Result<Self, Failure> {
        let corners: [Point3; 4] = points
            .get(..4)
            .and_then(|slice| slice.try_into().ok())
            .ok_or_else(|| {
                Failure::with_type(
                    "Standard_OutOfRange",
                    &format!(
                        "MeshVS_SensitiveQuad points={}: expected at least 4",
                        points.len()
                    ),
                    "",
                )
            })?;
        Ok(Self::new(owner, corners))
    }
    /// The four corners; `quad.vertices()[0]`.
    pub fn vertices(&self) -> [Point3; 4] {
        self.vertices
    }
    fn triangle(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        second: usize,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let v = &self.vertices;
        manager.overlaps_triangle(
            &v[0],
            &v[second],
            &v[second + 1],
            Select3DTypeOfSensitivity::Interior as i32,
            pick,
        )
    }
    fn is_inside(&self, manager: &dyn SelectBasicsSelectingVolumeManager) -> Result<bool, Failure> {
        if manager.get_active_selection_type() == SelectMgrSelectionType::Polyline {
            let mut dummy = SelectBasicsPickResult::default();
            return Ok(
                self.triangle(manager, 1, &mut dummy)? && self.triangle(manager, 2, &mut dummy)?
            );
        }
        for vertex in &self.vertices {
            if !manager.overlaps_point_only(vertex)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl Select3DSensitiveEntity for MeshVSSensitiveQuad {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if !manager.is_overlap_allowed() {
            return self.is_inside(manager);
        }
        let (mut first, mut second) = (
            SelectBasicsPickResult::default(),
            SelectBasicsPickResult::default(),
        );
        // Both operands of the native `&&` short-circuit: the second runs only when the first misses.
        if !self.triangle(manager, 1, &mut first)? && !self.triangle(manager, 2, &mut second)? {
            return Ok(false);
        }
        *pick = *SelectBasicsPickResult::min(&first, &second);
        pick.set_dist_to_geom_center(manager.dist_to_geometry_center(&self.center_of_geometry())?);
        Ok(true)
    }
    /// `NbSubElements`: 1.
    fn nb_sub_elements(&self) -> i32 {
        1
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        let mut bounds = Select3DBndBox3d::default();
        for v in &self.vertices {
            bounds.add(&Vec3::new(v.x(), v.y(), v.z()));
        }
        bounds
    }
    /// `CenterOfGeometry`: the mean of the corners.
    fn center_of_geometry(&self) -> Point3 {
        let mut sum = Xyz::new(0.0, 0.0, 0.0);
        for v in &self.vertices {
            sum += v.xyz();
        }
        Point3::from(sum / 4.0)
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        Ok(Handle::new(Self::new(&self.owner_id(), self.vertices)).cast())
    }
}
