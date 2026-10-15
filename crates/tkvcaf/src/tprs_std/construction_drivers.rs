// Rust port of ApplicationFramework/TKVCAF/TPrsStd/TPrsStd_AxisDriver, TPrsStd_PlaneDriver
// and TPrsStd_PointDriver (.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The drivers of the construction attributes: `TDataXtd_Axis` shows as an `AIS_Axis`,
//! `TDataXtd_Plane` as an `AIS_Plane` and `TDataXtd_Point` as an `AIS_Point`. An object of
//! the right type is updated in place; any other object is replaced.
use super::TPrsStdDriver;
use rustycad_tkcaf::tdata_xtd::{TDataXtdAxis, TDataXtdGeometry, TDataXtdPlane, TDataXtdPoint};
use rustycad_tkcaf::tnaming::{TNamingNamedShape, TNamingTool};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg3d::geom::{GeomCartesianPoint, GeomLine, GeomPlane};
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkmath::gp::{Line3, Plane3, Point3};
use rustycad_tkv3d::ais::{AisAxis, AisInteractiveObject, AisPlane, AisPoint};
use rustycad_tkv3d::prs_dim::prs_dim_units_api;
use rustycad_tkv3d::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use rustycad_tkv3d::select_mgr::SelectMgrSelectableObjectMembers;

/// Declares a stateless `TPrsStd_Driver` subclass with its RTTI and default constructor.
macro_rules! presentation_driver {
    ($(#[$doc:meta])* $name:ident, $occt:literal) => {
        $(#[$doc])*
        #[derive(Default)]
        pub struct $name {
            transient: rustycad_tkernel::standard::StandardTransient,
        }
        rustycad_tkernel::impl_transient!($name, $occt, dyn $crate::tprs_std::TPrsStdDriver,
            transient, interfaces: [dyn $crate::tprs_std::TPrsStdDriver]);
        impl $name {
            #[doc = concat!("`", $occt, "()`.")]
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}
pub(super) use presentation_driver;

presentation_driver!(
    /// `TPrsStd_AxisDriver`: the `AIS_Axis` of a `TDataXtd_Axis`.
    TPrsStdAxisDriver,
    "TPrsStd_AxisDriver"
);
presentation_driver!(
    /// `TPrsStd_PlaneDriver`: the `AIS_Plane` of a `TDataXtd_Plane`.
    TPrsStdPlaneDriver,
    "TPrsStd_PlaneDriver"
);
presentation_driver!(
    /// `TPrsStd_PointDriver`: the `AIS_Point` of a `TDataXtd_Point`.
    TPrsStdPointDriver,
    "TPrsStd_PointDriver"
);

/// True when the label holds the attribute of that ID.
pub(super) fn label_has<T: RuntimeTyped + ?Sized>(
    label: &TdfLabel,
    id: &rustycad_tkernel::standard::Guid,
) -> Result<bool, Failure> {
    let mut found: Handle<T> = Handle::default();
    label.find_attribute_as(id, &mut found)
}

/// The update step every driver applies to an object it keeps: `ResetTransformation`,
/// `SetToUpdate` (all modes) and `UpdateSelection` (all modes).
pub(super) fn refresh_kept_object<
    O: PrsMgrPresentableObject + SelectMgrSelectableObjectMembers + ?Sized,
>(
    object: &O,
) -> Result<(), Failure> {
    object.reset_transformation()?;
    object.set_to_update_all();
    object.update_selection(-1);
    Ok(())
}

impl TPrsStdDriver for TPrsStdAxisDriver {
    /// `Update`: the label's line (false when its named shape is empty).
    fn update(
        &self,
        label: &TdfLabel,
        ais: &mut Handle<dyn AisInteractiveObject>,
    ) -> Result<bool, Failure> {
        if !label_has::<TDataXtdAxis>(label, &TDataXtdAxis::get_id())? {
            return Ok(false);
        }
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        if label.find_attribute_as(&TNamingNamedShape::get_id(), &mut named)?
            && TNamingTool::get_shape(&named)?.is_null()
        {
            return Ok(false);
        }
        let mut line = Line3::default();
        if !TDataXtdGeometry::line_of_label(label, &mut line)? {
            return Ok(false);
        }
        let geometry = Handle::new(GeomLine::from_lin(line));
        let kept = ais.cast::<AisAxis>();
        let axis = match kept.get() {
            Some(axis) => {
                axis.set_component(&geometry)?;
                refresh_kept_object(axis)?;
                kept.clone()
            }
            None => AisAxis::new(&geometry, prs_dim_units_api())?,
        };
        *ais = axis.cast();
        Ok(true)
    }
}

impl TPrsStdDriver for TPrsStdPlaneDriver {
    /// `Update`: the label's plane, centered at its location.
    fn update(
        &self,
        label: &TdfLabel,
        ais: &mut Handle<dyn AisInteractiveObject>,
    ) -> Result<bool, Failure> {
        if !label_has::<TDataXtdPlane>(label, &TDataXtdPlane::get_id())? {
            return Ok(false);
        }
        let mut plane = Plane3::default();
        if !TDataXtdGeometry::plane_of_label(label, &mut plane)? {
            return Ok(false);
        }
        let geometry = Handle::new(GeomPlane::from_pln(plane));
        let kept = ais.cast::<AisPlane>();
        let object = match kept.get() {
            Some(object) => {
                object.set_component(&geometry);
                object.set_center(plane.location());
                refresh_kept_object(object)?;
                kept.clone()
            }
            None => AisPlane::with_center(&geometry, plane.location(), false)?,
        };
        *ais = object.cast();
        Ok(true)
    }
}

impl TPrsStdDriver for TPrsStdPointDriver {
    /// `Update`: the label's point.
    fn update(
        &self,
        label: &TdfLabel,
        ais: &mut Handle<dyn AisInteractiveObject>,
    ) -> Result<bool, Failure> {
        if !label_has::<TDataXtdPoint>(label, &TDataXtdPoint::get_id())? {
            return Ok(false);
        }
        let mut point = Point3::default();
        if !TDataXtdGeometry::point_of_label(label, &mut point)? {
            return Ok(false);
        }
        *ais = point_object(ais, point)?.cast();
        Ok(true)
    }
}

/// The `AIS_Point` of a point: the kept object updated, or a new one.
pub(super) fn point_object(
    ais: &Handle<dyn AisInteractiveObject>,
    point: Point3,
) -> Result<Handle<AisPoint>, Failure> {
    let geometry = Handle::new(GeomCartesianPoint::from_pnt(point)).cast();
    let kept = ais.cast::<AisPoint>();
    match kept.get() {
        Some(object) => {
            object.set_component(&geometry);
            refresh_kept_object(object)?;
            Ok(kept.clone())
        }
        None => AisPoint::new(&geometry),
    }
}
