//! Record actual primitive buffers, aspects and flipping calls without drawing.
use super::DisplayTrace;
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::{graphic3d::*, null_driver::*};
pub(super) struct SensitiveDisplayGroup {
    transient: StandardTransient,
    inner: NullGroup,
    trace: DisplayTrace,
}
rustycad_tkernel::impl_transient!(SensitiveDisplayGroup, "SensitiveDisplayGroup", StandardTransient, transient, interfaces: [dyn Graphic3dGroup]);
impl SensitiveDisplayGroup {
    pub fn new(structure: &Handle<dyn Graphic3dStructureSource>, trace: DisplayTrace) -> Self {
        Self {
            transient: StandardTransient::default(),
            inner: NullGroup::new(structure),
            trace,
        }
    }
    fn record_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        let color = aspect.interior_color();
        self.trace.write().events.extend([
            3.,
            color.red(),
            color.green(),
            color.blue(),
            f64::from(aspect.line_type() as i32),
            f64::from(aspect.line_width()),
            f64::from(aspect.marker_type() as i32),
            f64::from(aspect.marker_scale()),
        ]);
    }
    fn apply_append_callbacks(&self) {
        use rustycad_tkv3d::select_mgr::*;
        let (selection, object) = {
            let mut trace = self.trace.write();
            (
                std::mem::take(&mut trace.append_selection),
                std::mem::take(&mut trace.append_object),
            )
        };
        if !selection.is_null() {
            self.trace.write().events.push(6.);
            append_display_point(&selection, rustycad_tkmath::gp::Point3::new(9., 8., 7.));
        }
        if !object.is_null() {
            self.trace.write().events.push(7.);
            let added = Handle::new(SelectMgrSelection::new(1));
            append_display_point(&added, rustycad_tkmath::gp::Point3::new(6., 5., 4.));
            added.set_selection_state(SelectMgrStateOfSelection::Activated);
            object.add_selection(&added, 1).unwrap();
        }
    }
}
impl Graphic3dGroup for SensitiveDisplayGroup {
    fn group_base(&self) -> &Graphic3dGroupBase {
        self.inner.group_base()
    }
    fn add_text(&self, text: &Handle<Graphic3dText>, evaluate: bool) {
        self.trace.write().texts.push(text.clone());
        self.inner.add_text(text, evaluate);
    }
    fn aspects(&self) -> Handle<dyn Graphic3dAspectsSource> {
        self.inner.aspects()
    }
    fn set_group_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.record_aspect(aspect);
        self.inner.set_group_primitives_aspect(aspect)
    }
    fn set_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.record_aspect(aspect);
        self.inner.set_primitives_aspect(aspect)
    }
    fn synchronize_aspects(&self) {
        self.inner.synchronize_aspects()
    }
    fn replace_aspects(&self, map: &Graphic3dAspectsReplaceMap) {
        self.inner.replace_aspects(map)
    }
    fn set_stencil_test_options(&self, is_enabled: bool) {
        self.inner.set_stencil_test_options(is_enabled)
    }
    fn set_flipping_options(&self, is_enabled: bool, ref_plane: &Axis2) {
        let point = ref_plane.location();
        self.trace.write().events.extend([
            5.,
            f64::from(is_enabled),
            point.x(),
            point.y(),
            point.z(),
        ]);
        self.apply_append_callbacks();
    }
    fn add_primitive_array_buffers(
        &self,
        kind: Graphic3dTypeOfPrimitiveArray,
        indices: &Handle<dyn Graphic3dIndexBufferSource>,
        attribs: &Handle<dyn Graphic3dBufferSource>,
        bounds: &Handle<Graphic3dBoundBuffer>,
        evaluate: bool,
    ) {
        let mut values = vec![4., f64::from(kind as i32), f64::from(attribs.nb_elements())];
        let (_, start, stride) = attribs
            .attribute_data(Graphic3dTypeOfAttribute::Pos)
            .unwrap();
        for index in 0..attribs.nb_elements() {
            let vertex: rustycad_tkernel::ncollection::Vec3<f32> =
                attribs.read_at(start + index as usize * stride).unwrap();
            values.extend([vertex.x, vertex.y, vertex.z].map(f64::from));
        }
        append_display_indices(&mut values, indices);
        self.trace.write().events.extend(values);
        self.inner
            .add_primitive_array_buffers(kind, indices, attribs, bounds, evaluate);
    }
}

fn append_display_point(
    selection: &Handle<rustycad_tkv3d::select_mgr::SelectMgrSelection>,
    point: rustycad_tkmath::gp::Point3,
) {
    use rustycad_tkv3d::{select_mgr::SelectMgrEntityOwner, select3d::Select3DSensitivePoint};
    let entity = Select3DSensitivePoint::new(&SelectMgrEntityOwner::new(0).cast(), point);
    selection.add(&Handle::new(entity).cast()).unwrap();
}
fn append_display_indices(values: &mut Vec<f64>, indices: &Handle<dyn Graphic3dIndexBufferSource>) {
    values.push(f64::from(if indices.is_null() {
        0
    } else {
        indices.nb_elements()
    }));
    if indices.is_null() {
        return;
    }
    for index in 0..indices.nb_elements() {
        values.push(f64::from(indices.index(index).unwrap()));
    }
}
