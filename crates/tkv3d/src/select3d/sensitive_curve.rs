//! `Select3D_SensitiveCurve` (a sensitive polyline of sensitivity 3) and
//! `Select3D_SensitiveCircle` (an exact sensitive circle).
use super::{
    Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, Select3DSensitivePolyCore, Select3DTypeOfSensitivity,
};
use crate::impl_select3d_poly;
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::{Mat4, Vec3};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::bvh::BvhBox;
use rustycad_tkmath::gp::{Axis3, Circle3, Point3, Transform3, xoy};

/// A sensitive polyline of sensitivity 3.
///
/// Example: `let curve = Select3DSensitiveCurve::from_points(&owner, &points)?;`.
pub struct Select3DSensitiveCurve {
    transient: StandardTransient,
    /// The poly state.
    pub core: Select3DSensitivePolyCore,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveCurve, "Select3D_SensitiveCurve", dyn super::Select3DSensitiveSet, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn super::Select3DSensitiveSet]);
impl_select3d_poly!(Select3DSensitiveCurve, |curve: &Select3DSensitiveCurve| {
    let connected =
        Select3DSensitiveCurve::from_points(&curve.owner_id(), &curve.core.points_3d())?;
    Ok(Handle::new(connected).cast())
});

impl Select3DSensitiveCurve {
    /// A curve of a core, with the sensitivity factor 3.
    fn of(core: Select3DSensitivePolyCore) -> Self {
        core.entity.fields.write().sensitivity_factor = 3;
        Self {
            transient: StandardTransient::default(),
            core,
        }
    }
    /// `Select3D_SensitiveCurve(owner, Geom_Curve, nbPnts)`: `nb_pnts` evenly spaced points of
    /// the curve, with BVH. Example: `Select3DSensitiveCurve::from_curve(&owner, &*curve, 17)?`.
    pub fn from_curve(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        curve: &dyn GeomCurve,
        nb_pnts: i32,
    ) -> Result<Self, Failure> {
        let core = Select3DSensitivePolyCore::with_size(owner, true, nb_pnts)?;
        let step = (curve.last_parameter() - curve.first_parameter()) / f64::from(nb_pnts - 1);
        let mut param = curve.first_parameter();
        {
            let mut data = core.data.write();
            for index in 0..data.polyg.size() {
                data.polyg.set_pnt(index, &curve.value(param)?)?;
                param += step;
            }
        }
        Ok(Self::of(core))
    }
    /// `Select3D_SensitiveCurve(owner, points)`, with BVH; `Select3DSensitiveCurve::from_points(&owner, &points)?`.
    pub fn from_points(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        points: &[Point3],
    ) -> Result<Self, Failure> {
        Ok(Self::of(Select3DSensitivePolyCore::from_points(
            owner, points, true,
        )?))
    }
}

/// An exact sensitive circle, the circle of radius `radius` in `gp::XOY` placed by `trsf`.
///
/// Example: `let circle = Select3DSensitiveCircle::new(&owner, &circ, false);`.
pub struct Select3DSensitiveCircle {
    transient: StandardTransient,
    base: Select3DSensitiveEntityBase,
    sens_type: Select3DTypeOfSensitivity,
    trsf: Transform3,
    radius: f64,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveCircle, "Select3D_SensitiveCircle", dyn Select3DSensitiveEntity, transient,
    interfaces: [dyn Select3DSensitiveEntity]);

impl Select3DSensitiveCircle {
    /// A circle of an owner, filled or a boundary of sensitivity 6.
    /// Example: `Select3DSensitiveCircle::new(&owner, &circ, false)`.
    pub fn new(
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        circle: &Circle3,
        is_filled: bool,
    ) -> Self {
        let mut trsf = Transform3::default();
        trsf.set_transformation_between(
            Axis3::from_axis2(circle.position()),
            Axis3::from_axis2(xoy()),
        );
        let sens_type = if is_filled {
            Select3DTypeOfSensitivity::Interior
        } else {
            Select3DTypeOfSensitivity::Boundary
        };
        let entity = Self {
            transient: StandardTransient::default(),
            base: Select3DSensitiveEntityBase::new(owner),
            sens_type,
            trsf,
            radius: circle.radius(),
        };
        if sens_type == Select3DTypeOfSensitivity::Boundary {
            entity.base.fields.write().sensitivity_factor = 6;
        }
        entity
    }
    /// `Transformation`; `circle.transformation()`.
    pub fn transformation(&self) -> Transform3 {
        self.trsf
    }
    /// `Circle`: the circle of `gp::XOY` transformed; the radius check of `gp_Circ` cannot fail
    /// for a stored radius. Example: `circle.circle()?`.
    pub fn circle(&self) -> Result<Circle3, Failure> {
        Circle3::new(xoy().transformed(self.trsf)?, self.radius)
    }
    /// `Radius`; `circle.radius()`.
    pub fn radius(&self) -> f64 {
        self.radius
    }
}

impl Select3DSensitiveEntity for Select3DSensitiveCircle {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.base
    }
    /// `Matches`: as the sphere, with the filled flag of the sensitivity.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let is_filled = self.sens_type == Select3DTypeOfSensitivity::Interior;
        if manager.get_active_selection_type() != SelectMgrSelectionType::Point {
            if !manager.is_overlap_allowed() {
                let mut is_inside = true;
                return Ok(manager.overlaps_circle_inside(
                    self.radius,
                    &self.trsf,
                    is_filled,
                    Some(&mut is_inside),
                ) && is_inside);
            }
            return Ok(manager.overlaps_circle_inside(self.radius, &self.trsf, is_filled, None));
        }
        if !manager.overlaps_circle(self.radius, &self.trsf, is_filled, pick_result) {
            return Ok(false);
        }
        pick_result
            .set_dist_to_geom_center(manager.dist_to_geometry_center(&self.center_of_geometry()));
        Ok(true)
    }
    fn nb_sub_elements(&self) -> i32 {
        1
    }
    /// `BoundingBox`: the square of the radius in XOY, transformed.
    fn bounding_box(&self) -> Select3DBndBox3d {
        let mut matrix = Mat4::<f64>::default();
        self.trsf.get_mat4(&mut matrix);
        let mut bounds = BvhBox::from_corners(
            Vec3::new(-self.radius, -self.radius, 0.0),
            Vec3::new(self.radius, self.radius, 0.0),
        );
        bounds.transform(&matrix);
        bounds
    }
    fn center_of_geometry(&self) -> Point3 {
        Point3::from(self.trsf.translation_part())
    }
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let is_filled = self.sens_type == Select3DTypeOfSensitivity::Interior;
        Ok(Handle::new(Self::new(&self.owner_id(), &self.circle()?, is_filled)).cast())
    }
    fn to_build_bvh(&self) -> bool {
        false
    }
}
