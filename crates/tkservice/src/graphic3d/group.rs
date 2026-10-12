//! `Graphic3d_Group`: a group of primitives of a structure (visualization-design.md section
//! 1). The class is abstract, so it is the [`Graphic3dGroup`] trait over the embedded
//! [`Graphic3dGroupBase`], with the non-virtual members in [`Graphic3dGroupMembers`].
use super::Graphic3dVertex;
use super::group_text::{self, Graphic3dGroupTextLayout};
use super::{
    Graphic3dArrayOfPrimitives, Graphic3dAspectsSource, Graphic3dBndBox4f, Graphic3dBoundBuffer,
    Graphic3dBufferMembers as _, Graphic3dBufferSource, Graphic3dCStructureMembers as _,
    Graphic3dFlipper, Graphic3dIndexBufferSource, Graphic3dStructureMembers as _,
    Graphic3dStructureSource, Graphic3dText, Graphic3dTransformPersSource,
    Graphic3dTypeOfAttribute, Graphic3dTypeOfData, Graphic3dTypeOfPrimitiveArray,
};
use rustycad_tkernel::ncollection::Vec4;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient, Transient, WeakHandle};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkmath::gp::{Axis2, Transform3};
use std::collections::HashMap;
use std::io;

/// `NCollection_DataMap<handle<Graphic3d_Aspects>, handle<Graphic3d_Aspects>>`, keyed by the
/// address of the aspects to replace.
pub type Graphic3dAspectsReplaceMap = HashMap<
    usize,
    (
        Handle<dyn Graphic3dAspectsSource>,
        Handle<dyn Graphic3dAspectsSource>,
    ),
>;

/// The protected fields of a group.
#[derive(Clone, Default)]
pub struct Graphic3dGroupFields {
    /// `myTrsfPers`: the current transform persistence.
    pub trsf_pers: Handle<dyn Graphic3dTransformPersSource>,
    /// `myFlipper`: the current flipping.
    pub flipper: Handle<Graphic3dFlipper>,
    /// `myStructure`: the parent structure (a raw pointer natively; null once removed).
    pub structure: WeakHandle<dyn Graphic3dStructureSource>,
    /// `myBounds`.
    pub bounds: Graphic3dBndBox4f,
    /// `myTrsf`: the group transformation.
    pub trsf: Transform3,
    /// `myIsClosed`: whether the group is a closed volume.
    pub is_closed: bool,
}

/// The embedded state of a group.
#[derive(Default)]
pub struct Graphic3dGroupBase {
    /// The fields, shared by every handle.
    pub fields: SharedState<Graphic3dGroupFields>,
}

impl Graphic3dGroupBase {
    /// The protected constructor: a group of `structure`, open, with void bounds;
    /// `Graphic3dGroupBase::new(&structure)`.
    pub fn new(structure: &Handle<dyn Graphic3dStructureSource>) -> Self {
        let fields = Graphic3dGroupFields {
            structure: structure.downgrade(),
            ..Default::default()
        };
        Self {
            fields: SharedState::new(fields),
        }
    }
}

/// A group of primitives; the aspect and option members are implemented by the driver.
pub trait Graphic3dGroup: Transient {
    /// The embedded base; `group.group_base()`.
    fn group_base(&self) -> &Graphic3dGroupBase;
    /// `Clear`: empties the bounds and, when asked, updates the structure manager;
    /// `group.clear(true)`.
    fn clear(&self, update_structure_mgr: bool) {
        if self.is_deleted() {
            return;
        }
        self.group_base().fields.write().bounds.clear();
        if update_structure_mgr {
            self.update();
        }
    }
    /// `Aspects`: the group aspects.
    fn aspects(&self) -> Handle<dyn Graphic3dAspectsSource>;
    /// `SetGroupPrimitivesAspect`: the aspects of the whole group.
    fn set_group_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>);
    /// `SetPrimitivesAspect`: the aspects of the next primitives.
    fn set_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>);
    /// `SynchronizeAspects`: updates the presentation after an aspect change.
    fn synchronize_aspects(&self);
    /// `ReplaceAspects`: replaces aspects by the map.
    fn replace_aspects(&self, map: &Graphic3dAspectsReplaceMap);
    /// `AddText`: adds the text position to the bounds (when asked) and marks a text without
    /// plane as 2D text; `group.add_text(&text, true)`.
    fn add_text(&self, text: &Handle<Graphic3dText>, to_eval_min_max: bool) {
        self.record_text_bounds(text, to_eval_min_max);
    }
    /// Apply the native base AddText bookkeeping from a driver override; `group.record_text_bounds(&text, true)`.
    fn record_text_bounds(&self, text: &Handle<Graphic3dText>, to_eval_min_max: bool) {
        group_add_text(self, text, to_eval_min_max);
    }
    /// `AddPrimitiveArray(type, indices, attribs, bounds, toEvalMinMax)`: adds the vertex
    /// positions to the bounds (when asked); `group.add_primitive_array_buffers(t, &i, &a, &b, true)`.
    fn add_primitive_array_buffers(
        &self,
        primitive_type: Graphic3dTypeOfPrimitiveArray,
        indices: &Handle<dyn Graphic3dIndexBufferSource>,
        attribs: &Handle<dyn Graphic3dBufferSource>,
        bounds: &Handle<Graphic3dBoundBuffer>,
        to_eval_min_max: bool,
    ) {
        let _ = (primitive_type, indices, bounds);
        group_add_vertices(self, attribs, to_eval_min_max);
    }
    /// The deprecated virtual `Text(const char*, Graphic3d_Vertex, height, angle, path, hta,
    /// vta, evalMinMax)`; `group.text_at_vertex("A", &point, &layout, true)`.
    fn text_at_vertex(
        &self,
        text: &str,
        point: &Graphic3dVertex,
        layout: &Graphic3dGroupTextLayout,
        to_eval_min_max: bool,
    ) {
        group_text::text_at_vertex(self, text, point, layout, to_eval_min_max);
    }
    /// The deprecated virtual `Text(const char*, gp_Ax2, height, angle, path, hta, vta,
    /// evalMinMax, hasOwnAnchor)`, `flags` = [evalMinMax, hasOwnAnchor];
    /// `group.text_in_plane("A", &axes, &layout, [true, true])`.
    fn text_in_plane(
        &self,
        text: &str,
        orientation: &Axis2,
        layout: &Graphic3dGroupTextLayout,
        flags: [bool; 2],
    ) {
        group_text::text_in_plane(self, text, orientation, layout, flags);
    }
    /// The deprecated virtual `Text(TCollection_ExtendedString, gp_Ax2, ...)`;
    /// `group.ext_text_in_plane(&text, &axes, &layout, [true, true])`.
    fn ext_text_in_plane(
        &self,
        text: &ExtendedString,
        orientation: &Axis2,
        layout: &Graphic3dGroupTextLayout,
        flags: [bool; 2],
    ) {
        group_text::ext_text_in_plane(self, text, orientation, layout, flags);
    }
    /// `DumpJson`: the address, persistence, structure address, bounds and closed flag;
    /// `group.dump_json(&mut stream, -1)?`.
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        super::driver_dump::dump_group(self, stream, depth)
    }
    /// `SetStencilTestOptions`.
    fn set_stencil_test_options(&self, is_enabled: bool);
    /// `SetFlippingOptions`.
    fn set_flipping_options(&self, is_enabled: bool, ref_plane: &Axis2);
    /// `SetTransformation`; `group.set_transformation(&trsf)`.
    fn set_transformation(&self, trsf: &Transform3) {
        self.group_base().fields.write().trsf = *trsf;
    }
    /// `SetTransformPersistence`: stores a changed persistence and, when not null, marks the
    /// parent structure as having group persistence; `group.set_transform_persistence(&pers)`.
    fn set_transform_persistence(&self, trsf_pers: &Handle<dyn Graphic3dTransformPersSource>) {
        let changed = self.group_base().fields.read().trsf_pers != *trsf_pers;
        if !changed {
            return;
        }
        self.group_base().fields.write().trsf_pers = trsf_pers.clone();
        if !self.is_deleted() && !trsf_pers.is_null() {
            mark_group_persistence(self);
        }
    }
}
rustycad_tkernel::impl_transient_interface!(
    dyn Graphic3dGroup,
    "Graphic3d_Group",
    StandardTransient
);

/// Sets the group transform persistence flag of the parent driver structure.
fn mark_group_persistence<G: Graphic3dGroup + ?Sized>(group: &G) {
    let structure = group.structure();
    if let Some(cstructure) = structure.get().map(|s| s.cstructure())
        && let Some(cstructure) = cstructure.get()
    {
        cstructure.set_group_transform_persistence(true);
    }
}

/// The native `AddText` body.
fn group_add_text<G: Graphic3dGroup + ?Sized>(
    group: &G,
    text: &Handle<Graphic3dText>,
    to_eval_min_max: bool,
) {
    if group.is_deleted() {
        return;
    }
    if let (true, Some(text)) = (to_eval_min_max, text.get()) {
        let structure = group.structure();
        if let Some(cstructure) = structure.get().map(|s| s.cstructure())
            && let Some(cstructure) = cstructure.get()
        {
            cstructure.cstructure_base().fields.write().is_2d_text = !text.has_plane();
        }
        let position = text.position();
        let point = Vec4::new(
            position.x() as f32,
            position.y() as f32,
            position.z() as f32,
            1.0f32,
        );
        group.group_base().fields.write().bounds.add(&point);
    }
    group.update();
}

/// The native `AddPrimitiveArray(type, ...)` body: the positions (2D at z = 0, or 3D) of the
/// first position attribute are added to the bounds.
fn group_add_vertices<G: Graphic3dGroup + ?Sized>(
    group: &G,
    attribs: &Handle<dyn Graphic3dBufferSource>,
    to_eval_min_max: bool,
) {
    let Some(buffer) = attribs.get().filter(|_| !group.is_deleted()) else {
        return;
    };
    if to_eval_min_max && let Some(points) = position_points(buffer) {
        let mut fields = group.group_base().fields.write();
        points.iter().for_each(|point| fields.bounds.add(point));
    }
    group.update();
}

/// The homogeneous positions of the vertices, or `None` without a position attribute.
fn position_points(buffer: &dyn Graphic3dBufferSource) -> Option<Vec<Vec4<f32>>> {
    let (index, offset, stride) = buffer.attribute_data(Graphic3dTypeOfAttribute::Pos)?;
    let components = match buffer.attribute(index)?.data_type {
        Graphic3dTypeOfData::Vec2 => 2,
        Graphic3dTypeOfData::Vec3 | Graphic3dTypeOfData::Vec4 => 3,
        _ => return Some(Vec::new()),
    };
    let read = |at: usize| buffer.read_at::<f32>(at).unwrap_or(0.0);
    let points = (0..buffer.nb_elements().max(0) as usize).map(|vertex| {
        let at = offset + stride * vertex;
        let z = if components == 3 { read(at + 8) } else { 0.0 };
        Vec4::new(read(at), read(at + 4), z, 1.0)
    });
    Some(points.collect())
}

/// The non-virtual members of every group.
pub trait Graphic3dGroupMembers: Graphic3dGroup {
    /// `Remove`: detaches the group from its structure, updates the manager and empties the
    /// bounds; `group.remove()`.
    fn remove(&self) {
        if self.is_deleted() {
            return;
        }
        let structure = self.structure();
        if let Some(structure) = structure.get() {
            structure.remove_group_at(std::ptr::from_ref(self).cast::<()>());
        }
        self.update();
        self.group_base().fields.write().bounds.clear();
    }
    /// `Flipper`; `group.flipper()`.
    fn flipper(&self) -> Handle<Graphic3dFlipper> {
        self.group_base().fields.read().flipper.clone()
    }
    /// `Transformation`; `group.transformation()`.
    fn transformation(&self) -> Transform3 {
        self.group_base().fields.read().trsf
    }
    /// `TransformPersistence`; `group.transform_persistence()`.
    fn transform_persistence(&self) -> Handle<dyn Graphic3dTransformPersSource> {
        self.group_base().fields.read().trsf_pers.clone()
    }
    /// `IsDeleted`: no parent structure, or a deleted one; `group.is_deleted()`.
    fn is_deleted(&self) -> bool {
        let structure = self.structure();
        structure.get().is_none_or(|s| s.is_deleted())
    }
    /// `IsEmpty`: deleted, or a finite structure with void bounds; `group.is_empty()`.
    fn is_empty(&self) -> bool {
        let structure = self.structure();
        let Some(structure) = structure.get().filter(|s| !s.is_deleted()) else {
            return true;
        };
        !structure.is_infinite() && !self.group_base().fields.read().bounds.is_valid()
    }
    /// `MinMaxValues`: `[xmin, ymin, zmin, xmax, ymax, zmax]`; the whole short-real range
    /// when empty, inverted when the bounds are void; `group.min_max_values()`.
    fn min_max_values(&self) -> [f64; 6] {
        let (first, last) = (f64::from(f32::MIN), f64::from(f32::MAX));
        if self.is_empty() {
            return [first, first, first, last, last, last];
        }
        let bounds = self.group_base().fields.read().bounds;
        if !bounds.is_valid() {
            return [last, last, last, first, first, first];
        }
        let (min, max) = (bounds.corner_min(), bounds.corner_max());
        [min.x, min.y, min.z, max.x, max.y, max.z].map(f64::from)
    }
    /// `SetMinMaxValues`: the bounds as single-precision corners;
    /// `group.set_min_max_values([0.0, 0.0, 0.0, 1.0, 1.0, 1.0])`.
    fn set_min_max_values(&self, values: [f64; 6]) {
        let [x0, y0, z0, x1, y1, z1] = values.map(|v| v as f32);
        let bounds =
            Graphic3dBndBox4f::from_corners(Vec4::new(x0, y0, z0, 1.0), Vec4::new(x1, y1, z1, 1.0));
        self.group_base().fields.write().bounds = bounds;
    }
    /// `BoundingBox`; `group.bounding_box()`.
    fn bounding_box(&self) -> Graphic3dBndBox4f {
        self.group_base().fields.read().bounds
    }
    /// `ChangeBoundingBox` as an assignment; `group.set_bounding_box(bounds)`.
    fn set_bounding_box(&self, bounds: Graphic3dBndBox4f) {
        self.group_base().fields.write().bounds = bounds;
    }
    /// `Structure`: the parent structure (null once removed); `group.structure()`.
    fn structure(&self) -> Handle<dyn Graphic3dStructureSource> {
        self.group_base().fields.read().structure.upgrade()
    }
    /// `SetClosed`; `group.set_closed(true)`.
    fn set_closed(&self, is_closed: bool) {
        self.group_base().fields.write().is_closed = is_closed;
    }
    /// `IsClosed`; `group.is_closed()`.
    fn is_closed(&self) -> bool {
        self.group_base().fields.read().is_closed
    }
    /// `AddPrimitiveArray(handle<Graphic3d_ArrayOfPrimitives>)`: adds a valid array's buffers;
    /// `group.add_primitive_array(&array, true)`.
    fn add_primitive_array(
        &self,
        array: &Handle<Graphic3dArrayOfPrimitives>,
        to_eval_min_max: bool,
    ) {
        let Some(array) = array.get().filter(|a| !self.is_deleted() && a.is_valid()) else {
            return;
        };
        // OCCT 3d097a0 forwards real const-buffer fields, preserving driver-visible ownership counts.
        self.add_primitive_array_buffers(
            array.primitive_type(),
            array.indices_ref(),
            array.attributes_ref(),
            array.bounds_ref(),
            to_eval_min_max,
        );
    }
    /// `Update`: updates the structure manager of the parent structure; `group.update()`.
    fn update(&self) {
        if self.is_deleted() {
            return;
        }
        let structure = self.structure();
        if let Some(manager) = structure.get().map(|s| s.structure_manager())
            && let Some(manager) = manager.get()
        {
            manager.update(super::GRAPHIC3D_ZLAYER_ID_UNKNOWN);
        }
    }
}
impl<T: Graphic3dGroup + ?Sized> Graphic3dGroupMembers for T {}
