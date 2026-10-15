// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_DimTolTool.hxx and .cxx (the tool, its
// labels and the legacy `XCAFDoc_DimTol` and datum entries) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_DimTolTool`: the GD&T of an XDE document (dimensions, geometric tolerances,
//! datums and the legacy `XCAFDoc_DimTol` entries, one per child of the tool's label).
use super::color_tool::entry_text;
use super::generic_empty_tool::generic_empty_attribute;
use super::support::{child_labels, child_labels_with, find, find_or_add, live};
use super::tool_support::cached_shape_tool;
use super::{
    XCAFDocDatum, XCAFDocDimTol, XCAFDocDimension, XCAFDocDocumentTool, XCAFDocGeomTolerance,
    XCAFDocShapeTool,
};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use rustycad_tkernel::tcollection::{ExtendedString, HAsciiString};
use rustycad_tklcaf::tdata_std::TDataStdName;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTagSource};
use rustycad_tkservice::json_dump::field_string;
use std::io;

generic_empty_attribute!(
    /// `XCAFDoc_DimTolTool`.
    XCAFDocDimTolTool, "XCAFDoc_DimTolTool", "72afb19b-44de-11d8-8776-001083004c77",
    state: Handle<XCAFDocShapeTool>,
    dump: |tool: &XCAFDocDimTolTool, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        rustycad_tkservice::json_dump::class_begin(stream, "XCAFDoc_DimTolTool")?;
        rustycad_tklcaf::json::dump_attribute_json_base(tool, stream, depth)?;
        let groups = [
            ("aDimensionLabel", tool.get_dimension_labels()),
            ("aGeomToleranceLabel", tool.get_geom_tolerance_labels()),
            ("aDimTolLabelLabel", tool.get_dim_tol_labels()),
            ("aDatumLabel", tool.get_datum_labels()),
        ];
        for (field, labels) in groups {
            for label in labels.map_err(io::Error::other)? {
                field_string(stream, field, entry_text(&label).as_bytes())?;
            }
        }
        Ok(())
    }
);

/// The `GetDimTol` out values: the kind, the values, the name and the description.
pub type XCAFDocDimTolValues = (
    i32,
    Handle<HArray1<f64>>,
    Handle<HAsciiString>,
    Handle<HAsciiString>,
);

/// The `GetDatum` out values: the name, the description and the identification.
pub type XCAFDocDatumNames = (
    Handle<HAsciiString>,
    Handle<HAsciiString>,
    Handle<HAsciiString>,
);

/// The same string object, or both null (the handle `==`).
pub(super) fn same_string(first: &Handle<HAsciiString>, second: &Handle<HAsciiString>) -> bool {
    first.object_address() == second.object_address()
}

/// `aVal->Value(i)`: `Standard_NullObject` for a null array, `Standard_OutOfRange` outside it.
fn value_at(values: &Handle<HArray1<f64>>, index: i32) -> Result<f64, Failure> {
    let array = live(values, "XCAFDoc_DimTolTool::FindDimTol")?.array1();
    array
        .value(index)
        .copied()
        .map_err(|_| Failure::out_of_range("NCollection_Array1::Value"))
}

impl XCAFDocDimTolTool {
    /// `Set(L)`: the tool of the label, created (with the document's shape tool) when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocDimTolTool>, Failure> {
        let existing = find::<XCAFDocDimTolTool>(label, &Self::get_id())?;
        if !existing.is_null() {
            return Ok(existing);
        }
        let tool = find_or_add(label, &Self::get_id(), Self::new)?;
        *live(&tool, "XCAFDoc_DimTolTool::Set")?.state.write() =
            XCAFDocDocumentTool::shape_tool(label)?;
        Ok(tool)
    }
    /// `BaseLabel`.
    pub fn base_label(&self) -> TdfLabel {
        self.label()
    }
    /// `ShapeTool()`: the document's shape tool (looked up once).
    pub fn shape_tool(&self) -> Result<Handle<XCAFDocShapeTool>, Failure> {
        cached_shape_tool(&self.state, &self.label())
    }
    /// `IsDimTol(theLab)`: the label holds a legacy `XCAFDoc_DimTol`.
    pub fn is_dim_tol(&self, label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&XCAFDocDimTol::get_id())
    }
    /// `IsDimension(theLab)`.
    pub fn is_dimension(&self, label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&XCAFDocDimension::get_id())
    }
    /// `IsGeomTolerance(theLab)`.
    pub fn is_geom_tolerance(&self, label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&XCAFDocGeomTolerance::get_id())
    }
    /// `IsDatum(theLab)`.
    pub fn is_datum(&self, label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&XCAFDocDatum::get_id())
    }
    /// The children of the tool's label that hold an attribute of `id`.
    fn children_with(&self, id: &Guid) -> Result<Vec<TdfLabel>, Failure> {
        let mut labels = Vec::new();
        for label in child_labels(&self.label())? {
            if label.is_attribute(id)? {
                labels.push(label);
            }
        }
        Ok(labels)
    }
    /// `GetDimTolLabels(theLabels)`.
    pub fn get_dim_tol_labels(&self) -> Result<Vec<TdfLabel>, Failure> {
        self.children_with(&XCAFDocDimTol::get_id())
    }
    /// `GetDimensionLabels(theLabels)`.
    pub fn get_dimension_labels(&self) -> Result<Vec<TdfLabel>, Failure> {
        self.children_with(&XCAFDocDimension::get_id())
    }
    /// `GetGeomToleranceLabels(theLabels)`.
    pub fn get_geom_tolerance_labels(&self) -> Result<Vec<TdfLabel>, Failure> {
        self.children_with(&XCAFDocGeomTolerance::get_id())
    }
    /// `GetDatumLabels(theLabels)`.
    pub fn get_datum_labels(&self) -> Result<Vec<TdfLabel>, Failure> {
        self.children_with(&XCAFDocDatum::get_id())
    }
    /// A new child with the attribute `add` sets, then the name `DGT:<kind>`.
    fn new_entry(
        &self,
        add: impl FnOnce(&TdfLabel) -> Result<(), Failure>,
        name: &str,
    ) -> Result<TdfLabel, Failure> {
        let label = TdfTagSource::new_child_of(&self.label())?;
        add(&label)?;
        TDataStdName::set(&label, &ExtendedString::from_utf8(name))?;
        Ok(label)
    }
    /// `AddDimension`: a new child with an empty dimension, named `DGT:Dimension`.
    pub fn add_dimension(&self) -> Result<TdfLabel, Failure> {
        self.new_entry(
            |label| XCAFDocDimension::set(label).map(drop),
            "DGT:Dimension",
        )
    }
    /// `AddGeomTolerance`: a new child with an empty tolerance, named `DGT:Tolerance`.
    pub fn add_geom_tolerance(&self) -> Result<TdfLabel, Failure> {
        self.new_entry(
            |label| XCAFDocGeomTolerance::set(label).map(drop),
            "DGT:Tolerance",
        )
    }
    /// `AddDatum()`: a new child with an empty datum, named `DGT:Datum`.
    pub fn add_datum(&self) -> Result<TdfLabel, Failure> {
        self.new_entry(|label| XCAFDocDatum::set(label).map(drop), "DGT:Datum")
    }
}

/// The legacy `XCAFDoc_DimTol` entries.
impl XCAFDocDimTolTool {
    /// `FindDimTol(kind, aVal, aName, aDescription, lab)`: the first entry of that kind, the
    /// same name and description objects and equal values (all of a dimension, kind < 20;
    /// the first of a tolerance, kind < 50).
    pub fn find_dim_tol(
        &self,
        kind: i32,
        values: &Handle<HArray1<f64>>,
        name: &Handle<HAsciiString>,
        description: &Handle<HAsciiString>,
    ) -> Result<Option<TdfLabel>, Failure> {
        for label in child_labels_with(&self.label(), &XCAFDocDimTol::get_id())? {
            let attribute = find::<XCAFDocDimTol>(&label, &XCAFDocDimTol::get_id())?;
            let Some(dim_tol) = attribute.get() else {
                continue;
            };
            if dim_tol.get_kind() != kind
                || !same_string(name, &dim_tol.get_name())
                || !same_string(description, &dim_tol.get_description())
            {
                continue;
            }
            if equal_values(kind, values, &dim_tol.get_val())? {
                return Ok(Some(label));
            }
        }
        Ok(None)
    }
    /// `FindDimTol(kind, aVal, aName, aDescription)`: the entry, null when not found.
    pub fn find_dim_tol_label(
        &self,
        kind: i32,
        values: &Handle<HArray1<f64>>,
        name: &Handle<HAsciiString>,
        description: &Handle<HAsciiString>,
    ) -> Result<TdfLabel, Failure> {
        Ok(self
            .find_dim_tol(kind, values, name, description)?
            .unwrap_or_default())
    }
    /// `AddDimTol(kind, aVal, aName, aDescription)`: a new entry, named `DGT:Dimension`
    /// (kind < 20) or `DGT:Tolerance`.
    pub fn add_dim_tol(
        &self,
        kind: i32,
        values: &Handle<HArray1<f64>>,
        name: &Handle<HAsciiString>,
        description: &Handle<HAsciiString>,
    ) -> Result<TdfLabel, Failure> {
        let text = if kind < 20 {
            "DGT:Dimension"
        } else {
            "DGT:Tolerance"
        };
        self.new_entry(
            |label| XCAFDocDimTol::set(label, kind, values, name, description).map(drop),
            text,
        )
    }
    /// `GetDimTol(DimTolL, kind, aVal, aName, aDescription)`: the values of an entry.
    pub fn get_dim_tol(&self, label: &TdfLabel) -> Result<Option<XCAFDocDimTolValues>, Failure> {
        let attribute = find::<XCAFDocDimTol>(label, &XCAFDocDimTol::get_id())?;
        Ok(attribute
            .get()
            .map(|d| (d.get_kind(), d.get_val(), d.get_name(), d.get_description())))
    }
}

/// The value test of `FindDimTol`: every value of a dimension, the first of a tolerance,
/// none for other kinds.
fn equal_values(
    kind: i32,
    values: &Handle<HArray1<f64>>,
    found: &Handle<HArray1<f64>>,
) -> Result<bool, Failure> {
    let count = match kind {
        k if k < 20 => live(values, "XCAFDoc_DimTolTool::FindDimTol")?
            .array1()
            .length(),
        k if k < 50 => 1,
        _ => 0,
    };
    let mut equal = true;
    for index in 1..=count {
        if (value_at(values, index)? - value_at(found, index)?).abs() > precision::confusion() {
            equal = false;
        }
    }
    Ok(equal)
}

/// The datums.
impl XCAFDocDimTolTool {
    /// `FindDatum(aName, aDescription, anIdentification, lab)`: the first datum with the same
    /// three string objects.
    pub fn find_datum(
        &self,
        name: &Handle<HAsciiString>,
        description: &Handle<HAsciiString>,
        identification: &Handle<HAsciiString>,
    ) -> Result<Option<TdfLabel>, Failure> {
        for label in child_labels_with(&self.label(), &XCAFDocDatum::get_id())? {
            let Some((name1, description1, identification1)) = self.get_datum(&label)? else {
                continue;
            };
            if same_string(name, &name1)
                && same_string(description, &description1)
                && same_string(identification, &identification1)
            {
                return Ok(Some(label));
            }
        }
        Ok(None)
    }
    /// `AddDatum(aName, aDescription, anIdentification)`: a new child with the datum, named
    /// `DGT:Datum`.
    pub fn add_datum_with(
        &self,
        name: &Handle<HAsciiString>,
        description: &Handle<HAsciiString>,
        identification: &Handle<HAsciiString>,
    ) -> Result<TdfLabel, Failure> {
        self.new_entry(
            |label| {
                XCAFDocDatum::set_with_names(label, name, description, identification).map(drop)
            },
            "DGT:Datum",
        )
    }
    /// `GetDatum(theDatumL, theName, theDescription, theIdentification)`: the names of a
    /// datum, `None` for a null label or a label without datum.
    pub fn get_datum(&self, label: &TdfLabel) -> Result<Option<XCAFDocDatumNames>, Failure> {
        if label.is_null() {
            return Ok(None);
        }
        let attribute = find::<XCAFDocDatum>(label, &XCAFDocDatum::get_id())?;
        Ok(attribute
            .get()
            .map(|d| (d.get_name(), d.get_description(), d.get_identification())))
    }
    /// `IsLocked(theViewL)`.
    pub fn is_locked(&self, label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&super::XCAFDoc::lock_guid())
    }
    /// `Lock(theViewL)`.
    pub fn lock(&self, label: &TdfLabel) -> Result<(), Failure> {
        rustycad_tklcaf::tdata_std::TDataStdUAttribute::set(label, &super::XCAFDoc::lock_guid())?;
        Ok(())
    }
    /// `Unlock(theViewL)`.
    pub fn unlock(&self, label: &TdfLabel) -> Result<(), Failure> {
        label.forget_attribute_id(&super::XCAFDoc::lock_guid())?;
        Ok(())
    }
}
