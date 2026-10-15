// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_DimTolTool.cxx (the links of shapes to
// dimensions, tolerances and datums, and the GD&T presentations) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GD&T links of `XCAFDoc_DimTolTool`: the shapes of a dimension (first and second
//! graph nodes), of a tolerance and of a datum, the datums of a tolerance, and the
//! presentation shapes of the GD&T objects.
use super::dim_tol_tool::XCAFDocDimTolTool;
use super::graph_links::{
    child_labels_of, child_node, detach_fathers, father_labels, link_father, link_pair,
};
use super::support::{find, live};
use super::{XCAFDoc, XCAFDocDatum, XCAFDocDimension, XCAFDocGeomTolerance};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tklcaf::tdata_std::TDataStdTreeNode;
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel};
use std::collections::HashSet;

/// Replaces the fathers of the GD&T label's node of the GUID by the labels.
fn relink(gdt: &TdfLabel, labels: &[TdfLabel], guid: &Guid) -> Result<(), Failure> {
    detach_fathers(gdt, guid)?;
    let child = child_node(gdt, guid)?;
    for label in labels {
        link_father(label, &child, guid)?;
    }
    Ok(())
}

impl XCAFDocDimTolTool {
    /// `SetDimension(theL, theDimTolL)`: one first shape.
    pub fn set_dimension(&self, label: &TdfLabel, dimension: &TdfLabel) -> Result<(), Failure> {
        self.set_dimension_pair(label, &TdfLabel::default(), dimension)
    }
    /// `SetDimension(theFirstL, theSecondL, theDimTolL)`: the non-null shapes.
    pub fn set_dimension_pair(
        &self,
        first: &TdfLabel,
        second: &TdfLabel,
        dimension: &TdfLabel,
    ) -> Result<(), Failure> {
        let firsts: Vec<TdfLabel> = [first.clone()]
            .into_iter()
            .filter(|l| !l.is_null())
            .collect();
        let seconds: Vec<TdfLabel> = [second.clone()]
            .into_iter()
            .filter(|l| !l.is_null())
            .collect();
        self.set_dimension_shapes(&firsts, &seconds, dimension)
    }
    /// `SetDimension(theFirstL, theSecondL, theDimTolL)`: replaces the first and second shapes
    /// of a dimension (nothing without first shapes).
    pub fn set_dimension_shapes(
        &self,
        firsts: &[TdfLabel],
        seconds: &[TdfLabel],
        dimension: &TdfLabel,
    ) -> Result<(), Failure> {
        if !self.is_dimension(dimension)? || firsts.is_empty() {
            return Ok(());
        }
        let (first_guid, second_guid) = (
            XCAFDoc::dimension_ref_first_guid(),
            XCAFDoc::dimension_ref_second_guid(),
        );
        detach_fathers(dimension, &first_guid)?;
        detach_fathers(dimension, &second_guid)?;
        relink(dimension, firsts, &first_guid)?;
        if seconds.is_empty() {
            return Ok(());
        }
        let child = child_node(dimension, &second_guid)?;
        for label in seconds {
            link_father(label, &child, &second_guid)?;
        }
        Ok(())
    }
    /// `SetGeomTolerance(theL, theGeomTolL)`: one shape.
    pub fn set_geom_tolerance(
        &self,
        label: &TdfLabel,
        tolerance: &TdfLabel,
    ) -> Result<(), Failure> {
        self.set_geom_tolerance_shapes(std::slice::from_ref(label), tolerance)
    }
    /// `SetGeomTolerance(theL, theGeomTolL)`: replaces the shapes of a tolerance (nothing
    /// without shapes).
    pub fn set_geom_tolerance_shapes(
        &self,
        labels: &[TdfLabel],
        tolerance: &TdfLabel,
    ) -> Result<(), Failure> {
        if !self.is_geom_tolerance(tolerance)? || labels.is_empty() {
            return Ok(());
        }
        relink(tolerance, labels, &XCAFDoc::geom_tolerance_ref_guid())
    }
    /// `SetDimTol(theL, theDimTolL)`: the entry's node last among the users of the shape's
    /// node.
    pub fn set_dim_tol(&self, label: &TdfLabel, dim_tol: &TdfLabel) -> Result<(), Failure> {
        let guid = XCAFDoc::dim_tol_ref_guid();
        let reference = TDataStdTreeNode::set_with_id(dim_tol, &guid)?;
        let main = TDataStdTreeNode::set_with_id(label, &guid)?;
        // abv: fix against bug in TreeNode::Append()
        live(&reference, "XCAFDoc_DimTolTool::SetDimTol")?.remove()?;
        live(&main, "XCAFDoc_DimTolTool::SetDimTol")?.append(&reference)?;
        Ok(())
    }
    /// `SetDimTol(L, kind, aVal, aName, aDescription)`: links the shape to a new entry.
    pub fn set_dim_tol_values(
        &self,
        label: &TdfLabel,
        kind: i32,
        values: &Handle<rustycad_tkernel::ncollection::HArray1<f64>>,
        name: &Handle<HAsciiString>,
        description: &Handle<HAsciiString>,
    ) -> Result<TdfLabel, Failure> {
        let dim_tol = self.add_dim_tol(kind, values, name, description)?;
        self.set_dim_tol(label, &dim_tol)?;
        Ok(dim_tol)
    }
}

/// The father of the label's tree node of the GUID, when it has one.
fn tree_father(label: &TdfLabel, guid: &Guid) -> Result<Option<TdfLabel>, Failure> {
    let node = find::<TDataStdTreeNode>(label, guid)?;
    let Some(node) = node.get().filter(|node| node.has_father()) else {
        return Ok(None);
    };
    Ok(Some(
        live(&node.father(), "XCAFDoc_DimTolTool::GetRefShapeLabel")?.label(),
    ))
}

/// The fathers of the label's graph node of the GUID, when it has any.
fn graph_fathers(label: &TdfLabel, guid: &Guid) -> Result<Option<Vec<TdfLabel>>, Failure> {
    Ok(father_labels(label, guid)?.filter(|fathers| !fathers.is_empty()))
}

/// The children of the label's graph node of the GUID, when it has any.
fn graph_children(label: &TdfLabel, guid: &Guid) -> Result<Option<Vec<TdfLabel>>, Failure> {
    Ok(child_labels_of(label, guid)?.filter(|children| !children.is_empty()))
}

/// The references of the GD&T.
impl XCAFDocDimTolTool {
    /// `GetRefShapeLabel(theL, theShapeLFirst, theShapeLSecond)`: the shapes of a legacy entry
    /// or datum (tree nodes), else of a tolerance, datum or dimension (graph nodes).
    pub fn get_ref_shape_label(
        label: &TdfLabel,
        firsts: &mut Vec<TdfLabel>,
        seconds: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        firsts.clear();
        seconds.clear();
        for guid in [XCAFDoc::dim_tol_ref_guid(), XCAFDoc::datum_ref_guid()] {
            if let Some(father) = tree_father(label, &guid)? {
                firsts.push(father);
                return Ok(true);
            }
        }
        for guid in [
            XCAFDoc::geom_tolerance_ref_guid(),
            XCAFDoc::datum_ref_guid(),
        ] {
            if let Some(fathers) = graph_fathers(label, &guid)? {
                firsts.extend(fathers);
                return Ok(true);
            }
        }
        let Some(fathers) = graph_fathers(label, &XCAFDoc::dimension_ref_first_guid())? else {
            return Ok(false);
        };
        firsts.extend(fathers);
        seconds.extend(
            graph_fathers(label, &XCAFDoc::dimension_ref_second_guid())?.unwrap_or_default(),
        );
        Ok(true)
    }
    /// `GetRefDimensionLabels(theShapeL, theDimTols)`: appends the dimensions of which the
    /// shape is a first, then a second shape.
    pub fn get_ref_dimension_labels(
        &self,
        shape: &TdfLabel,
        dimensions: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        let mut found = false;
        for guid in [
            XCAFDoc::dimension_ref_first_guid(),
            XCAFDoc::dimension_ref_second_guid(),
        ] {
            if let Some(children) = graph_children(shape, &guid)? {
                dimensions.extend(children);
                found = true;
            }
        }
        Ok(found)
    }
    /// `GetRefGeomToleranceLabels(theShapeL, theDimTols)`: appends the tolerances of the shape.
    pub fn get_ref_geom_tolerance_labels(
        &self,
        shape: &TdfLabel,
        tolerances: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        let Some(children) = graph_children(shape, &XCAFDoc::geom_tolerance_ref_guid())? else {
            return Ok(false);
        };
        tolerances.extend(children);
        Ok(true)
    }
    /// `GetRefDatumLabel(theShapeL, theDatum)`: appends the datums of the shape (true with a
    /// node, even without children).
    pub fn get_ref_datum_label(
        &self,
        shape: &TdfLabel,
        datums: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        let Some(children) = child_labels_of(shape, &XCAFDoc::datum_ref_guid())? else {
            return Ok(false);
        };
        datums.extend(children);
        Ok(true)
    }
}

/// The datums of tolerances.
impl XCAFDocDimTolTool {
    /// `SetDatum(theL, theDatumL)`: replaces the shapes of a datum.
    pub fn set_datum_shapes(&self, labels: &[TdfLabel], datum: &TdfLabel) -> Result<(), Failure> {
        if !self.is_datum(datum)? {
            return Ok(());
        }
        relink(datum, labels, &XCAFDoc::datum_ref_guid())
    }
    /// `SetDatum(L, TolerL, aName, aDescription, anIdentification)`: the datum of these
    /// strings (found or added) on the shape and the tolerance.
    pub fn set_datum(
        &self,
        label: &TdfLabel,
        tolerance: &TdfLabel,
        name: &Handle<HAsciiString>,
        description: &Handle<HAsciiString>,
        identification: &Handle<HAsciiString>,
    ) -> Result<(), Failure> {
        let datum = match self.find_datum(name, description, identification)? {
            Some(datum) => datum,
            None => self.add_datum_with(name, description, identification)?,
        };
        self.set_datum_shapes(std::slice::from_ref(label), &datum)?;
        link_pair(tolerance, &datum, &XCAFDoc::datum_tol_ref_guid())
    }
    /// `SetDatumToGeomTol(theDatumL, theTolerL)`: links the datum to the tolerance.
    pub fn set_datum_to_geom_tol(
        &self,
        datum: &TdfLabel,
        tolerance: &TdfLabel,
    ) -> Result<(), Failure> {
        link_pair(tolerance, datum, &XCAFDoc::datum_tol_ref_guid())
    }
    /// `GetDatumOfTolerLabels(theDimTolL, theDatums)`: appends the datums of the tolerance.
    pub fn get_datum_of_toler_labels(
        tolerance: &TdfLabel,
        datums: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        let Some(children) = child_labels_of(tolerance, &XCAFDoc::datum_tol_ref_guid())? else {
            return Ok(false);
        };
        datums.extend(children);
        Ok(true)
    }
    /// `GetDatumWithObjectOfTolerLabels(theDimTolL, theDatums)`: appends the datums of the
    /// tolerance that have an object, once per object name (`Standard_NullObject` for an
    /// object without name, a null dereference natively).
    pub fn get_datum_with_object_of_toler_labels(
        tolerance: &TdfLabel,
        datums: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        let Some(children) = child_labels_of(tolerance, &XCAFDoc::datum_tol_ref_guid())? else {
            return Ok(false);
        };
        let mut names = HashSet::new();
        for datum in children {
            let attribute = find::<XCAFDocDatum>(&datum, &XCAFDocDatum::get_id())?;
            let Some(attribute) = attribute.get() else {
                continue;
            };
            let object = attribute.get_object()?;
            let Some(object) = object.get() else {
                continue;
            };
            let name = object.get_name();
            let text = live(&name, "XCAFDoc_DimTolTool::GetDatumWithObjectOfTolerLabels")?
                .string()
                .as_bytes()
                .to_vec();
            if names.insert(text) {
                datums.push(datum);
            }
        }
        Ok(true)
    }
    /// `GetTolerOfDatumLabels(theDatumL, theTols)`: appends the tolerances of the datum.
    pub fn get_toler_of_datum_labels(
        &self,
        datum: &TdfLabel,
        tolerances: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        let Some(fathers) = father_labels(datum, &XCAFDoc::datum_tol_ref_guid())? else {
            return Ok(false);
        };
        tolerances.extend(fathers);
        Ok(true)
    }
}

/// The GD&T attributes with an object that has a presentation shape.
trait GdtEntry: rustycad_tkernel::standard::RuntimeTyped + Sized {
    /// `GetID`.
    fn entry_id() -> Guid;
    /// The presentation of the object, `None` without object.
    fn presentation(&self) -> Result<Option<Shape>, Failure>;
    /// `anObject->SetPresentation(aPrs, anObject->GetPresentationName()); SetObject(anObject)`,
    /// nothing without object.
    fn update_presentation(&self, shape: &Shape) -> Result<(), Failure>;
}

/// Implements [`GdtEntry`] for an attribute with `get_object`/`set_object`.
macro_rules! gdt_entry {
    ($($attribute:ty),*) => {$(
        impl GdtEntry for $attribute {
            fn entry_id() -> Guid {
                <$attribute>::get_id()
            }
            fn presentation(&self) -> Result<Option<Shape>, Failure> {
                Ok(self.get_object()?.get().map(|object| object.get_presentation()))
            }
            fn update_presentation(&self, shape: &Shape) -> Result<(), Failure> {
                let object = self.get_object()?;
                let Some(o) = object.get() else {
                    return Ok(());
                };
                o.set_presentation(shape, &o.get_presentation_name());
                self.set_object(&object)
            }
        }
    )*};
}
gdt_entry!(XCAFDocDimension, XCAFDocGeomTolerance, XCAFDocDatum);

/// Appends the non-null presentations of the labels' entries of type `A` (a label already
/// in the map keeps its shape, as `NCollection_IndexedDataMap::Add`).
fn collect_presentations<A: GdtEntry>(
    labels: Vec<TdfLabel>,
    result: &mut Vec<(TdfLabel, Shape)>,
) -> Result<(), Failure> {
    for label in labels {
        let attribute = find::<A>(&label, &A::entry_id())?;
        let Some(shape) = attribute
            .get()
            .map(GdtEntry::presentation)
            .transpose()?
            .flatten()
        else {
            continue;
        };
        if !shape.is_null() && !result.iter().any(|(known, _)| *known == label) {
            result.push((label, shape));
        }
    }
    Ok(())
}

/// Updates the presentation of the label's entry of type `A`; false without that entry.
fn update_presentation<A: GdtEntry>(label: &TdfLabel, shape: &Shape) -> Result<bool, Failure> {
    let attribute = find::<A>(label, &A::entry_id())?;
    let Some(attribute) = attribute.get() else {
        return Ok(false);
    };
    attribute.update_presentation(shape)?;
    Ok(true)
}

/// The presentations of the GD&T objects.
impl XCAFDocDimTolTool {
    /// `GetGDTPresentations(theGDTLabelToShape)`: the non-null presentations of the
    /// dimensions, tolerances and datums with objects, in that order.
    pub fn get_gdt_presentations(&self) -> Result<Vec<(TdfLabel, Shape)>, Failure> {
        let mut result = Vec::new();
        collect_presentations::<XCAFDocDimension>(self.get_dimension_labels()?, &mut result)?;
        collect_presentations::<XCAFDocGeomTolerance>(
            self.get_geom_tolerance_labels()?,
            &mut result,
        )?;
        collect_presentations::<XCAFDocDatum>(self.get_datum_labels()?, &mut result)?;
        Ok(result)
    }
    /// `SetGDTPresentations(theGDTLabelToPrs)`: sets the presentation (keeping its name) of
    /// the object of each dimension, else tolerance, else datum label, and stores the object
    /// back.
    pub fn set_gdt_presentations(
        &self,
        presentations: &[(TdfLabel, Shape)],
    ) -> Result<(), Failure> {
        for (label, shape) in presentations {
            if !update_presentation::<XCAFDocDimension>(label, shape)?
                && !update_presentation::<XCAFDocGeomTolerance>(label, shape)?
            {
                update_presentation::<XCAFDocDatum>(label, shape)?;
            }
        }
        Ok(())
    }
}
