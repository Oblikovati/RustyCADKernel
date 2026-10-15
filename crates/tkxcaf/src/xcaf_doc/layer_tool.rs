// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_LayerTool.hxx and .cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_LayerTool`: the layers of an XDE document (one named child of the tool's label
//! each) and the links of shapes to them (graph nodes of `LayerRefGUID`, the layer the
//! father).
use super::color_tool::entry_text;
use super::generic_empty_tool::generic_empty_attribute;
use super::graph_links::link_pair;
use super::support::{child_labels, find, find_or_add, live};
use super::tool_support::{cached_shape_tool, search_shape};
use super::{XCAFDoc, XCAFDocDocumentTool, XCAFDocGraphNode, XCAFDocShapeTool};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{TDataStdGenericExtString, TDataStdName, TDataStdUAttribute};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfLabel, TdfTagSource};
use rustycad_tkservice::json_dump::field_string;
use std::io;

generic_empty_attribute!(
    /// `XCAFDoc_LayerTool`.
    XCAFDocLayerTool, "XCAFDoc_LayerTool", "efd212f4-6dfd-11d4-b9c8-0060b0ee281b",
    state: Handle<XCAFDocShapeTool>,
    dump: |tool: &XCAFDocLayerTool, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        rustycad_tkservice::json_dump::class_begin(stream, "XCAFDoc_LayerTool")?;
        rustycad_tklcaf::json::dump_attribute_json_base(tool, stream, depth)?;
        for label in tool.get_layer_labels().map_err(io::Error::other)? {
            field_string(stream, "aLayerLabel", entry_text(&label).as_bytes())?;
        }
        Ok(())
    }
);

/// The name of a label, when it has one.
fn name_of(label: &TdfLabel) -> Result<Option<ExtendedString>, Failure> {
    let name = find::<TDataStdName>(label, &TDataStdName::get_id())?;
    Ok(name.get().map(TDataStdGenericExtString::get))
}

/// The layer graph node of a label, or null.
fn layer_node(label: &TdfLabel) -> Result<Handle<XCAFDocGraphNode>, Failure> {
    find::<XCAFDocGraphNode>(label, &XCAFDoc::layer_ref_guid())
}

/// The labels of the fathers (the layers) of the label's layer node.
fn layer_labels_of(label: &TdfLabel) -> Result<Vec<TdfLabel>, Failure> {
    let node = layer_node(label)?;
    let Some(node) = node.get() else {
        return Ok(Vec::new());
    };
    node.fathers()
        .iter()
        .map(|father| Ok(live(father, "XCAFDoc_LayerTool")?.label()))
        .collect()
}

impl XCAFDocLayerTool {
    /// `Set(L)`: the tool of the label, created (with the document's shape tool) when absent.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocLayerTool>, Failure> {
        let existing = find::<XCAFDocLayerTool>(label, &Self::get_id())?;
        if !existing.is_null() {
            return Ok(existing);
        }
        let tool = find_or_add(label, &Self::get_id(), Self::new)?;
        *live(&tool, "XCAFDoc_LayerTool::Set")?.state.write() =
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
    /// `IsLayer(lab)`: a named child of the tool's label.
    pub fn is_layer(&self, label: &TdfLabel) -> Result<bool, Failure> {
        Ok(self.get_layer(label)?.is_some())
    }
    /// `GetLayer(lab, aLayer)`: the name of a layer label.
    pub fn get_layer(&self, label: &TdfLabel) -> Result<Option<ExtendedString>, Failure> {
        if label.father() != self.label() {
            return Ok(None);
        }
        name_of(label)
    }
    /// `FindLayer(aLayer, lab)`: the first layer of that name.
    pub fn find_layer(
        &self,
        layer: &ExtendedString,
        label: &mut TdfLabel,
    ) -> Result<bool, Failure> {
        *label = self.find_layer_with(layer, false, false)?;
        Ok(!label.is_null())
    }
    /// `FindLayer(aLayer, theToFindWithProperty, theToFindVisible)`: the first child of that
    /// name (with that visibility when asked), null when none.
    pub fn find_layer_with(
        &self,
        layer: &ExtendedString,
        with_property: bool,
        visible: bool,
    ) -> Result<TdfLabel, Failure> {
        for label in child_labels(&self.label())? {
            let named = name_of(&label)?.is_some_and(|name| name == *layer);
            if named && (!with_property || self.is_visible(&label)? == visible) {
                return Ok(label);
            }
        }
        Ok(TdfLabel::default())
    }
    /// `AddLayer(theLayer)`: the layer of that name, or a new named child.
    pub fn add_layer(&self, layer: &ExtendedString) -> Result<TdfLabel, Failure> {
        let mut label = TdfLabel::default();
        if self.find_layer(layer, &mut label)? {
            return Ok(label);
        }
        self.new_layer(layer)
    }
    /// `AddLayer(theLayer, theToFindVisible)`: the layer of that name and visibility, or a new
    /// named child.
    pub fn add_layer_visible(
        &self,
        layer: &ExtendedString,
        visible: bool,
    ) -> Result<TdfLabel, Failure> {
        let label = self.find_layer_with(layer, true, visible)?;
        if !label.is_null() {
            return Ok(label);
        }
        self.new_layer(layer)
    }
    /// A new child of the tool's label with that name.
    fn new_layer(&self, layer: &ExtendedString) -> Result<TdfLabel, Failure> {
        let label = TdfTagSource::new_child_of(&self.label())?;
        TDataStdName::set(&label, layer)?;
        Ok(label)
    }
    /// `RemoveLayer(lab)`: forgets every attribute of the layer label.
    pub fn remove_layer(&self, label: &TdfLabel) -> Result<(), Failure> {
        label.forget_all_attributes(true)
    }
    /// `GetLayerLabels(Labels)`: the layer labels.
    pub fn get_layer_labels(&self) -> Result<Vec<TdfLabel>, Failure> {
        let mut labels = Vec::new();
        for label in child_labels(&self.label())? {
            if self.is_layer(&label)? {
                labels.push(label);
            }
        }
        Ok(labels)
    }
}

/// The layer links of labels.
impl XCAFDocLayerTool {
    /// `SetLayer(L, LayerL, shapeInOneLayer)`: links the label to the layer (after removing
    /// its other layers when `shapeInOneLayer`).
    pub fn set_layer_label(
        &self,
        label: &TdfLabel,
        layer: &TdfLabel,
        one_layer: bool,
    ) -> Result<(), Failure> {
        if one_layer {
            self.unset_layers(label)?;
        }
        link_pair(layer, label, &XCAFDoc::layer_ref_guid())
    }
    /// `SetLayer(L, aLayer, shapeInOneLayer)`: links the label to the (added) layer.
    pub fn set_layer(
        &self,
        label: &TdfLabel,
        layer: &ExtendedString,
        one_layer: bool,
    ) -> Result<(), Failure> {
        let layer_label = self.add_layer(layer)?;
        self.set_layer_label(label, &layer_label, one_layer)
    }
    /// `UnSetLayers(L)`: unlinks the label from all its layers and forgets its node.
    pub fn unset_layers(&self, label: &TdfLabel) -> Result<(), Failure> {
        let child = layer_node(label)?;
        let Some(node) = child.get() else {
            return Ok(());
        };
        while node.nb_fathers() != 0 {
            let father = node.get_father(1)?;
            live(&father, "XCAFDoc_LayerTool::UnSetLayers")?.unset_child(&child)?;
        }
        label.forget_attribute_id(&XCAFDoc::layer_ref_guid())?;
        Ok(())
    }
    /// `UnSetOneLayer(L, aLayer)`: unlinks the label from the layer of that name.
    pub fn unset_one_layer(
        &self,
        label: &TdfLabel,
        layer: &ExtendedString,
    ) -> Result<bool, Failure> {
        let mut layer_label = TdfLabel::default();
        if !self.find_layer(layer, &mut layer_label)? {
            return Ok(false);
        }
        self.unset_one_layer_label(label, &layer_label)
    }
    /// `UnSetOneLayer(L, aLayerL)`: unlinks the label from that layer.
    pub fn unset_one_layer_label(
        &self,
        label: &TdfLabel,
        layer: &TdfLabel,
    ) -> Result<bool, Failure> {
        let child = layer_node(label)?;
        let father = layer_node(layer)?;
        if child.is_null() || father.is_null() {
            return Ok(false);
        }
        live(&child, "XCAFDoc_LayerTool::UnSetOneLayer")?.unset_father(&father)?;
        Ok(true)
    }
    /// `IsSet(L, aLayer)`: a layer of the label has that name.
    pub fn is_set(&self, label: &TdfLabel, layer: &ExtendedString) -> Result<bool, Failure> {
        for father in layer_labels_of(label)? {
            if name_of(&father)?.is_some_and(|name| name == *layer) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// `IsSet(L, aLayerL)`: the layer is one of the label's.
    pub fn is_set_label(&self, label: &TdfLabel, layer: &TdfLabel) -> Result<bool, Failure> {
        Ok(layer_labels_of(label)?.contains(layer))
    }
    /// `GetLayers(L, aLayerLS)`: the layer labels of the label.
    pub fn get_layer_labels_of(label: &TdfLabel) -> Result<Vec<TdfLabel>, Failure> {
        layer_labels_of(label)
    }
    /// `GetLayers(L)` (and `GetLayers(L, aLayerS)`, true when not empty): the names of the
    /// label's layers that have one.
    pub fn get_layers(label: &TdfLabel) -> Result<Vec<ExtendedString>, Failure> {
        let mut names = Vec::new();
        for layer in layer_labels_of(label)? {
            names.extend(name_of(&layer)?);
        }
        Ok(names)
    }
    /// `GetShapesOfLayer(theLayerL, theShLabels)`: the labels linked to the layer.
    pub fn get_shapes_of_layer(layer: &TdfLabel) -> Result<Vec<TdfLabel>, Failure> {
        let node = layer_node(layer)?;
        let Some(node) = node.get() else {
            return Ok(Vec::new());
        };
        node.children()
            .iter()
            .map(|child| Ok(live(child, "XCAFDoc_LayerTool")?.label()))
            .collect()
    }
    /// `IsVisible(layerL)`: no invisibility flag.
    pub fn is_visible(&self, layer: &TdfLabel) -> Result<bool, Failure> {
        Ok(!layer.is_attribute(&XCAFDoc::invisible_guid())?)
    }
    /// `SetVisibility(layerL, isvisible)`: sets (when absent) or forgets the invisibility flag.
    pub fn set_visibility(&self, layer: &TdfLabel, visible: bool) -> Result<(), Failure> {
        let guid = XCAFDoc::invisible_guid();
        if visible {
            layer.forget_attribute_id(&guid)?;
        } else if !layer.is_attribute(&guid)? {
            TDataStdUAttribute::set(layer, &guid)?;
        }
        Ok(())
    }
}

/// The layer links of shapes (through `myShapeTool->Search`, without the lazy lookup).
impl XCAFDocLayerTool {
    /// The label of a shape, `None` when not found.
    fn search(&self, shape: &Shape) -> Result<Option<TdfLabel>, Failure> {
        let tool = self.state.read().clone();
        search_shape(&tool, shape, "XCAFDoc_LayerTool")
    }
    /// `SetLayer(Sh, LayerL, shapeInOneLayer)`.
    pub fn set_shape_layer_label(
        &self,
        shape: &Shape,
        layer: &TdfLabel,
        one_layer: bool,
    ) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.set_layer_label(&label, layer, one_layer)?;
        Ok(true)
    }
    /// `SetLayer(Sh, aLayer, shapeInOneLayer)`: the layer is added even when the shape is not
    /// found.
    pub fn set_shape_layer(
        &self,
        shape: &Shape,
        layer: &ExtendedString,
        one_layer: bool,
    ) -> Result<bool, Failure> {
        let layer_label = self.add_layer(layer)?;
        self.set_shape_layer_label(shape, &layer_label, one_layer)
    }
    /// `UnSetLayers(Sh)`.
    pub fn unset_shape_layers(&self, shape: &Shape) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.unset_layers(&label)?;
        Ok(true)
    }
    /// `UnSetOneLayer(Sh, aLayer)`.
    pub fn unset_shape_one_layer(
        &self,
        shape: &Shape,
        layer: &ExtendedString,
    ) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.unset_one_layer(&label, layer)
    }
    /// `UnSetOneLayer(Sh, aLayerL)`.
    pub fn unset_shape_one_layer_label(
        &self,
        shape: &Shape,
        layer: &TdfLabel,
    ) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.unset_one_layer_label(&label, layer)
    }
    /// `IsSet(Sh, aLayer)`.
    pub fn is_shape_set(&self, shape: &Shape, layer: &ExtendedString) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.is_set(&label, layer)
    }
    /// `IsSet(Sh, aLayerL)`.
    pub fn is_shape_set_label(&self, shape: &Shape, layer: &TdfLabel) -> Result<bool, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(false);
        };
        self.is_set_label(&label, layer)
    }
    /// `GetLayers(Sh, aLayerLS)`: `None` when the shape is not found.
    pub fn get_shape_layer_labels(&self, shape: &Shape) -> Result<Option<Vec<TdfLabel>>, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(None);
        };
        Ok(Some(layer_labels_of(&label)?))
    }
    /// `GetLayers(Sh)` (and `GetLayers(Sh, aLayerS)`): the layer names of the shape, empty
    /// when it is not found.
    pub fn get_shape_layers(&self, shape: &Shape) -> Result<Vec<ExtendedString>, Failure> {
        let Some(label) = self.search(shape)? else {
            return Ok(Vec::new());
        };
        Self::get_layers(&label)
    }
}
