//! `Select3D_SensitiveCurve`: sensitive polyline approximation with sensitivity 3.
use super::{
    Select3DSensitiveEntity, Select3DSensitiveEntityMembers as _, Select3DSensitivePolyCore,
};
use crate::impl_select3d_poly;
use crate::select_mgr::SelectMgrEntityOwnerSource;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkmath::gp::Point3;

/// A sensitive polyline of sensitivity 3.
///
/// Example: `let curve = Select3DSensitiveCurve::from_points(&owner, &points)?;`.
pub struct Select3DSensitiveCurve {
    transient: StandardTransient,
    /// The poly state.
    pub core: Select3DSensitivePolyCore,
}
rustycad_tkernel::impl_transient!(Select3DSensitiveCurve, "Select3D_SensitiveCurve", dyn super::Select3DSensitivePolySource, transient,
    interfaces: [dyn Select3DSensitiveEntity, dyn super::Select3DSensitiveSet, dyn super::Select3DSensitivePolySource]);
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
