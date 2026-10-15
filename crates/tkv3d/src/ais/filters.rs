//! The AIS selection filters (`AIS_TypeFilter`, `AIS_SignatureFilter`, `AIS_AttributeFilter`,
//! `AIS_ExclusionFilter`, `AIS_BadEdgeFilter`, `AIS_C0RegularityFilter`), `AIS_GraphicTool`
//! and the small AIS enumerations they use.
use super::{AisInteractiveObject, AisKindOfInteractive};
use crate::prs_mgr::{
    PrsMgrDisplayStatus, PrsMgrPresentableObject, PrsMgrPresentableObjectMembers,
};
use crate::prs3d::{Prs3dDatumParts, Prs3dDrawer};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrFilter};
use crate::std_select::{StdSelectBRepOwnerMembers, StdSelectBRepOwnerSource};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::DataMap;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkservice::aspect::AspectTypeOfLine;
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectLine3d, Graphic3dAspectsMembers, Graphic3dMaterialAspect,
};

/// `AIS_DisplayMode`: the neutral display modes of the shapes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AisDisplayMode {
    /// `AIS_WireFrame`.
    #[default]
    WireFrame = 0,
    /// `AIS_Shaded`.
    Shaded = 1,
}

/// `AIS_DisplayStatus`: the display status of an interactive object.
pub type AisDisplayStatus = PrsMgrDisplayStatus;

/// `AIS_SelectionModesConcurrency`: how a new selection mode treats the active ones.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AisSelectionModesConcurrency {
    /// `AIS_SelectionModesConcurrency_Single`: the other modes are deactivated.
    #[default]
    Single = 0,
    /// `AIS_SelectionModesConcurrency_GlobalOrLocal`: the global mode or local modes.
    GlobalOrLocal = 1,
    /// `AIS_SelectionModesConcurrency_Multiple`: any combination.
    Multiple = 2,
}

/// `AIS_TypeOfAttribute`: the line attributes of a drawer read by `AIS_GraphicTool`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AisTypeOfAttribute {
    /// `AIS_TOA_Line`.
    Line = 0,
    /// `AIS_TOA_Dimension`.
    Dimension,
    /// `AIS_TOA_Wire`.
    Wire,
    /// `AIS_TOA_Plane`.
    Plane,
    /// `AIS_TOA_Vector`.
    Vector,
    /// `AIS_TOA_UIso`.
    UIso,
    /// `AIS_TOA_VIso`.
    VIso,
    /// `AIS_TOA_Free`.
    Free,
    /// `AIS_TOA_UnFree`.
    UnFree,
    /// `AIS_TOA_Section`.
    Section,
    /// `AIS_TOA_Hidden`.
    Hidden,
    /// `AIS_TOA_Seen`.
    Seen,
    /// `AIS_TOA_FaceBoundary`.
    FaceBoundary,
    /// `AIS_TOA_FirstAxis`.
    FirstAxis,
    /// `AIS_TOA_SecondAxis`.
    SecondAxis,
    /// `AIS_TOA_ThirdAxis`.
    ThirdAxis,
}

/// The interactive object selected by an owner, None for another selectable.
fn owner_object(
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Option<Handle<dyn AisInteractiveObject>> {
    let object = owner.get()?.selectable().cast::<dyn AisInteractiveObject>();
    (!object.is_null()).then_some(object)
}

/// The shape of a BRep owner, None for another owner.
fn brep_shape(owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Option<Shape> {
    let brep = owner.cast::<dyn StdSelectBRepOwnerSource>();
    brep.get().map(|owner| owner.shape_ref().clone_shape())
}

/// `AIS_TypeFilter`: accepts the interactive objects of a kind.
/// Example: `AisTypeFilter::new(AisKindOfInteractive::Shape)`.
pub struct AisTypeFilter {
    transient: StandardTransient,
    kind: AisKindOfInteractive,
}
rustycad_tkernel::impl_transient!(AisTypeFilter, "AIS_TypeFilter", dyn SelectMgrFilter, transient,
    interfaces: [dyn SelectMgrFilter]);

impl AisTypeFilter {
    /// The filter of a kind; `AisTypeFilter::new(AisKindOfInteractive::Datum)`.
    pub fn new(kind: AisKindOfInteractive) -> Self {
        Self {
            transient: StandardTransient::default(),
            kind,
        }
    }
    /// `myKind`: the accepted kind; `filter.kind()`.
    pub fn kind(&self) -> AisKindOfInteractive {
        self.kind
    }
}

impl SelectMgrFilter for AisTypeFilter {
    /// `IsOk`: the selectable is an interactive object of the kind.
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<bool, Failure> {
        Ok(owner_object(owner)
            .and_then(|object| object.get().map(|o| o.kind_of_interactive() == self.kind))
            .unwrap_or(false))
    }
}

/// `AIS_SignatureFilter`: accepts the interactive objects of a kind and a signature.
/// Example: `AisSignatureFilter::new(AisKindOfInteractive::Shape, 0)`.
pub struct AisSignatureFilter {
    parent: AisTypeFilter,
    signature: i32,
}
rustycad_tkernel::impl_transient!(AisSignatureFilter, "AIS_SignatureFilter", AisTypeFilter, parent,
    interfaces: [dyn SelectMgrFilter]);

impl AisSignatureFilter {
    /// The filter of a kind and a signature; `AisSignatureFilter::new(kind, 2)`.
    pub fn new(kind: AisKindOfInteractive, signature: i32) -> Self {
        Self {
            parent: AisTypeFilter::new(kind),
            signature,
        }
    }
    /// The type filter part (`myKind`); `filter.type_filter().kind()`.
    pub fn type_filter(&self) -> &AisTypeFilter {
        &self.parent
    }
    /// `mySig`: the accepted signature; `filter.signature()`.
    pub fn signature(&self) -> i32 {
        self.signature
    }
}

impl SelectMgrFilter for AisSignatureFilter {
    /// `IsOk`: the selectable is an interactive object of the signature and kind.
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<bool, Failure> {
        Ok(owner_object(owner)
            .and_then(|object| {
                object.get().map(|o| {
                    o.signature() == self.signature && o.kind_of_interactive() == self.parent.kind
                })
            })
            .unwrap_or(false))
    }
}

/// The color and width criteria of an attribute filter.
#[derive(Clone, Copy, Debug)]
struct AttributeCriteria {
    color: NamedColor,
    width: f64,
    has_color: bool,
    has_width: bool,
}

/// `AIS_AttributeFilter`: accepts the interactive objects whose own color and width (when set
/// on the object) are the filter's. Example: `AisAttributeFilter::with_color(NamedColor::RED)`.
pub struct AisAttributeFilter {
    transient: StandardTransient,
    criteria: SharedState<AttributeCriteria>,
}
rustycad_tkernel::impl_transient!(AisAttributeFilter, "AIS_AttributeFilter", dyn SelectMgrFilter, transient,
    interfaces: [dyn SelectMgrFilter]);

impl Default for AisAttributeFilter {
    /// No criterion (the native color and width are left uninitialized and never read).
    fn default() -> Self {
        Self::with_criteria(NamedColor::BLACK, 0., false, false)
    }
}

impl AisAttributeFilter {
    fn with_criteria(color: NamedColor, width: f64, has_color: bool, has_width: bool) -> Self {
        Self {
            transient: StandardTransient::default(),
            criteria: SharedState::new(AttributeCriteria {
                color,
                width,
                has_color,
                has_width,
            }),
        }
    }
    /// The filter of a color; `AisAttributeFilter::with_color(NamedColor::RED)`.
    pub fn with_color(color: NamedColor) -> Self {
        Self::with_criteria(color, 0., true, false)
    }
    /// The filter of a width; `AisAttributeFilter::with_width(2.)`.
    pub fn with_width(width: f64) -> Self {
        Self::with_criteria(NamedColor::BLACK, width, false, true)
    }
    /// `HasColor`; `filter.has_color()`.
    pub fn has_color(&self) -> bool {
        self.criteria.read().has_color
    }
    /// `HasWidth`; `filter.has_width()`.
    pub fn has_width(&self) -> bool {
        self.criteria.read().has_width
    }
    /// `SetColor`; `filter.set_color(NamedColor::GREEN)`.
    pub fn set_color(&self, color: NamedColor) {
        let mut criteria = self.criteria.write();
        (criteria.color, criteria.has_color) = (color, true);
    }
    /// `SetWidth`; `filter.set_width(1.5)`.
    pub fn set_width(&self, width: f64) {
        let mut criteria = self.criteria.write();
        (criteria.width, criteria.has_width) = (width, true);
    }
    /// `UnsetColor`; `filter.unset_color()`.
    pub fn unset_color(&self) {
        self.criteria.write().has_color = false;
    }
    /// `UnsetWidth`; `filter.unset_width()`.
    pub fn unset_width(&self) {
        self.criteria.write().has_width = false;
    }
}

impl SelectMgrFilter for AisAttributeFilter {
    /// `IsOk`: an interactive object whose set color has the filter's name and whose set
    /// width is the filter's.
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<bool, Failure> {
        let Some(object) = owner_object(owner) else {
            return Ok(false);
        };
        let Some(object) = object.get() else {
            return Ok(false);
        };
        let criteria = *self.criteria.read();
        let mut ok = true;
        if criteria.has_color && object.has_color() {
            ok = criteria.color == PrsMgrPresentableObject::color(object).name();
        }
        if criteria.has_width && object.has_width() {
            ok = criteria.width == object.width() && ok;
        }
        Ok(ok)
    }
}

/// `AIS_ExclusionFilter`: refuses (or, without the exclusion flag, accepts only) the kinds
/// and signatures stored. Example: `AisExclusionFilter::new(true)`.
pub struct AisExclusionFilter {
    transient: StandardTransient,
    state: SharedState<ExclusionState>,
}
rustycad_tkernel::impl_transient!(AisExclusionFilter, "AIS_ExclusionFilter", dyn SelectMgrFilter, transient,
    interfaces: [dyn SelectMgrFilter]);

struct ExclusionState {
    is_exclusion_flag_on: bool,
    stored_types: DataMap<i32, Vec<i32>>,
}

impl AisExclusionFilter {
    /// A filter without stored kind: accepts everything when the flag is on.
    /// Example: `AisExclusionFilter::new(true)`.
    pub fn new(exclusion_flag_on: bool) -> Self {
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(ExclusionState {
                is_exclusion_flag_on: exclusion_flag_on,
                stored_types: DataMap::default(),
            }),
        }
    }
    /// A filter of one kind (every signature); `AisExclusionFilter::with_type(kind, true)`.
    pub fn with_type(type_to_exclude: AisKindOfInteractive, exclusion_flag_on: bool) -> Self {
        let filter = Self::new(exclusion_flag_on);
        filter
            .state
            .write()
            .stored_types
            .bind(type_to_exclude as i32, Vec::new());
        filter
    }
    /// A filter of one signature of a kind; `AisExclusionFilter::with_signature(kind, 2, true)`.
    pub fn with_signature(
        type_to_exclude: AisKindOfInteractive,
        signature_in_type: i32,
        exclusion_flag_on: bool,
    ) -> Self {
        let filter = Self::new(exclusion_flag_on);
        filter
            .state
            .write()
            .stored_types
            .bind(type_to_exclude as i32, vec![signature_in_type]);
        filter
    }
    /// `Add(kind)`: false when the kind is already stored; `filter.add(kind)`.
    pub fn add(&self, type_to_exclude: AisKindOfInteractive) -> bool {
        if self.is_stored(type_to_exclude) {
            return false;
        }
        self.state
            .write()
            .stored_types
            .bind(type_to_exclude as i32, Vec::new());
        true
    }
    /// `Add(kind, signature)`: appends the signature to the kind; `filter.add_signature(kind, 2)`.
    pub fn add_signature(
        &self,
        type_to_exclude: AisKindOfInteractive,
        signature_in_type: i32,
    ) -> bool {
        let mut state = self.state.write();
        let key = type_to_exclude as i32;
        match state.stored_types.change_find(&key) {
            Ok(list) => list.push(signature_in_type),
            Err(_) => {
                state.stored_types.bind(key, vec![signature_in_type]);
            }
        }
        true
    }
    /// `Remove(kind)`: false when the kind is not stored; `filter.remove(kind)`.
    pub fn remove(&self, type_to_exclude: AisKindOfInteractive) -> bool {
        self.state
            .write()
            .stored_types
            .unbind(&(type_to_exclude as i32))
    }
    /// `Remove(kind, signature)`: the first occurrence of the signature.
    /// Example: `filter.remove_signature(kind, 2)`.
    pub fn remove_signature(
        &self,
        type_to_exclude: AisKindOfInteractive,
        signature_in_type: i32,
    ) -> bool {
        let mut state = self.state.write();
        let Ok(list) = state.stored_types.change_find(&(type_to_exclude as i32)) else {
            return false;
        };
        match list.iter().position(|s| *s == signature_in_type) {
            Some(index) => {
                list.remove(index);
                true
            }
            None => false,
        }
    }
    /// `Clear`; `filter.clear()`.
    pub fn clear(&self) {
        self.state.write().stored_types.clear(true);
    }
    /// `IsExclusionFlagOn`; `filter.is_exclusion_flag_on()`.
    pub fn is_exclusion_flag_on(&self) -> bool {
        self.state.read().is_exclusion_flag_on
    }
    /// `SetExclusionFlag`; `filter.set_exclusion_flag(false)`.
    pub fn set_exclusion_flag(&self, status: bool) {
        self.state.write().is_exclusion_flag_on = status;
    }
    /// `IsStored`; `filter.is_stored(kind)`.
    pub fn is_stored(&self, kind: AisKindOfInteractive) -> bool {
        self.state.read().stored_types.is_bound(&(kind as i32))
    }
    /// `ListOfStoredTypes`: the stored kinds in the native map order; `filter.list_of_stored_types()`.
    pub fn list_of_stored_types(&self) -> Vec<i32> {
        self.state
            .read()
            .stored_types
            .items()
            .map(|(key, _)| *key)
            .collect()
    }
    /// `ListOfSignature`: the signatures stored for a kind; `filter.list_of_signature(kind)`.
    pub fn list_of_signature(&self, kind: AisKindOfInteractive) -> Vec<i32> {
        self.state
            .read()
            .stored_types
            .find(&(kind as i32))
            .cloned()
            .unwrap_or_default()
    }
    /// `IsSignatureIn` (private): whether the signature is stored for the kind.
    /// Example: `filter.is_signature_in(kind, 2)`.
    pub fn is_signature_in(&self, kind: AisKindOfInteractive, signature: i32) -> bool {
        self.state
            .read()
            .stored_types
            .find(&(kind as i32))
            .is_ok_and(|list| list.contains(&signature))
    }
}

impl SelectMgrFilter for AisExclusionFilter {
    /// `IsOk`: the exclusion flag without stored kind; for an interactive object of a
    /// stored kind without signature, or with its signature stored, the opposite of the flag.
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<bool, Failure> {
        let flag = self.is_exclusion_flag_on();
        if self.state.read().stored_types.is_empty() {
            return Ok(flag);
        }
        let Some(object) = owner_object(owner) else {
            return Ok(false);
        };
        let Some(object) = object.get() else {
            return Ok(false);
        };
        let kind = object.kind_of_interactive();
        let Ok(signatures) = self.state.read().stored_types.find(&(kind as i32)).cloned() else {
            return Ok(flag);
        };
        if signatures.is_empty() || signatures.contains(&object.signature()) {
            return Ok(!flag);
        }
        Ok(flag)
    }
}

/// `AIS_BadEdgeFilter`: refuses the edges stored for the current contour.
/// Example: `let filter = AisBadEdgeFilter::default(); filter.set_contour(1);`.
#[derive(Default)]
pub struct AisBadEdgeFilter {
    transient: StandardTransient,
    state: SharedState<BadEdgeState>,
}
rustycad_tkernel::impl_transient!(AisBadEdgeFilter, "AIS_BadEdgeFilter", dyn SelectMgrFilter, transient,
    interfaces: [dyn SelectMgrFilter]);

#[derive(Default)]
struct BadEdgeState {
    bad_edges: DataMap<i32, Vec<Shape>>,
    contour: i32,
}

impl AisBadEdgeFilter {
    /// `SetContour`: the contour whose edges are refused (0 accepts everything).
    /// Example: `filter.set_contour(2)`.
    pub fn set_contour(&self, index: i32) {
        self.state.write().contour = index;
    }
    /// `AddEdge`: an edge refused for a contour; `filter.add_edge(&edge, 2)`.
    pub fn add_edge(&self, edge: &Edge, index: i32) {
        let mut state = self.state.write();
        let shape = Shape::from(edge.clone());
        match state.bad_edges.change_find(&index) {
            Ok(list) => list.push(shape),
            Err(_) => {
                state.bad_edges.bind(index, vec![shape]);
            }
        }
    }
    /// `RemoveEdges`: forgets the edges of a contour; `filter.remove_edges(2)`.
    pub fn remove_edges(&self, index: i32) {
        self.state.write().bad_edges.unbind(&index);
    }
}

impl SelectMgrFilter for AisBadEdgeFilter {
    /// `IsOk`: true without contour or for another owner than a BRep one, false for a
    /// shape stored (IsSame) for the current contour.
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<bool, Failure> {
        let state = self.state.read();
        if state.contour == 0 {
            return Ok(true);
        }
        let Some(shape) = brep_shape(owner) else {
            return Ok(true);
        };
        Ok(state
            .bad_edges
            .find(&state.contour)
            .map(|list| !list.iter().any(|edge| edge.is_same(&shape)))
            .unwrap_or(true))
    }
    /// `ActsOn`: the edge mode only.
    fn acts_on(&self, kind: ShapeEnum) -> bool {
        kind == ShapeEnum::Edge
    }
}

/// `AIS_C0RegularityFilter`: accepts the edges of a shape shared by exactly two faces with a
/// C0 continuity between them. Example: `AisC0RegularityFilter::new(&shape)?`.
pub struct AisC0RegularityFilter {
    transient: StandardTransient,
    map_of_edges: MapOfShape,
}
rustycad_tkernel::impl_transient!(AisC0RegularityFilter, "AIS_C0RegularityFilter", dyn SelectMgrFilter, transient,
    interfaces: [dyn SelectMgrFilter]);

impl AisC0RegularityFilter {
    /// The C0 edges of a shape; `AisC0RegularityFilter::new(&shape)?`.
    pub fn new(shape: &Shape) -> Result<Self, Failure> {
        let mut sub_shapes = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut sub_shapes)?;
        let mut map_of_edges = MapOfShape::default();
        for index in 1..=sub_shapes.extent() {
            let faces: Vec<Shape> = sub_shapes.find_from_index(index)?.iter().cloned().collect();
            if faces.len() != 2 {
                continue;
            }
            let edge = sub_shapes.find_key(index)?.clone();
            let continuity = BRepTool::continuity_on_faces(
                &topods::edge(&edge)?,
                &topods::face(&faces[0])?,
                &topods::face(&faces[1])?,
            )?;
            if continuity == Continuity::C0 {
                map_of_edges.add(edge);
            }
        }
        Ok(Self {
            transient: StandardTransient::default(),
            map_of_edges,
        })
    }
}

impl SelectMgrFilter for AisC0RegularityFilter {
    /// `IsOk`: a BRep owner of a stored edge; a null shape fails with `Standard_NullObject`
    /// (a null dereference natively).
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<bool, Failure> {
        let Some(shape) = brep_shape(owner) else {
            return Ok(false);
        };
        if shape.shape_type()? != ShapeEnum::Edge {
            return Ok(false);
        }
        Ok(self.map_of_edges.contains(&shape))
    }
    /// `ActsOn`: the edge mode only.
    fn acts_on(&self, kind: ShapeEnum) -> bool {
        kind == ShapeEnum::Edge
    }
}

/// `AIS_GraphicTool`: the line and shading attributes of a drawer.
/// Example: `AisGraphicTool::get_line_width(&drawer, AisTypeOfAttribute::Wire)?`.
pub struct AisGraphicTool;

impl AisGraphicTool {
    /// `GetLineAspect` (file static): the line aspect of the attribute; a missing aspect is a
    /// `Standard_NullObject` failure (a null dereference natively).
    fn line_aspect(
        drawer: &Prs3dDrawer,
        attribute: AisTypeOfAttribute,
    ) -> Result<Handle<Graphic3dAspectLine3d>, Failure> {
        use AisTypeOfAttribute as A;
        let aspect = match attribute {
            A::Line => drawer.line_aspect(),
            A::Dimension => drawer.dimension_aspect().line_aspect(),
            A::Wire => drawer.wire_aspect(),
            A::Plane => drawer.plane_aspect().edges_aspect(),
            A::Vector => drawer.vector_aspect(),
            A::UIso => {
                let iso = drawer.u_iso_aspect();
                return iso.get().map(|iso| iso.aspect()).ok_or_else(null_aspect);
            }
            A::VIso => {
                let iso = drawer.v_iso_aspect();
                return iso.get().map(|iso| iso.aspect()).ok_or_else(null_aspect);
            }
            A::Free => drawer.free_boundary_aspect(),
            A::UnFree => drawer.un_free_boundary_aspect(),
            A::Section => drawer.section_aspect(),
            A::Hidden => drawer.hidden_line_aspect(),
            A::Seen => drawer.seen_line_aspect(),
            A::FaceBoundary => drawer.face_boundary_aspect(),
            A::FirstAxis => drawer.datum_aspect().line_aspect(Prs3dDatumParts::XAxis),
            A::SecondAxis => drawer.datum_aspect().line_aspect(Prs3dDatumParts::YAxis),
            A::ThirdAxis => drawer.datum_aspect().line_aspect(Prs3dDatumParts::ZAxis),
        };
        aspect
            .get()
            .map(|aspect| aspect.aspect())
            .ok_or_else(null_aspect)
    }
    /// `GetLineColor`: the name of the line color; `AisGraphicTool::get_line_color(&drawer, attribute)?`.
    pub fn get_line_color(
        drawer: &Prs3dDrawer,
        attribute: AisTypeOfAttribute,
    ) -> Result<NamedColor, Failure> {
        Ok(Self::get_line_color_value(drawer, attribute)?.name())
    }
    /// `GetLineColor(drawer, attribute, color)`: the line color.
    /// Example: `AisGraphicTool::get_line_color_value(&drawer, attribute)?`.
    pub fn get_line_color_value(
        drawer: &Prs3dDrawer,
        attribute: AisTypeOfAttribute,
    ) -> Result<Color, Failure> {
        Ok(Self::line_aspect(drawer, attribute)?.interior_color())
    }
    /// `GetLineWidth`; `AisGraphicTool::get_line_width(&drawer, attribute)?`.
    pub fn get_line_width(
        drawer: &Prs3dDrawer,
        attribute: AisTypeOfAttribute,
    ) -> Result<f64, Failure> {
        Ok(f64::from(Self::line_aspect(drawer, attribute)?.width()))
    }
    /// `GetLineType`; `AisGraphicTool::get_line_type(&drawer, attribute)?`.
    pub fn get_line_type(
        drawer: &Prs3dDrawer,
        attribute: AisTypeOfAttribute,
    ) -> Result<AspectTypeOfLine, Failure> {
        Ok(Self::line_aspect(drawer, attribute)?.line_kind())
    }
    /// `GetLineAtt`: the color name, width and type of the line.
    /// Example: `let (color, width, kind) = AisGraphicTool::get_line_att(&drawer, attribute)?;`.
    pub fn get_line_att(
        drawer: &Prs3dDrawer,
        attribute: AisTypeOfAttribute,
    ) -> Result<(NamedColor, f64, AspectTypeOfLine), Failure> {
        let aspect = Self::line_aspect(drawer, attribute)?;
        Ok((
            aspect.interior_color().name(),
            f64::from(aspect.width()),
            aspect.line_kind(),
        ))
    }
    /// `GetInteriorColor`: the name of the shading interior color.
    /// Example: `AisGraphicTool::get_interior_color(&drawer)?`.
    pub fn get_interior_color(drawer: &Prs3dDrawer) -> Result<NamedColor, Failure> {
        Ok(Self::get_interior_color_value(drawer)?.name())
    }
    /// `GetInteriorColor(drawer, color)`; `AisGraphicTool::get_interior_color_value(&drawer)?`.
    pub fn get_interior_color_value(drawer: &Prs3dDrawer) -> Result<Color, Failure> {
        let shading = drawer.shading_aspect();
        let shading = shading.get().ok_or_else(null_aspect)?;
        Ok(shading.aspect().interior_color())
    }
    /// `GetMaterial`: the back material of the shading aspect.
    /// Example: `AisGraphicTool::get_material(&drawer)?`.
    pub fn get_material(drawer: &Prs3dDrawer) -> Result<Graphic3dMaterialAspect, Failure> {
        let shading = drawer.shading_aspect();
        let shading = shading.get().ok_or_else(null_aspect)?;
        Ok(shading.aspect().back_material())
    }
}

fn null_aspect() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "AIS_GraphicTool: the drawer has no such aspect",
    )
}
