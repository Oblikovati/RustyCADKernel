// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ShapeTool.hxx and .cxx (the attribute, the
// label predicates and the shape and structure queries) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `XCAFDoc_ShapeTool`: the tool of the shapes of an XDE document. Its label holds one child
//! per top-level shape; an assembly holds one child per component, a reference to the shape it
//! instantiates (a `TDataStd_TreeNode` of `ShapeRefGUID`) with an `XCAFDoc_Location`.
use super::generic_empty_tool::generic_empty_attribute;
use super::support::{find, find_or_add, live};
use super::{XCAFDoc, XCAFDocLocation};
use rustycad_tkbrep::topods::{Builder, Compound, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkcaf::tnaming::{TNamingNamedShape, TNamingTool};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tklcaf::tdata_std::{
    TDataStdGenericExtString, TDataStdName, TDataStdTreeNode, TDataStdUAttribute,
};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfChildIterator, TdfLabel};
use rustycad_tkmath::toploc::Location;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

/// A map from shapes (location-sensitive, orientation-free) to labels.
pub(crate) type ShapeLabelMap = DataMap<Shape, TdfLabel, ShapeMapHasher>;

/// The caches of `XCAFDoc_ShapeTool` (`myShapeLabels`, `mySubShapes`, `mySimpleShapes`,
/// `hasSimpleShapes`): transient, neither backed up nor restored, as natively.
#[derive(Default)]
pub struct ShapeToolCaches {
    pub(super) shape_labels: ShapeLabelMap,
    pub(super) sub_shapes: ShapeLabelMap,
    pub(super) simple_shapes: ShapeLabelMap,
    pub(super) has_simple_shapes: bool,
}

/// `theAutoNaming`: the file-static flag of `SetAutoNaming`, true by default.
static AUTO_NAMING: AtomicBool = AtomicBool::new(true);

generic_empty_attribute!(
    /// `XCAFDoc_ShapeTool`.
    XCAFDocShapeTool, "XCAFDoc_ShapeTool", "efd212ee-6dfd-11d4-b9c8-0060b0ee281b",
    state: ShapeToolCaches,
    dump: |tool: &XCAFDocShapeTool, stream: &mut DumpStream, depth: i32| -> io::Result<()> {
        rustycad_tkservice::json_dump::class_begin(stream, "XCAFDoc_ShapeTool")?;
        rustycad_tklcaf::json::dump_attribute_json_base(tool, stream, depth)?;
        super::shape_tool_dump::dump_caches(tool, stream)
    },
    text: |tool: &XCAFDocShapeTool, stream: &mut dyn io::Write| -> io::Result<()> {
        rustycad_tklcaf::tdf::dump_attribute(tool, stream)?;
        tool.dump_shapes(stream, false).map_err(io::Error::other)
    }
);

impl XCAFDocShapeTool {
    /// `Set(L)`: the tool of the label, added when absent, then `Init`.
    pub fn set(label: &TdfLabel) -> Result<Handle<XCAFDocShapeTool>, Failure> {
        let tool = find_or_add(label, &Self::get_id(), Self::new)?;
        live(&tool, "XCAFDoc_ShapeTool::Set")?.init();
        Ok(tool)
    }
    /// `Init`: the simple shapes are to be computed again.
    pub fn init(&self) {
        self.state.write().has_simple_shapes = false;
    }
    /// `SetAutoNaming(V)`: whether new shape labels get names (process-wide).
    pub fn set_auto_naming(value: bool) {
        AUTO_NAMING.store(value, Ordering::SeqCst);
    }
    /// `AutoNaming`.
    pub fn auto_naming() -> bool {
        AUTO_NAMING.load(Ordering::SeqCst)
    }
    /// `BaseLabel`: the label of the tool.
    pub fn base_label(&self) -> TdfLabel {
        self.label()
    }
    /// `IsTopLevel(L)`: a child of the tool's label.
    pub fn is_top_level(&self, label: &TdfLabel) -> bool {
        label.father() == self.label()
    }
}

/// The reference node of a label (`ShapeRefGUID`), or null.
pub(crate) fn shape_ref_node(label: &TdfLabel) -> Result<Handle<TDataStdTreeNode>, Failure> {
    find::<TDataStdTreeNode>(label, &XCAFDoc::shape_ref_guid())
}

/// The label predicates (static members natively).
impl XCAFDocShapeTool {
    /// `IsFree(L)`: no reference node, or a node without children (no user).
    pub fn is_free(label: &TdfLabel) -> Result<bool, Failure> {
        let node = shape_ref_node(label)?;
        Ok(node.get().is_none_or(|node| !node.has_first()))
    }
    /// `IsShape(L)`: a simple shape, an assembly or a reference.
    pub fn is_shape(label: &TdfLabel) -> Result<bool, Failure> {
        Ok(
            Self::is_simple_shape(label)?
                || Self::is_assembly(label)?
                || Self::is_reference(label)?,
        )
    }
    /// `IsSimpleShape(L)`: a named shape, neither an assembly nor a reference.
    pub fn is_simple_shape(label: &TdfLabel) -> Result<bool, Failure> {
        Ok(label.is_attribute(&TNamingNamedShape::get_id())?
            && !Self::is_assembly(label)?
            && !Self::is_reference(label)?)
    }
    /// `IsReference(L)`: a reference node with a father.
    pub fn is_reference(label: &TdfLabel) -> Result<bool, Failure> {
        Ok(shape_ref_node(label)?
            .get()
            .is_some_and(TDataStdTreeNode::has_father))
    }
    /// `IsAssembly(L)`: the assembly flag.
    pub fn is_assembly(label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&XCAFDoc::assembly_guid())
    }
    /// `IsComponent(L)`: a reference in an assembly.
    pub fn is_component(label: &TdfLabel) -> Result<bool, Failure> {
        Ok(Self::is_reference(label)? && Self::is_assembly(&label.father())?)
    }
    /// `IsCompound(L)`: named `COMPOUND`.
    pub fn is_compound(label: &TdfLabel) -> Result<bool, Failure> {
        let name = find::<TDataStdName>(label, &TDataStdName::get_id())?;
        Ok(name
            .get()
            .is_some_and(|name| name.get() == ExtendedString::from_utf8("COMPOUND")))
    }
    /// `IsSubShape(L)`: a simple shape under a shape.
    pub fn is_sub_shape_label(label: &TdfLabel) -> Result<bool, Failure> {
        Ok(Self::is_simple_shape(label)? && Self::is_shape(&label.father())?)
    }
    /// `IsExternRef(L)`: the external reference flag.
    pub fn is_extern_ref(label: &TdfLabel) -> Result<bool, Failure> {
        label.is_attribute(&XCAFDoc::extern_ref_guid())
    }
}

/// An empty compound.
pub(crate) fn empty_compound() -> Shape {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    compound.into_shape()
}

/// The shape queries.
impl XCAFDocShapeTool {
    /// `GetShape(L, S)`: the referred shape moved by the location for a reference, else the
    /// label's named shape (an empty compound first for an external reference); false
    /// without a shape.
    pub fn get_shape(label: &TdfLabel, shape: &mut Shape) -> Result<bool, Failure> {
        if Self::is_extern_ref(label)? {
            *shape = empty_compound();
        }
        let node = shape_ref_node(label)?;
        let location = find::<XCAFDocLocation>(label, &XCAFDocLocation::get_id())?;
        if let (Some(node), Some(location)) =
            (node.get().filter(|n| n.has_father()), location.get())
        {
            let father = node.father();
            if !Self::get_shape(
                &live(&father, "XCAFDoc_ShapeTool::GetShape")?.label(),
                shape,
            )? {
                return Ok(false);
            }
            shape.move_by(&location.get(), false)?;
            return Ok(true);
        }
        let named = find::<TNamingNamedShape>(label, &TNamingNamedShape::get_id())?;
        if named.is_null() {
            return Ok(false);
        }
        *shape = TNamingTool::get_shape(&named)?;
        Ok(true)
    }
    /// `GetShape(L)`: the shape, null without one.
    pub fn shape_of(label: &TdfLabel) -> Result<Shape, Failure> {
        let mut shape = Shape::default();
        Self::get_shape(label, &mut shape)?;
        Ok(shape)
    }
    /// `GetOneShape(theLabels)`: the shape of a single label, else a compound of the shapes
    /// of the labels (null when none has a shape).
    pub fn get_one_shape_of(labels: &[TdfLabel]) -> Result<Shape, Failure> {
        if labels.len() == 1 {
            return Self::shape_of(&labels[0]);
        }
        let mut compound = empty_compound();
        for label in labels {
            let mut shape = Shape::default();
            if Self::get_shape(label, &mut shape)? {
                Builder.add(&mut compound, &shape)?;
            }
        }
        Ok(if compound.nb_children() > 0 {
            compound
        } else {
            Shape::default()
        })
    }
    /// `GetOneShape()`: the one shape of the free shapes.
    pub fn get_one_shape(&self) -> Result<Shape, Failure> {
        Self::get_one_shape_of(&self.get_free_shapes()?)
    }
    /// `GetShapes(Labels)`: the children of the tool that hold a shape.
    pub fn get_shapes(&self) -> Result<Vec<TdfLabel>, Failure> {
        self.children_where(|label| {
            let mut shape = Shape::default();
            Self::get_shape(label, &mut shape)
        })
    }
    /// `GetFreeShapes(FreeLabels)`: the children of the tool that hold a shape and are free.
    pub fn get_free_shapes(&self) -> Result<Vec<TdfLabel>, Failure> {
        self.children_where(|label| {
            let mut shape = Shape::default();
            Ok(Self::get_shape(label, &mut shape)? && Self::is_free(label)?)
        })
    }
    /// The direct children of the tool's label that pass the test.
    fn children_where(
        &self,
        test: impl Fn(&TdfLabel) -> Result<bool, Failure>,
    ) -> Result<Vec<TdfLabel>, Failure> {
        let mut labels = Vec::new();
        let mut iterator = TdfChildIterator::new(&self.label(), false)?;
        while iterator.more() {
            let label = iterator.value();
            if test(&label)? {
                labels.push(label);
            }
            iterator.next();
        }
        Ok(labels)
    }
    /// `GetLocation(L)`: the label's `XCAFDoc_Location`, else the location of its named
    /// shape (the identity without one).
    pub fn get_location(label: &TdfLabel) -> Result<Location, Failure> {
        let location = find::<XCAFDocLocation>(label, &XCAFDocLocation::get_id())?;
        if let Some(location) = location.get() {
            return Ok(location.get());
        }
        let named = find::<TNamingNamedShape>(label, &TNamingNamedShape::get_id())?;
        if named.is_null() {
            return Ok(Location::default());
        }
        Ok(TNamingTool::get_shape(&named)?.location().clone())
    }
    /// `GetReferredShape(L, Label)`: the label of the referred shape of a reference.
    pub fn get_referred_shape(label: &TdfLabel, referred: &mut TdfLabel) -> Result<bool, Failure> {
        if !Self::is_reference(label)? {
            return Ok(false);
        }
        let node = shape_ref_node(label)?;
        let father = live(&node, "XCAFDoc_ShapeTool::GetReferredShape")?.father();
        *referred = live(&father, "XCAFDoc_ShapeTool::GetReferredShape")?.label();
        Ok(true)
    }
}

/// The users and components.
impl XCAFDocShapeTool {
    /// `GetUsers(L, Labels, getsubchilds)`: the references to the label's shape (with
    /// `getsubchilds`, the users of each user's assembly first); returns their count.
    pub fn get_users(
        label: &TdfLabel,
        labels: &mut Vec<TdfLabel>,
        sub_children: bool,
    ) -> Result<i32, Failure> {
        let mut count = 0;
        let node = shape_ref_node(label)?;
        let Some(node) = node.get() else {
            return Ok(count);
        };
        let mut user = node.first();
        while let Some(current) = user.get() {
            if sub_children {
                count += Self::get_users(&current.label().father(), labels, sub_children)?;
            }
            labels.push(current.label());
            user = current.next();
            count += 1;
        }
        Ok(count)
    }
    /// `NbComponents(L, getsubchilds)`.
    pub fn nb_components(label: &TdfLabel, sub_children: bool) -> Result<i32, Failure> {
        let mut labels = Vec::new();
        Self::get_components(label, &mut labels, sub_children)?;
        Ok(labels.len() as i32)
    }
    /// `GetComponents(L, Labels, getsubchilds)`: the components of an assembly (with
    /// `getsubchilds`, those of each referred assembly first); false for a non-assembly.
    pub fn get_components(
        label: &TdfLabel,
        labels: &mut Vec<TdfLabel>,
        sub_children: bool,
    ) -> Result<bool, Failure> {
        if !Self::is_assembly(label)? {
            return Ok(false);
        }
        let mut iterator = TdfChildIterator::new(label, false)?;
        while iterator.more() {
            let component = iterator.value();
            if Self::is_component(&component)? {
                let mut referred = TdfLabel::default();
                if sub_children && Self::get_referred_shape(&component, &mut referred)? {
                    Self::get_components(&referred, labels, sub_children)?;
                }
                labels.push(component);
            }
            iterator.next();
        }
        Ok(true)
    }
    /// `GetSubShapes(L, Labels)`: the sub-shape children; true when there is one.
    pub fn get_sub_shapes(label: &TdfLabel, labels: &mut Vec<TdfLabel>) -> Result<bool, Failure> {
        let mut iterator = TdfChildIterator::new(label, false)?;
        while iterator.more() {
            let sub = iterator.value();
            if Self::is_sub_shape_label(&sub)? {
                labels.push(sub);
            }
            iterator.next();
        }
        Ok(!labels.is_empty())
    }
    /// The assembly flag of a label (`TDataStd_UAttribute::Set(L, AssemblyGUID)`).
    pub(crate) fn set_assembly_flag(label: &TdfLabel) -> Result<(), Failure> {
        TDataStdUAttribute::set(label, &XCAFDoc::assembly_guid())?;
        Ok(())
    }
}
