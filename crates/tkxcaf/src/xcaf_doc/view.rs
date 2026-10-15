// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_View.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_View`: the attribute that keeps an `XCAFView_Object` in the children of its
//! label.
use super::dimension::enum_of;
use super::generic_empty_tool::generic_empty_attribute;
use super::object_storage::{child, integer_value, real_value};
use super::support::{child_labels, find, find_or_add};
use crate::xcaf_view::{XCAFViewObject, XCAFViewProjectionType};
use rustycad_tkcaf::tdata_xtd::{TDataXtdAxis, TDataXtdGeometry, TDataXtdPoint};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tklcaf::tdata_std::{TDataStdAsciiString, TDataStdInteger, TDataStdReal};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use rustycad_tkmath::gp::{Axis1, Direction3, Line3, Point3};
use std::io;

/// A real setter of the view object.
type RealSetter = fn(&XCAFViewObject, f64);

/// The child labels of a view (`ChildLab`).
mod child_lab {
    pub const NAME: i32 = 1;
    pub const TYPE: i32 = 2;
    pub const PROJECTION_POINT: i32 = 3;
    pub const VIEW_DIRECTION: i32 = 4;
    pub const UP_DIRECTION: i32 = 5;
    pub const ZOOM_FACTOR: i32 = 6;
    pub const WINDOW_HORIZONTAL_SIZE: i32 = 7;
    pub const WINDOW_VERTICAL_SIZE: i32 = 8;
    pub const FRONT_PLANE_DISTANCE: i32 = 9;
    pub const BACK_PLANE_DISTANCE: i32 = 10;
    pub const VIEW_VOLUME_SIDES_CLIPPING: i32 = 11;
    pub const CLIPPING_EXPRESSION: i32 = 12;
    pub const GDT_POINTS: i32 = 13;
}
use child_lab as lab;

generic_empty_attribute!(
    /// `XCAFDoc_View`.
    XCAFDocView, "XCAFDoc_View", "efd213e8-6dfd-11d4-b9c8-0060b0ee281b", state: (),
    dump: |view: &XCAFDocView, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        // No override natively: the `TDataStd_GenericEmpty` dump.
        rustycad_tklcaf::tdata_std::dump_generic_empty(view, stream, depth)
    }
);

/// `TDataStd_AsciiString::Set(label, string->String())`: `Standard_NullObject` for a null
/// string.
fn set_ascii(label: &TdfLabel, text: &Handle<HAsciiString>) -> Result<(), Failure> {
    let text = rustycad_tklcaf::live(text, "XCAFDoc_View::SetObject")?
        .string()
        .clone();
    TDataStdAsciiString::set(label, &text)?;
    Ok(())
}

/// The label's `TDataStd_AsciiString` as a new string, when it has one.
fn get_ascii(label: &TdfLabel) -> Result<Option<Handle<HAsciiString>>, Failure> {
    let text = find::<TDataStdAsciiString>(label, &TDataStdAsciiString::get_id())?;
    Ok(text
        .get()
        .map(|text| Handle::new(HAsciiString::from_string(&text.get()))))
}

/// `TDataXtd_Axis::Set(label, gp_Ax1(gp_Pnt(), direction))`.
fn set_direction(label: &TdfLabel, direction: Direction3) -> Result<(), Failure> {
    TDataXtdAxis::set_line(label, &Line3::new(Point3::default(), direction))?;
    Ok(())
}

/// The direction of the label's axis, when it has one.
fn get_direction(label: &TdfLabel) -> Result<Option<Direction3>, Failure> {
    if !label.is_attribute(&TDataXtdAxis::get_id())? {
        return Ok(None);
    }
    let mut axis = Axis1::default();
    TDataXtdGeometry::axis_of_label(label, &mut axis)?;
    Ok(Some(axis.direction()))
}

/// The label's point, when it has one.
fn get_point(label: &TdfLabel) -> Result<Option<Point3>, Failure> {
    if !label.is_attribute(&TDataXtdPoint::get_id())? {
        return Ok(None);
    }
    let mut point = Point3::default();
    TDataXtdGeometry::point_of_label(label, &mut point)?;
    Ok(Some(point))
}

impl XCAFDocView {
    /// `Set(theLabel)`: the view of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocView>, Failure> {
        find_or_add(label, &Self::get_id(), Self::new)
    }
    /// `SetObject(theObject)`: backed up; the values of the object on fresh children (the
    /// optional ones only when set). `Standard_NullObject` for a null object or name.
    pub fn set_object(&self, object: &Handle<XCAFViewObject>) -> Result<(), Failure> {
        self.backup()?;
        let label = self.label();
        for child_label in child_labels(&label)? {
            child_label.forget_all_attributes(true)?;
        }
        let object = rustycad_tklcaf::live(object, "XCAFDoc_View::SetObject")?;
        set_ascii(&child(&label, lab::NAME)?, &object.name())?;
        TDataStdInteger::set(&child(&label, lab::TYPE)?, object.kind() as i32)?;
        TDataXtdPoint::set_point(
            &child(&label, lab::PROJECTION_POINT)?,
            object.projection_point(),
        )?;
        set_direction(
            &child(&label, lab::VIEW_DIRECTION)?,
            object.view_direction(),
        )?;
        set_direction(&child(&label, lab::UP_DIRECTION)?, object.up_direction())?;
        TDataStdReal::set(&child(&label, lab::ZOOM_FACTOR)?, object.zoom_factor())?;
        TDataStdReal::set(
            &child(&label, lab::WINDOW_HORIZONTAL_SIZE)?,
            object.window_horizontal_size(),
        )?;
        TDataStdReal::set(
            &child(&label, lab::WINDOW_VERTICAL_SIZE)?,
            object.window_vertical_size(),
        )?;
        self.set_optional_values(&label, object)
    }
    /// The clipping values and the GD&T points of `SetObject`.
    fn set_optional_values(
        &self,
        label: &TdfLabel,
        object: &XCAFViewObject,
    ) -> Result<(), Failure> {
        if object.has_front_plane_clipping() {
            TDataStdReal::set(
                &child(label, lab::FRONT_PLANE_DISTANCE)?,
                object.front_plane_distance(),
            )?;
        }
        if object.has_back_plane_clipping() {
            TDataStdReal::set(
                &child(label, lab::BACK_PLANE_DISTANCE)?,
                object.back_plane_distance(),
            )?;
        }
        let sides = i32::from(object.has_view_volume_sides_clipping());
        TDataStdInteger::set(&child(label, lab::VIEW_VOLUME_SIDES_CLIPPING)?, sides)?;
        let expression = object.clipping_expression();
        if !expression.is_null() {
            set_ascii(&child(label, lab::CLIPPING_EXPRESSION)?, &expression)?;
        }
        if object.has_gdt_points() {
            let points = child(label, lab::GDT_POINTS)?;
            for index in 1..=object.nb_gdt_points() {
                TDataXtdPoint::set_point(&child(&points, index)?, object.gdt_point(index))?;
            }
        }
        Ok(())
    }
}

/// Reading the object back.
impl XCAFDocView {
    /// `GetObject`: a new object with the values stored in the children (which it creates
    /// when absent, as natively).
    pub fn get_object(&self) -> Result<Handle<XCAFViewObject>, Failure> {
        let label = self.label();
        let object = XCAFViewObject::new();
        if let Some(name) = get_ascii(&child(&label, lab::NAME)?)? {
            object.set_name(&name);
        }
        if let Some(kind) = integer_value(&child(&label, lab::TYPE)?)? {
            object.set_type(enum_of::<XCAFViewProjectionType>(
                kind,
                "XCAFView_ProjectionType",
            )?);
        }
        if let Some(point) = get_point(&child(&label, lab::PROJECTION_POINT)?)? {
            object.set_projection_point(&point);
        }
        if let Some(direction) = get_direction(&child(&label, lab::VIEW_DIRECTION)?)? {
            object.set_view_direction(&direction);
        }
        if let Some(direction) = get_direction(&child(&label, lab::UP_DIRECTION)?)? {
            object.set_up_direction(&direction);
        }
        Self::get_reals(&label, &object)?;
        if let Some(clipping) = integer_value(&child(&label, lab::VIEW_VOLUME_SIDES_CLIPPING)?)? {
            object.set_view_volume_sides_clipping(clipping == 1);
        }
        if let Some(expression) = get_ascii(&child(&label, lab::CLIPPING_EXPRESSION)?)? {
            object.set_clipping_expression(&expression);
        }
        Self::get_gdt_points(&label, &object)?;
        Ok(Handle::new(object))
    }
    /// The zoom, window sizes and plane distances of `GetObject`.
    fn get_reals(label: &TdfLabel, object: &XCAFViewObject) -> Result<(), Failure> {
        let setters: [(i32, RealSetter); 5] = [
            (lab::ZOOM_FACTOR, XCAFViewObject::set_zoom_factor),
            (
                lab::WINDOW_HORIZONTAL_SIZE,
                XCAFViewObject::set_window_horizontal_size,
            ),
            (
                lab::WINDOW_VERTICAL_SIZE,
                XCAFViewObject::set_window_vertical_size,
            ),
            (
                lab::FRONT_PLANE_DISTANCE,
                XCAFViewObject::set_front_plane_distance,
            ),
            (
                lab::BACK_PLANE_DISTANCE,
                XCAFViewObject::set_back_plane_distance,
            ),
        ];
        for (tag, set) in setters {
            if let Some(value) = real_value(&child(label, tag)?)? {
                set(object, value);
            }
        }
        Ok(())
    }
    /// The GD&T points of `GetObject`: as many as the children of the points label with a
    /// point, set from the children of tags 1 to the number of children (created when absent,
    /// as natively).
    fn get_gdt_points(label: &TdfLabel, object: &XCAFViewObject) -> Result<(), Failure> {
        if label.find_child(lab::GDT_POINTS, false)?.is_null() {
            return Ok(());
        }
        let points_label = child(label, lab::GDT_POINTS)?;
        let mut count = 0;
        for point_label in child_labels(&points_label)? {
            if point_label.is_attribute(&TDataXtdPoint::get_id())? {
                count += 1;
            }
        }
        if count == 0 {
            return Ok(());
        }
        object.create_gdt_points(count);
        let mut point_index = 1;
        for tag in 1..=points_label.nb_children()? {
            if let Some(point) = get_point(&child(&points_label, tag)?)? {
                object.set_gdt_point(point_index, &point);
                point_index += 1;
            }
        }
        Ok(())
    }
}
