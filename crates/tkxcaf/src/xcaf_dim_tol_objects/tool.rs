// Rust port of DataExchange/TKXCAF/XCAFDimTolObjects/XCAFDimTolObjects_Tool.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDimTolObjects_Tool`: the GD&T objects of a document and of its shapes.
use super::{
    XCAFDimTolObjectsDatumObject, XCAFDimTolObjectsDimensionObject,
    XCAFDimTolObjectsGeomToleranceObject,
};
use crate::xcaf_doc::{
    XCAFDocDatum, XCAFDocDimTolTool, XCAFDocDimension, XCAFDocDocumentTool, XCAFDocGeomTolerance,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};

/// A tolerance object and the datum object it maps to
/// (`NCollection_DataMap<handle<GeomToleranceObject>, handle<DatumObject>>`, by object).
pub type XCAFDimTolObjectsToleranceDatums = Vec<(
    Handle<XCAFDimTolObjectsGeomToleranceObject>,
    Handle<XCAFDimTolObjectsDatumObject>,
)>;

/// `theMap.Bind(tolerance, datum)`: replaces the datum of a known tolerance object.
fn bind(
    map: &mut XCAFDimTolObjectsToleranceDatums,
    tolerance: &Handle<XCAFDimTolObjectsGeomToleranceObject>,
    datum: &Handle<XCAFDimTolObjectsDatumObject>,
) {
    match map
        .iter_mut()
        .find(|(known, _)| known.object_address() == tolerance.object_address())
    {
        Some(entry) => entry.1 = datum.clone(),
        None => map.push((tolerance.clone(), datum.clone())),
    }
}

/// `XCAFDimTolObjects_Tool`.
pub struct XCAFDimTolObjectsTool {
    dim_tol_tool: Handle<XCAFDocDimTolTool>,
}

impl XCAFDimTolObjectsTool {
    /// `XCAFDimTolObjects_Tool(theDoc)`: the GD&T tool of the document (created when absent).
    pub fn new(
        document: &Handle<rustycad_tklcaf::tdoc_std::TDocStdDocument>,
    ) -> Result<Self, Failure> {
        let main = rustycad_tklcaf::live(document, "XCAFDimTolObjects_Tool")?.main()?;
        Ok(Self {
            dim_tol_tool: XCAFDocDocumentTool::dim_tol_tool(&main)?,
        })
    }
    /// The GD&T tool.
    fn tool(&self) -> Result<&XCAFDocDimTolTool, Failure> {
        rustycad_tklcaf::live(&self.dim_tol_tool, "XCAFDimTolObjects_Tool")
    }
    /// `GetDimensions(theDimensionObjectSequence)`: the objects of the dimensions.
    pub fn get_dimensions(&self) -> Result<Vec<Handle<XCAFDimTolObjectsDimensionObject>>, Failure> {
        let mut objects = Vec::new();
        for label in crate::xcaf_doc::child_labels_of_tool(&self.tool()?.label())? {
            if let Some(dimension) =
                find_attribute::<XCAFDocDimension>(&label, &XCAFDocDimension::get_id())?.get()
            {
                objects.push(dimension.get_object()?);
            }
        }
        Ok(objects)
    }
    /// `GetGeomTolerances(theGeomToleranceObjectSequence, theDatumSequence, theMap)`: the
    /// objects of the tolerances, and the objects of their datums (appended), each mapping
    /// its tolerance.
    pub fn get_geom_tolerances(
        &self,
        datums: &mut Vec<Handle<XCAFDimTolObjectsDatumObject>>,
        map: &mut XCAFDimTolObjectsToleranceDatums,
    ) -> Result<Vec<Handle<XCAFDimTolObjectsGeomToleranceObject>>, Failure> {
        let mut objects = Vec::new();
        for label in crate::xcaf_doc::child_labels_of_tool(&self.tool()?.label())? {
            self.add_tolerance(&label, &mut objects, datums, map)?;
        }
        Ok(objects)
    }
    /// Appends the object of the label's tolerance and those of its datums.
    fn add_tolerance(
        &self,
        label: &TdfLabel,
        objects: &mut Vec<Handle<XCAFDimTolObjectsGeomToleranceObject>>,
        datums: &mut Vec<Handle<XCAFDimTolObjectsDatumObject>>,
        map: &mut XCAFDimTolObjectsToleranceDatums,
    ) -> Result<(), Failure> {
        let tolerance =
            find_attribute::<XCAFDocGeomTolerance>(label, &XCAFDocGeomTolerance::get_id())?;
        let Some(tolerance) = tolerance.get() else {
            return Ok(());
        };
        let object = tolerance.get_object()?;
        objects.push(object.clone());
        let mut datum_labels = Vec::new();
        if XCAFDocDimTolTool::get_datum_of_toler_labels(&tolerance.label(), &mut datum_labels)? {
            for datum_label in datum_labels {
                if let Some(datum) =
                    find_attribute::<XCAFDocDatum>(&datum_label, &XCAFDocDatum::get_id())?.get()
                {
                    datums.push(datum.get_object()?);
                    bind(map, &object, datums.last().expect("just pushed"));
                }
            }
        }
        Ok(())
    }
}

/// The objects of the GD&T of shapes.
impl XCAFDimTolObjectsTool {
    /// The label of a shape through the shape tool's `Search`, `None` when not found.
    fn shape_label(&self, shape: &Shape) -> Result<Option<TdfLabel>, Failure> {
        let shapes = self.tool()?.shape_tool()?;
        let mut label = TdfLabel::default();
        rustycad_tklcaf::live(&shapes, "XCAFDimTolObjects_Tool")?.search(
            shape,
            &mut label,
            [true, true, true],
        )?;
        Ok((!label.is_null()).then_some(label))
    }
    /// `GetRefDimensions(theShape, theDimensionObjectSequence)`: the objects of the
    /// dimensions of the shape, `None` when it has none.
    pub fn get_ref_dimensions(
        &self,
        shape: &Shape,
    ) -> Result<Option<Vec<Handle<XCAFDimTolObjectsDimensionObject>>>, Failure> {
        let Some(label) = self.shape_label(shape)? else {
            return Ok(None);
        };
        let mut labels = Vec::new();
        if !self.tool()?.get_ref_dimension_labels(&label, &mut labels)? {
            return Ok(None);
        }
        let mut objects = Vec::new();
        for dimension_label in labels {
            if let Some(dimension) =
                find_attribute::<XCAFDocDimension>(&dimension_label, &XCAFDocDimension::get_id())?
                    .get()
            {
                objects.push(dimension.get_object()?);
            }
        }
        Ok(Some(objects))
    }
    /// `GetRefGeomTolerances(theShape, theGeomToleranceObjectSequence, theDatumSequence,
    /// theMap)`: the objects of the tolerances of the shape and of their datums, `None` when
    /// it has none.
    pub fn get_ref_geom_tolerances(
        &self,
        shape: &Shape,
        datums: &mut Vec<Handle<XCAFDimTolObjectsDatumObject>>,
        map: &mut XCAFDimTolObjectsToleranceDatums,
    ) -> Result<Option<Vec<Handle<XCAFDimTolObjectsGeomToleranceObject>>>, Failure> {
        let Some(label) = self.shape_label(shape)? else {
            return Ok(None);
        };
        let mut labels = Vec::new();
        if !self
            .tool()?
            .get_ref_geom_tolerance_labels(&label, &mut labels)?
        {
            return Ok(None);
        }
        let mut objects = Vec::new();
        for tolerance_label in labels {
            self.add_tolerance(&tolerance_label, &mut objects, datums, map)?;
        }
        Ok(Some(objects))
    }
    /// `GetRefDatum(theShape, theDatumObject)`: the object of the first datum of the shape.
    pub fn get_ref_datum(
        &self,
        shape: &Shape,
    ) -> Result<Option<Handle<XCAFDimTolObjectsDatumObject>>, Failure> {
        let Some(label) = self.shape_label(shape)? else {
            return Ok(None);
        };
        let mut labels = Vec::new();
        if !self.tool()?.get_ref_datum_label(&label, &mut labels)? {
            return Ok(None);
        }
        let first = labels.first().ok_or_else(|| {
            Failure::with_message("Standard_NoSuchObject", "NCollection_Sequence::First")
        })?;
        let datum = find_attribute::<XCAFDocDatum>(first, &XCAFDocDatum::get_id())?;
        datum.get().map(XCAFDocDatum::get_object).transpose()
    }
}

/// The label's attribute of `id` as a `T`, or null.
fn find_attribute<T: rustycad_tkernel::standard::RuntimeTyped + ?Sized>(
    label: &TdfLabel,
    id: &rustycad_tkernel::standard::Guid,
) -> Result<Handle<T>, Failure> {
    let mut found: Handle<T> = Handle::default();
    label.find_attribute_as(id, &mut found)?;
    Ok(found)
}
