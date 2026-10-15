// Rust port of ApplicationFramework/TKVCAF/TPrsStd/TPrsStd_GeometryDriver and
// TPrsStd_NamedShapeDriver (.hxx and .cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TPrsStd_GeometryDriver` (the geometry of a `TDataXtd_Geometry` or of a named shape as
//! an `AIS_Point`, `AIS_Line`, `AIS_Circle` or the `AIS_Shape` of an ellipse edge) and
//! `TPrsStd_NamedShapeDriver` (the `AIS_Shape` of a named shape).
use super::TPrsStdDriver;
use super::construction_drivers::{point_object, presentation_driver, refresh_kept_object};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkcaf::tdata_xtd::{TDataXtdGeometry, TDataXtdGeometryEnum};
use rustycad_tkcaf::tnaming::{TNamingNamedShape, TNamingTool};
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomLine};
use rustycad_tklcaf::tdf::TdfLabel;
use rustycad_tkmath::gp::{Circle3, Ellipse3, Line3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeShape};
use rustycad_tkv3d::ais::{AisCircle, AisInteractiveObject, AisLine, AisShape, AisShapeParameters};
use rustycad_tkv3d::prs_dim::prs_dim_units_api;
use rustycad_tkv3d::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use rustycad_tkv3d::select_mgr::SelectMgrSelectableObjectMembers;

presentation_driver!(
    /// `TPrsStd_GeometryDriver`: the presentation of a point, line, circle or ellipse.
    TPrsStdGeometryDriver,
    "TPrsStd_GeometryDriver"
);
presentation_driver!(
    /// `TPrsStd_NamedShapeDriver`: the `AIS_Shape` of the label's named shape.
    TPrsStdNamedShapeDriver,
    "TPrsStd_NamedShapeDriver"
);

/// The red of every geometry presentation.
fn red() -> Color {
    Color::named(NamedColor::RED)
}

impl TPrsStdGeometryDriver {
    /// The geometry type: the `TDataXtd_Geometry`'s, else the named shape's; `None`
    /// without either attribute.
    fn geometry_type(label: &TdfLabel) -> Result<Option<TDataXtdGeometryEnum>, Failure> {
        let mut geometry: Handle<TDataXtdGeometry> = Handle::default();
        if label.find_attribute_as(&TDataXtdGeometry::get_id(), &mut geometry)? {
            return Ok(Some(
                rustycad_tklcaf::live(&geometry, "TPrsStd_GeometryDriver")?.get_type(),
            ));
        }
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        if label.find_attribute_as(&TNamingNamedShape::get_id(), &mut named)? {
            return Ok(Some(TDataXtdGeometry::type_of_label(label)?));
        }
        Ok(None)
    }
}

impl TPrsStdDriver for TPrsStdGeometryDriver {
    /// `Update`: red point, red infinite line, red circle or the red edge of an ellipse.
    fn update(
        &self,
        label: &TdfLabel,
        ais: &mut Handle<dyn AisInteractiveObject>,
    ) -> Result<bool, Failure> {
        let Some(kind) = Self::geometry_type(label)? else {
            return Ok(false);
        };
        let object = match kind {
            TDataXtdGeometryEnum::Point => point_presentation(label, ais)?,
            TDataXtdGeometryEnum::Line => line_presentation(label, ais)?,
            TDataXtdGeometryEnum::Circle => circle_presentation(label, ais)?,
            TDataXtdGeometryEnum::Ellipse => ellipse_presentation(label, ais)?,
            _ => None,
        };
        let Some(object) = object else {
            return Ok(false);
        };
        *ais = object;
        Ok(true)
    }
}

/// `TDataXtd_POINT`: a red `AIS_Point`.
fn point_presentation(
    label: &TdfLabel,
    ais: &Handle<dyn AisInteractiveObject>,
) -> Result<Option<Handle<dyn AisInteractiveObject>>, Failure> {
    let mut point = Point3::default();
    if !TDataXtdGeometry::point_of_label(label, &mut point)? {
        return Ok(None);
    }
    let object: Handle<dyn AisInteractiveObject> = point_object(ais, point)?.cast();
    rustycad_tklcaf::live(&object, "TPrsStd_GeometryDriver")?.set_color(red())?;
    Ok(Some(object))
}

/// `TDataXtd_LINE`: a red infinite `AIS_Line`.
fn line_presentation(
    label: &TdfLabel,
    ais: &Handle<dyn AisInteractiveObject>,
) -> Result<Option<Handle<dyn AisInteractiveObject>>, Failure> {
    let mut line = Line3::default();
    if !TDataXtdGeometry::line_of_label(label, &mut line)? {
        return Ok(None);
    }
    let geometry = Handle::new(GeomLine::from_lin(line));
    let kept = ais.cast::<AisLine>();
    let object = match kept.get() {
        Some(object) => {
            object.set_line(&geometry);
            refresh_kept_object(object)?;
            kept.clone()
        }
        None => AisLine::new(&geometry, prs_dim_units_api()),
    };
    object.set_color(red())?;
    object.set_infinite_state(true);
    Ok(Some(object.cast()))
}

/// `TDataXtd_CIRCLE`: a red `AIS_Circle`.
fn circle_presentation(
    label: &TdfLabel,
    ais: &Handle<dyn AisInteractiveObject>,
) -> Result<Option<Handle<dyn AisInteractiveObject>>, Failure> {
    let mut circle = Circle3::default();
    if !TDataXtdGeometry::circle_of_label(label, &mut circle)? {
        return Ok(None);
    }
    let geometry = Handle::new(GeomCircle::from_circ(circle));
    let kept = ais.cast::<AisCircle>();
    let object = match kept.get() {
        Some(object) => {
            object.set_circle(&geometry);
            refresh_kept_object(object)?;
            kept.clone()
        }
        None => AisCircle::new(&geometry),
    };
    object.set_color(red())?;
    Ok(Some(object.cast()))
}

/// `TDataXtd_ELLIPSE`: the red `AIS_Shape` of the edge of the ellipse.
fn ellipse_presentation(
    label: &TdfLabel,
    ais: &Handle<dyn AisInteractiveObject>,
) -> Result<Option<Handle<dyn AisInteractiveObject>>, Failure> {
    let mut ellipse = Ellipse3::default();
    if !TDataXtdGeometry::ellipse_of_label(label, &mut ellipse)? {
        return Ok(None);
    }
    let mut maker = BRepBuilderAPIMakeEdge::from_elips(ellipse)?;
    if !maker.is_done() {
        return Ok(None);
    }
    let edge = maker.shape()?.clone();
    let kept = ais.cast::<AisShape>();
    let object = match kept.get() {
        Some(object) => {
            object.reset_transformation()?;
            object.set(&edge);
            object.set_to_update_all();
            object.update_selection(-1);
            kept.clone()
        }
        None => AisShape::new(&edge),
    };
    object.set_color(red())?;
    Ok(Some(object.cast()))
}

/// `TopoDS_Shape::Infinite()` (the shape is not null here).
fn shape_is_infinite(shape: &Shape) -> bool {
    shape.tshape().get().is_some_and(|tshape| tshape.infinite())
}

impl TPrsStdDriver for TPrsStdNamedShapeDriver {
    /// `Update`: the `AIS_Shape` of `TNaming_Tool::GetShape` (false for an empty named
    /// shape). A kept object takes the new shape only when it differs (the native
    /// `OPTIM_UPDATE` branch is compiled out), and always the infinite state of the shape.
    fn update(
        &self,
        label: &TdfLabel,
        ais: &mut Handle<dyn AisInteractiveObject>,
    ) -> Result<bool, Failure> {
        let mut named: Handle<TNamingNamedShape> = Handle::default();
        if !label.find_attribute_as(&TNamingNamedShape::get_id(), &mut named)? {
            return Ok(false);
        }
        let shape = TNamingTool::get_shape(&named)?;
        if shape.is_null() {
            return Ok(false);
        }
        let kept = ais.cast::<AisShape>();
        let object = match kept.get() {
            Some(object) => {
                if object.shape() != shape {
                    object.reset_transformation()?;
                    object.set(&shape);
                    object.update_selection(-1);
                    object.set_to_update_all();
                }
                object.set_infinite_state(shape_is_infinite(&shape));
                kept.clone()
            }
            None => AisShape::new(&shape),
        };
        *ais = object.cast();
        Ok(true)
    }
}
