// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ClippingPlaneTool.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_ClippingPlaneTool`: the clipping planes of the views of an XDE document (one
//! `TDataXtd_Plane` per child of the tool's label, with a name and a capping flag).
use super::XCAFDoc;
use super::generic_empty_tool::generic_empty_attribute;
use super::support::{child_labels_with, find};
use rustycad_tkcaf::tdata_xtd::{TDataXtdGeometry, TDataXtdPlane};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString};
use rustycad_tklcaf::tdata_std::{TDataStdGenericExtString, TDataStdInteger, TDataStdName};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTagSource};
use rustycad_tkmath::gp::Plane3;
use std::io;

generic_empty_attribute!(
    /// `XCAFDoc_ClippingPlaneTool`.
    XCAFDocClippingPlaneTool, "XCAFDoc_ClippingPlaneTool", "efd213ea-6dfd-11d4-b9c8-0060b0ee281b",
    state: (),
    dump: |tool: &XCAFDocClippingPlaneTool, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        // No override natively: the `TDataStd_GenericEmpty` dump.
        rustycad_tklcaf::tdata_std::dump_generic_empty(tool, stream, depth)
    }
);

/// A clipping plane: its plane, name and capping flag (`GetClippingPlane` out values).
#[derive(Clone, Debug, Default)]
pub struct XCAFDocClippingPlane {
    /// The plane (`gp_Pln` out value, untouched when the label has no geometry).
    pub plane: Plane3,
    /// The name (empty without one).
    pub name: ExtendedString,
    /// The capping flag (false without one).
    pub capping: bool,
}

impl XCAFDocClippingPlaneTool {
    /// `Set(L)`: the tool of the label, added when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocClippingPlaneTool>, Failure> {
        super::support::find_or_add(label, &Self::get_id(), Self::new)
    }
    /// `BaseLabel`.
    pub fn base_label(&self) -> TdfLabel {
        self.label()
    }
    /// `IsClippingPlane(theLabel)`: a child of the tool's label with a plane.
    pub fn is_clipping_plane(&self, label: &TdfLabel) -> Result<bool, Failure> {
        if label.father() != self.label() {
            return Ok(false);
        }
        label.is_attribute(&TDataXtdPlane::get_id())
    }
    /// `GetClippingPlane(theLabel, thePlane, theName, theCapping)`: the plane, its name and
    /// capping flag, `None` for a label that is not a clipping plane.
    pub fn get_clipping_plane(
        &self,
        label: &TdfLabel,
    ) -> Result<Option<XCAFDocClippingPlane>, Failure> {
        if !self.is_clipping_plane(label)? {
            return Ok(None);
        }
        let mut result = XCAFDocClippingPlane::default();
        TDataXtdGeometry::plane_of_label(label, &mut result.plane)?;
        let name = find::<TDataStdName>(label, &TDataStdName::get_id())?;
        if let Some(name) = name.get() {
            result.name = name.get();
        }
        let capping = find::<TDataStdInteger>(label, &TDataStdInteger::get_id())?;
        if let Some(capping) = capping.get() {
            result.capping = capping.get() == 1;
        }
        Ok(Some(result))
    }
    /// `GetClippingPlane(theLabel, thePlane, theName, theCapping)` with an ASCII name.
    pub fn get_clipping_plane_ascii(
        &self,
        label: &TdfLabel,
    ) -> Result<Option<(Plane3, Handle<HAsciiString>, bool)>, Failure> {
        Ok(self.get_clipping_plane(label)?.map(|found| {
            let name = HAsciiString::from(AsciiString::from_extended(&found.name, None));
            (found.plane, Handle::new(name), found.capping)
        }))
    }
    /// `AddClippingPlane(thePlane, theName)`: the clipping plane of that name and axes (within
    /// the angular precision), or a new child with the plane and (when not empty) the name.
    pub fn add_clipping_plane(
        &self,
        plane: &Plane3,
        name: &ExtendedString,
    ) -> Result<TdfLabel, Failure> {
        for label in self.get_clipping_planes()? {
            let found = self.get_clipping_plane(&label)?.unwrap_or_default();
            if found.name == *name && same_axes(&found.plane, plane) {
                return Ok(label);
            }
        }
        let label = TdfTagSource::new_child_of(&self.label())?;
        TDataXtdPlane::set_plane(&label, plane)?;
        if !name.is_empty() {
            TDataStdName::set(&label, name)?;
        }
        Ok(label)
    }
    /// `AddClippingPlane(thePlane, theName)` with an ASCII name.
    pub fn add_clipping_plane_ascii(
        &self,
        plane: &Plane3,
        name: &Handle<HAsciiString>,
    ) -> Result<TdfLabel, Failure> {
        self.add_clipping_plane(plane, &ascii_name(name)?)
    }
    /// `AddClippingPlane(thePlane, theName, theCapping)`: also sets the capping flag.
    pub fn add_clipping_plane_capping(
        &self,
        plane: &Plane3,
        name: &ExtendedString,
        capping: bool,
    ) -> Result<TdfLabel, Failure> {
        let label = self.add_clipping_plane(plane, name)?;
        TDataStdInteger::set(&label, i32::from(capping))?;
        Ok(label)
    }
    /// `AddClippingPlane(thePlane, theName, theCapping)` with an ASCII name.
    pub fn add_clipping_plane_ascii_capping(
        &self,
        plane: &Plane3,
        name: &Handle<HAsciiString>,
        capping: bool,
    ) -> Result<TdfLabel, Failure> {
        self.add_clipping_plane_capping(plane, &ascii_name(name)?, capping)
    }
}

/// `TCollection_ExtendedString(theName->String())`: `Standard_NullObject` for a null name.
fn ascii_name(name: &Handle<HAsciiString>) -> Result<ExtendedString, Failure> {
    let text = rustycad_tklcaf::live(name, "XCAFDoc_ClippingPlaneTool::AddClippingPlane")?
        .string()
        .clone();
    Ok(ExtendedString::from_ascii(&text))
}

/// The main, X and Y axes of the planes are parallel within `Precision::Angular()`.
fn same_axes(first: &Plane3, second: &Plane3) -> bool {
    let tolerance = precision::angular();
    first.axis().angle(second.axis()) <= tolerance
        && first.x_axis().angle(second.x_axis()) <= tolerance
        && first.y_axis().angle(second.y_axis()) <= tolerance
}

/// Removal, update and capping.
impl XCAFDocClippingPlaneTool {
    /// `RemoveClippingPlane(theLabel)`: forgets every attribute of a clipping plane no view
    /// refers to.
    pub fn remove_clipping_plane(&self, label: &TdfLabel) -> Result<bool, Failure> {
        if !self.is_clipping_plane(label)? || label.is_attribute(&XCAFDoc::view_ref_plane_guid())? {
            return Ok(false);
        }
        label.forget_all_attributes(true)?;
        Ok(true)
    }
    /// `GetClippingPlanes(theLabels)`: the clipping plane labels.
    pub fn get_clipping_planes(&self) -> Result<Vec<TdfLabel>, Failure> {
        let mut labels = Vec::new();
        for label in child_labels_with(&self.label(), &TDataXtdPlane::get_id())? {
            if self.is_clipping_plane(&label)? {
                labels.push(label);
            }
        }
        Ok(labels)
    }
    /// `UpdateClippingPlane(theLabel, thePlane, theName)`: replaces the plane and the name of
    /// a clipping plane.
    pub fn update_clipping_plane(
        &self,
        label: &TdfLabel,
        plane: &Plane3,
        name: &ExtendedString,
    ) -> Result<(), Failure> {
        if !self.is_clipping_plane(label)? {
            return Ok(());
        }
        label.forget_attribute_id(&TDataXtdPlane::get_id())?;
        TDataXtdPlane::set_plane(label, plane)?;
        label.forget_attribute_id(&TDataStdName::get_id())?;
        TDataStdName::set(label, name)?;
        Ok(())
    }
    /// `SetCapping(theClippingPlaneL, theCapping)`: replaces the capping flag of a child.
    pub fn set_capping(&self, label: &TdfLabel, capping: bool) -> Result<(), Failure> {
        if label.father() != self.label() {
            return Ok(());
        }
        label.forget_attribute_id(&TDataStdInteger::get_id())?;
        TDataStdInteger::set(label, i32::from(capping))?;
        Ok(())
    }
    /// `GetCapping(theClippingPlaneL)` (and the overload with an out flag, `None` meaning
    /// false): the capping flag of a child.
    pub fn get_capping(&self, label: &TdfLabel) -> Result<Option<bool>, Failure> {
        if label.father() != self.label() {
            return Ok(None);
        }
        let capping = find::<TDataStdInteger>(label, &TDataStdInteger::get_id())?;
        Ok(capping.get().map(|capping| capping.get() == 1))
    }
}
