// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc.cxx (AttributeInfo) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc::AttributeInfo`: a one-line description of an attribute of an XDE document,
//! by attribute class in the native order (an `XCAFDoc_Area` is described as the
//! `TDataStd_Real` it derives from).
use super::color_tool::entry_text;
use super::{
    XCAFDoc, XCAFDocCentroid, XCAFDocColor, XCAFDocColorType, XCAFDocDimTol, XCAFDocGraphNode,
    XCAFDocLengthUnit, XCAFDocMaterial, XCAFDocShapeMapTool,
};
use rustycad_tkcaf::tnaming::TNamingNamedShape;
use rustycad_tkernel::standard::c_string::{FormatArg, sprintf_into};
use rustycad_tkernel::standard::{Failure, Guid, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tklcaf::tdata_std::{
    TDataStdAsciiString, TDataStdByteArray, TDataStdComment, TDataStdGenericExtString,
    TDataStdInteger, TDataStdIntegerArray, TDataStdName, TDataStdReal, TDataStdRealArray,
    TDataStdTreeNode, TDataStdUAttribute,
};
use rustycad_tklcaf::tdf::{TdfAttribute, TdfAttributeState, TdfReference, TdfTagSource};

/// `TCollection_AsciiString(double)`.
fn real(value: f64) -> String {
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}

/// The labels of tree nodes, as their entries.
fn tree_entries(mut node: Handle<TDataStdTreeNode>) -> Result<Vec<String>, Failure> {
    let mut entries = Vec::new();
    while let Some(current) = node.get() {
        entries.push(entry_text(&current.label()));
        node = current.next();
    }
    Ok(entries)
}

/// The name of a tree-node link.
fn tree_link_name(id: Guid) -> &'static str {
    let names = [
        (XCAFDoc::shape_ref_guid(), "Shape Instance Link"),
        (
            XCAFDoc::color_ref_guid(XCAFDocColorType::ColorGen),
            "Generic Color Link",
        ),
        (
            XCAFDoc::color_ref_guid(XCAFDocColorType::ColorSurf),
            "Surface Color Link",
        ),
        (
            XCAFDoc::color_ref_guid(XCAFDocColorType::ColorCurv),
            "Curve Color Link",
        ),
        (XCAFDoc::dim_tol_ref_guid(), "DGT Link"),
        (XCAFDoc::datum_ref_guid(), "Datum Link"),
        (XCAFDoc::material_ref_guid(), "Material Link"),
    ];
    names
        .iter()
        .find(|(guid, _)| *guid == id)
        .map_or("", |(_, name)| name)
}

/// A tree node: `<type> ==> <father>`, else `<type> <== (<children>)`.
fn tree_node_info(node: &TDataStdTreeNode) -> Result<String, Failure> {
    let kind = tree_link_name(node.id());
    if node.has_father() {
        let father = rustycad_tklcaf::live(&node.father(), "XCAFDoc::AttributeInfo")?.label();
        return Ok(format!("{kind} ==> {}", entry_text(&father)));
    }
    Ok(format!(
        "{kind} <== ({})",
        tree_entries(node.first())?.join(", ")
    ))
}

/// The name of a graph-node link, `None` for another graph.
fn graph_link_name(id: Guid) -> Option<&'static str> {
    let names = [
        (XCAFDoc::layer_ref_guid(), "Layer Instance Link"),
        (XCAFDoc::shuo_ref_guid(), "SHUO Instance Link"),
        (XCAFDoc::datum_tol_ref_guid(), "DatumToler Link"),
        (XCAFDoc::dimension_ref_first_guid(), "Dimension Link First"),
        (
            XCAFDoc::dimension_ref_second_guid(),
            "Dimension Link Second",
        ),
        (XCAFDoc::geom_tolerance_ref_guid(), "GeomTolerance Link"),
        (XCAFDoc::datum_ref_guid(), "Datum Link"),
        (XCAFDoc::view_ref_shape_guid(), "View Shape Link"),
        (XCAFDoc::view_ref_gdt_guid(), "View GD&T Link"),
        (XCAFDoc::view_ref_plane_guid(), "View Clipping Plane Link"),
    ];
    names
        .iter()
        .find(|(guid, _)| *guid == id)
        .map(|(_, name)| *name)
}

/// The entries of graph nodes, joined by `, `.
fn graph_entries(nodes: &[Handle<XCAFDocGraphNode>]) -> Result<String, Failure> {
    let entries: Result<Vec<String>, Failure> = nodes
        .iter()
        .map(|node| {
            Ok(entry_text(
                &rustycad_tklcaf::live(node, "XCAFDoc::AttributeInfo")?.label(),
            ))
        })
        .collect();
    Ok(entries?.join(", "))
}

/// A graph node: `<type> ==> (<fathers>) ` then `<type> <== (<children>) `; empty for an
/// unknown graph.
fn graph_node_info(node: &XCAFDocGraphNode) -> Result<String, Failure> {
    let Some(kind) = graph_link_name(node.id()) else {
        return Ok(String::new());
    };
    let mut info = String::new();
    if node.nb_fathers() != 0 {
        info += &format!("{kind} ==> ({}) ", graph_entries(&node.fathers())?);
    }
    if node.nb_children() != 0 {
        info += &format!("{kind} <== ({}) ", graph_entries(&node.children())?);
    }
    Ok(info)
}

/// The values of an array attribute, joined by `, `.
fn array_info<T>(
    lower: i32,
    upper: i32,
    value: impl Fn(i32) -> Result<T, Failure>,
    text: impl Fn(T) -> String,
) -> Result<String, Failure> {
    let values: Result<Vec<String>, Failure> = (lower..=upper)
        .map(|index| Ok(text(value(index)?)))
        .collect();
    Ok(values?.join(", "))
}

/// A named shape: the class of its `TShape` (`Empty Shape` without one), `(located)` when
/// moved.
fn named_shape_info(named: &TNamingNamedShape) -> Result<String, Failure> {
    let shape = named.get()?;
    let mut info = match shape.tshape().get() {
        Some(tshape) if !shape.is_null() => tshape.dynamic_type().name().to_owned(),
        _ => "Empty Shape".to_owned(),
    };
    if !shape.location().is_identity() {
        info += "(located)";
    }
    Ok(info)
}

/// A color: `<name> (%g, %g, %g, %g)`.
fn color_info(color: &XCAFDocColor) -> Result<String, Failure> {
    let rgba = color.get_color_rgba();
    let rgb = *rgba.rgb();
    let name = rustycad_tkernel::quantity::Color::string_name(rgb.name());
    let mut text = Vec::new();
    let args = [
        FormatArg::Text(name.as_bytes()),
        FormatArg::Real(rgb.red()),
        FormatArg::Real(rgb.green()),
        FormatArg::Real(rgb.blue()),
        FormatArg::Real(f64::from(rgba.alpha())),
    ];
    sprintf_into(&mut text, b"%s (%g, %g, %g, %g)", &args)
        .map_err(|e| Failure::new(&e.to_string()))?;
    Ok(String::from_utf8_lossy(&text).into_owned())
}

/// The names of the legacy tolerance kinds 21 to 44.
fn tolerance_name(kind: i32) -> Option<&'static str> {
    Some(match kind {
        21 => "GeoTolAndGeoTolWthDatRefAndModGeoTolAndPosTol_1",
        22 => "GeoTolAndGeoTolWthDatRefAndModGeoTolAndPosTol_2",
        23 => "GeoTolAndGeoTolWthDatRefAndModGeoTolAndPosTol_3",
        24 => "AngularityTolerance",
        25 => "CircularRunoutTolerance",
        26 => "CoaxialityTolerance",
        27 => "ConcentricityTolerance",
        28 => "ParallelismTolerance",
        29 => "PerpendicularityTolerance",
        30 => "SymmetryTolerance",
        31 => "TotalRunoutTolerance",
        35 => "ModifiedGeometricTolerance_1",
        36 => "ModifiedGeometricTolerance_2",
        37 => "ModifiedGeometricTolerance_3",
        38 => "CylindricityTolerance",
        39 => "FlatnessTolerance",
        40 => "LineProfileTolerance",
        41 => "PositionTolerance",
        42 => "RoundnessTolerance",
        43 => "StraightnessTolerance",
        44 => "SurfaceProfileTolerance",
        _ => return None,
    })
}

/// A legacy dimension or tolerance: the value range of a dimension, the kind and first
/// value of a tolerance (`Standard_NullObject`/`Standard_OutOfRange` without the values).
fn dim_tol_info(dim_tol: &XCAFDocDimTol) -> Result<String, Failure> {
    let values = dim_tol.get_val();
    let array = rustycad_tklcaf::live(&values, "XCAFDoc::AttributeInfo")?.array1();
    let value = |index: i32| {
        array
            .value(index)
            .copied()
            .map_err(|_| Failure::out_of_range("NCollection_Array1::Value"))
    };
    let kind = dim_tol.get_kind();
    if kind < 20 {
        return Ok(format!(
            "Diameter (ValueRange[{},{}])",
            real(value(1)?),
            real(value(2)?)
        ));
    }
    Ok(match tolerance_name(kind) {
        Some(name) => format!("{name} (Value={})", real(value(1)?)),
        None => String::new(),
    })
}

/// A material: its name, with the density when not zero (`Standard_NullObject` without
/// name).
fn material_info(material: &XCAFDocMaterial) -> Result<String, Failure> {
    let name = material.get_name();
    let name = String::from_utf8_lossy(
        rustycad_tklcaf::live(&name, "XCAFDoc::AttributeInfo")?
            .string()
            .as_bytes(),
    )
    .into_owned();
    let density = material.get_density();
    if density == 0.0 {
        return Ok(name);
    }
    Ok(format!("{name}(density={}g/cu sm)", real(density)))
}

/// The description of the TDF and TDataStd attributes, `None` for another class.
fn standard_info(attribute: &Handle<dyn TdfAttribute>) -> Result<Option<String>, Failure> {
    if let Some(node) = attribute.cast::<TDataStdTreeNode>().get() {
        return Ok(Some(tree_node_info(node)?));
    }
    if let Some(reference) = attribute.cast::<TdfReference>().get() {
        return Ok(Some(format!("==> {}", entry_text(&reference.get()))));
    }
    if let Some(source) = attribute.cast::<TdfTagSource>().get() {
        return Ok(Some(source.get().to_string()));
    }
    if let Some(integer) = attribute.cast::<TDataStdInteger>().get() {
        return Ok(Some(integer.get().to_string()));
    }
    if let Some(value) = attribute.cast::<TDataStdReal>().get() {
        return Ok(Some(real(value.get())));
    }
    Ok(text_info(attribute))
}

/// The description of the string attributes (`?` for the non-ASCII characters).
fn text_info(attribute: &Handle<dyn TdfAttribute>) -> Option<String> {
    let ascii = |text: &rustycad_tkernel::tcollection::ExtendedString| {
        String::from_utf8_lossy(AsciiString::from_extended(text, Some(b'?')).as_bytes())
            .into_owned()
    };
    if let Some(name) = attribute.cast::<TDataStdName>().get() {
        return Some(ascii(&name.get()));
    }
    if let Some(comment) = attribute.cast::<TDataStdComment>().get() {
        return Some(ascii(&comment.get()));
    }
    let text = attribute.cast::<TDataStdAsciiString>();
    text.get()
        .map(|text| String::from_utf8_lossy(text.get().as_bytes()).into_owned())
}

/// The description of the arrays, named shapes and the XCAFDoc attributes.
fn xcaf_info(attribute: &Handle<dyn TdfAttribute>) -> Result<Option<String>, Failure> {
    if let Some(unit) = attribute.cast::<XCAFDocLengthUnit>().get() {
        let name = String::from_utf8_lossy(unit.get_unit_name().as_bytes()).into_owned();
        return Ok(Some(format!("{} {name}", real(unit.get_unit_value()))));
    }
    if let Some(array) = attribute.cast::<TDataStdIntegerArray>().get() {
        return Ok(Some(array_info(
            array.lower(),
            array.upper(),
            |i| array.value(i),
            |v| v.to_string(),
        )?));
    }
    if let Some(array) = attribute.cast::<TDataStdRealArray>().get() {
        return Ok(Some(array_info(
            array.lower(),
            array.upper(),
            |i| array.value(i),
            real,
        )?));
    }
    if let Some(array) = attribute.cast::<TDataStdByteArray>().get() {
        return Ok(Some(array_info(
            array.lower(),
            array.upper(),
            |i| array.value(i),
            |v| v.to_string(),
        )?));
    }
    if let Some(named) = attribute.cast::<TNamingNamedShape>().get() {
        return Ok(Some(named_shape_info(named)?));
    }
    if let Some(map) = attribute.cast::<XCAFDocShapeMapTool>().get() {
        return Ok(Some(map.get_map().extent().to_string()));
    }
    if let Some(centroid) = attribute.cast::<XCAFDocCentroid>().get() {
        let point = centroid.get();
        return Ok(Some(format!(
            "({} , {} , {})",
            real(point.x()),
            real(point.y()),
            real(point.z())
        )));
    }
    flag_and_object_info(attribute)
}

/// The description of the flags, colors, legacy GD&T, materials and graph nodes.
fn flag_and_object_info(attribute: &Handle<dyn TdfAttribute>) -> Result<Option<String>, Failure> {
    if let Some(flag) = attribute.cast::<TDataStdUAttribute>().get() {
        let mut info = String::new();
        if flag.id() == XCAFDoc::assembly_guid() {
            info += "is assembly";
        }
        if flag.id() == XCAFDoc::invisible_guid() {
            info += "invisible";
        }
        return Ok(Some(info));
    }
    if let Some(color) = attribute.cast::<XCAFDocColor>().get() {
        return Ok(Some(color_info(color)?));
    }
    if let Some(dim_tol) = attribute.cast::<XCAFDocDimTol>().get() {
        return Ok(Some(dim_tol_info(dim_tol)?));
    }
    if let Some(material) = attribute.cast::<XCAFDocMaterial>().get() {
        return Ok(Some(material_info(material)?));
    }
    attribute
        .cast::<XCAFDocGraphNode>()
        .get()
        .map(graph_node_info)
        .transpose()
}

impl XCAFDoc {
    /// `AttributeInfo(theAtt)`: a description of the attribute, empty for an attribute
    /// without one (`Standard_NullObject` for a null attribute).
    ///
    /// ```
    /// use rustycad_tklcaf::tdata_std::TDataStdInteger;
    /// use rustycad_tklcaf::tdf::TdfData;
    /// use rustycad_tkxcaf::xcaf_doc::XCAFDoc;
    /// let data = TdfData::new();
    /// let integer = TDataStdInteger::set(&data.root(), 1965)?;
    /// assert_eq!(XCAFDoc::attribute_info(&integer.cast())?.as_bytes(), b"1965");
    /// # Ok::<(), rustycad_tkernel::standard::Failure>(())
    /// ```
    pub fn attribute_info(attribute: &Handle<dyn TdfAttribute>) -> Result<AsciiString, Failure> {
        rustycad_tklcaf::live(attribute, "XCAFDoc::AttributeInfo")?;
        let info = match standard_info(attribute)? {
            Some(info) => info,
            None => xcaf_info(attribute)?.unwrap_or_default(),
        };
        Ok(AsciiString::from(info.as_str()))
    }
}
