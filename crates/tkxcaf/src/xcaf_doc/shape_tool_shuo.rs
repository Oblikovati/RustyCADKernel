// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ShapeTool.cxx (external references, the
// specified higher usage occurrences (SHUO) and FindComponent) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! External references and SHUO: a SHUO is a chain of `XCAFDoc_GraphNode`s of `SHUORefGUID`
//! on sub-labels of components, from the upper usage to the next usages.
use super::shape_tool::XCAFDocShapeTool;
use super::shape_tool_add::unlocated;
use super::support::{find, live};
use super::{XCAFDoc, XCAFDocGraphNode};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::Mat4;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::{AsciiString, ExtendedString, HAsciiString};
use rustycad_tklcaf::tdata_std::{TDataStdGenericExtString, TDataStdName, TDataStdUAttribute};
use rustycad_tklcaf::tdf::{
    TdfAttribute, TdfAttributeState, TdfChildIterator, TdfLabel, TdfTagSource,
};
use rustycad_tkmath::toploc::Location;

/// The names of an external reference on the children `1..` of its label.
fn set_names(label: &TdfLabel, names: &[Handle<HAsciiString>]) -> Result<(), Failure> {
    for (index, name) in names.iter().enumerate() {
        let child = label.find_child(index as i32 + 1, true)?;
        let text = live(name, "XCAFDoc_ShapeTool::SetExternRefs")?
            .string()
            .clone();
        TDataStdName::set(&child, &ExtendedString::from_ascii(&text))?;
    }
    Ok(())
}

/// External references.
impl XCAFDocShapeTool {
    /// `SetExternRefs(L, SHAS)`: a new child of `L` flagged as an external reference, with
    /// the names.
    pub fn set_extern_refs_on(
        &self,
        label: &TdfLabel,
        names: &[Handle<HAsciiString>],
    ) -> Result<(), Failure> {
        let shape_label = label.new_child()?;
        TDataStdUAttribute::set(&shape_label, &XCAFDoc::extern_ref_guid())?;
        set_names(&shape_label, names)
    }
    /// `SetExternRefs(SHAS)`: a new top-level external reference with the names.
    pub fn set_extern_refs(&self, names: &[Handle<HAsciiString>]) -> Result<TdfLabel, Failure> {
        let shape_label = TdfTagSource::new_child_of(&self.label())?;
        TDataStdUAttribute::set(&shape_label, &XCAFDoc::extern_ref_guid())?;
        set_names(&shape_label, names)?;
        Ok(shape_label)
    }
    /// `GetExternRefs(L, SHAS)`: appends the names of the children `1..=NbChildren`.
    pub fn get_extern_refs(
        label: &TdfLabel,
        names: &mut Vec<Handle<HAsciiString>>,
    ) -> Result<(), Failure> {
        for index in 1..=label.nb_children()? {
            let child = label.find_child(index, true)?;
            let name = find::<TDataStdName>(&child, &TDataStdName::get_id())?;
            if let Some(name) = name.get() {
                let text = AsciiString::from_extended(&name.get(), None);
                names.push(Handle::new(HAsciiString::from_string(&text)));
            }
        }
        Ok(())
    }
}

/// The SHUO.
impl XCAFDocShapeTool {
    /// `GetSHUO(SHUOLabel, aSHUOAttr)`: the SHUO node of the label.
    pub fn get_shuo(
        label: &TdfLabel,
        shuo: &mut Handle<XCAFDocGraphNode>,
    ) -> Result<bool, Failure> {
        label.find_attribute_as(&XCAFDoc::shuo_ref_guid(), shuo)
    }
    /// `GetAllComponentSHUO(CompLabel, SHUOAttrs)`: the SHUO nodes of the children.
    pub fn get_all_component_shuo(
        label: &TdfLabel,
        nodes: &mut Vec<Handle<dyn TdfAttribute>>,
    ) -> Result<bool, Failure> {
        let mut iterator = TdfChildIterator::new(label, false)?;
        while iterator.more() {
            let mut shuo = Handle::default();
            if Self::get_shuo(&iterator.value(), &mut shuo)? {
                nodes.push(shuo.cast());
            }
            iterator.next();
        }
        Ok(!nodes.is_empty())
    }
    /// `SetSHUO(Labels, MainSHUOAttr)`: a chain of SHUO nodes on new sub-labels of the
    /// components (named `SHUO`, `SHUO-2`... when naming automatically); false with fewer
    /// than two labels or a label that is not a component.
    pub fn set_shuo(
        &self,
        labels: &[TdfLabel],
        main: &mut Handle<XCAFDocGraphNode>,
    ) -> Result<bool, Failure> {
        *main = Handle::default();
        if labels.len() < 2 {
            return Ok(false);
        }
        for label in labels {
            if !Self::is_component(label)? {
                return Ok(false);
            }
        }
        let upper_label = TdfTagSource::new_child_of(&labels[0])?;
        if Self::auto_naming() {
            TDataStdName::set(&upper_label, &ExtendedString::from_utf8("SHUO"))?;
        }
        let mut upper = XCAFDocGraphNode::set_with_id(&upper_label, &XCAFDoc::shuo_ref_guid())?;
        *main = upper.clone();
        for (index, label) in labels.iter().enumerate().skip(1) {
            let next_label = TdfTagSource::new_child_of(label)?;
            if Self::auto_naming() {
                let name = format!("SHUO-{}", index + 1);
                TDataStdName::set(&next_label, &ExtendedString::from_utf8(&name))?;
            }
            let next = XCAFDocGraphNode::set_with_id(&next_label, &XCAFDoc::shuo_ref_guid())?;
            live(&upper, "XCAFDoc_ShapeTool::SetSHUO")?.set_child(&next)?;
            live(&next, "XCAFDoc_ShapeTool::SetSHUO")?.set_father(&upper)?;
            upper = next;
        }
        Ok(true)
    }
    /// `GetSHUOUpperUsage(NextUsageL, Labels)`: the labels of the fathers of the SHUO.
    pub fn get_shuo_upper_usage(
        label: &TdfLabel,
        labels: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        let mut shuo = Handle::default();
        if !Self::get_shuo(label, &mut shuo)? || live(&shuo, "GetSHUOUpperUsage")?.nb_fathers() < 1
        {
            return Ok(false);
        }
        for father in live(&shuo, "GetSHUOUpperUsage")?.fathers() {
            labels.push(live(&father, "XCAFDoc_ShapeTool::GetSHUOUpperUsage")?.label());
        }
        Ok(true)
    }
    /// `GetSHUONextUsage(UpperUsageL, Labels)`: the labels of the children of the SHUO.
    pub fn get_shuo_next_usage(
        label: &TdfLabel,
        labels: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        let mut shuo = Handle::default();
        if !Self::get_shuo(label, &mut shuo)? || live(&shuo, "GetSHUONextUsage")?.nb_children() < 1
        {
            return Ok(false);
        }
        for child in live(&shuo, "GetSHUONextUsage")?.children() {
            labels.push(live(&child, "XCAFDoc_ShapeTool::GetSHUONextUsage")?.label());
        }
        Ok(true)
    }
    /// `RemoveSHUO(SHUOLabel)`: forgets every attribute of the label.
    pub fn remove_shuo(&self, label: &TdfLabel) -> Result<bool, Failure> {
        label.forget_all_attributes(true)?;
        Ok(true)
    }
    /// `FindSHUO(Labels, theSHUOAttr)`: the SHUO of the first component whose chain goes
    /// through exactly the given components.
    pub fn find_shuo(
        labels: &[TdfLabel],
        shuo: &mut Handle<XCAFDocGraphNode>,
    ) -> Result<bool, Failure> {
        let mut nodes = Vec::new();
        let first = labels.first().ok_or_else(|| {
            rustycad_tkernel::standard::exceptions::out_of_range(
                "XCAFDoc_ShapeTool::FindSHUO: empty label sequence",
            )
        })?;
        if !Self::get_all_component_shuo(first, &mut nodes)? {
            return Ok(false);
        }
        for node in nodes {
            let start = node.cast::<XCAFDocGraphNode>();
            if shuo_chain(&start)?.as_slice() == labels {
                *shuo = start;
                break;
            }
        }
        Ok(!shuo.is_null())
    }
}

/// The components a SHUO chain goes through (the fathers of the node labels).
fn shuo_chain(start: &Handle<XCAFDocGraphNode>) -> Result<Vec<TdfLabel>, Failure> {
    let mut node = start.clone();
    let mut chain = vec![live(&node, "XCAFDoc_ShapeTool::FindSHUO")?.label().father()];
    while live(&node, "XCAFDoc_ShapeTool::FindSHUO")?.nb_children() > 0 {
        node = live(&node, "XCAFDoc_ShapeTool::FindSHUO")?.get_child(1)?;
        chain.push(live(&node, "XCAFDoc_ShapeTool::FindSHUO")?.label().father());
    }
    Ok(chain)
}

/// `Transformation().GetMat4(aMat4)`.
fn mat4_of(transform: &rustycad_tkmath::gp::Transform3) -> Mat4<f64> {
    let mut matrix = Mat4::<f64>::default();
    transform.get_mat4(&mut matrix);
    matrix
}

/// The file-static `IsEqual(theLoc1, theLoc2)`: equal locations, or the same chain of datums
/// with equal matrices and powers.
fn locations_equal(first: &Location, second: &Location) -> Result<bool, Failure> {
    if first.is_equal(second) {
        return Ok(true);
    }
    if first.is_identity() || second.is_identity() {
        return Ok(false);
    }
    let (datum1, datum2) = (first.first_datum()?, second.first_datum()?);
    match (datum1.get(), datum2.get()) {
        (Some(d1), Some(d2)) => {
            if !mat4_of(d1.transformation()).is_equal(&mat4_of(d2.transformation())) {
                return Ok(false);
            }
        }
        (None, None) => {}
        _ => return Ok(false),
    }
    if first.first_power()? != second.first_power()? {
        return Ok(false);
    }
    locations_equal(first.next_location(), second.next_location())
}

/// The file-static `IsSame(theShape1, theShape2)`: the same TShape, orientation and an equal
/// location.
fn shapes_same(first: &Shape, second: &Shape) -> Result<bool, Failure> {
    Ok(
        first.tshape().object_address() == second.tshape().object_address()
            && first.orientation() == second.orientation()
            && locations_equal(first.location(), second.location())?,
    )
}

/// `checkForShape(theShape, theCurSh, theUserL, theLabels)`: the user, located in its upper
/// assembly, is the shape (its label goes first), or one of the users of that assembly is.
fn check_for_shape(
    shape: &Shape,
    current: &Shape,
    user: &TdfLabel,
    labels: &mut Vec<TdfLabel>,
) -> Result<bool, Failure> {
    let super_user = user.father();
    let component_location =
        XCAFDocShapeTool::get_location(user)?.multiplied(current.location())?;
    let super_location =
        XCAFDocShapeTool::get_location(&super_user)?.multiplied(&component_location)?;
    let mut copy = current.clone();
    copy.set_location(&super_location, false)?;
    if shapes_same(shape, &copy)? {
        labels.insert(0, user.clone());
        return Ok(true);
    }
    let mut new_labels = labels.clone();
    new_labels.insert(0, user.clone());
    let mut users = Vec::new();
    XCAFDocShapeTool::get_users(&super_user, &mut users, false)?;
    for next_user in users {
        if check_for_shape(shape, &copy, &next_user, &mut new_labels)? {
            *labels = new_labels;
            return Ok(true);
        }
    }
    Ok(false)
}

impl XCAFDocShapeTool {
    /// `FindComponent(theShape, Labels)`: the chain of components (upper first) through
    /// which the located shape is an instance of its top-level shape.
    pub fn find_component(
        &self,
        shape: &Shape,
        labels: &mut Vec<TdfLabel>,
    ) -> Result<bool, Failure> {
        labels.clear();
        let unlocated_shape = unlocated(shape)?;
        let referred = self.find_shape_label(&unlocated_shape, false)?;
        if referred.is_null() {
            return Ok(false); // cannot find top-level shape.
        }
        let mut users = Vec::new();
        Self::get_users(&referred, &mut users, false)?;
        for user in users {
            if check_for_shape(shape, &unlocated_shape, &user, labels)? {
                break;
            }
        }
        Ok(!labels.is_empty())
    }
    /// `SetInstanceSHUO(theShape)`: the SHUO of the components of the located shape, or null.
    pub fn set_instance_shuo(&self, shape: &Shape) -> Result<Handle<XCAFDocGraphNode>, Failure> {
        let mut shuo = Handle::default();
        let mut labels = Vec::new();
        if self.find_component(shape, &mut labels)? {
            self.set_shuo(&labels, &mut shuo)?;
        }
        Ok(shuo)
    }
}
