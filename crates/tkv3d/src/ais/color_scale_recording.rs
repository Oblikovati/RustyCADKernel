//! Named CPU recorder observes real Graphic3d calls, without replacing scale geometry or layout.
use rustycad_tkernel::{
    ncollection::{Vec3, Vec4},
    standard::{Failure, Handle, SharedState, StandardTransient},
};
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::{graphic3d::*, null_driver::*};
use std::sync::Arc;
#[derive(Clone, Debug, PartialEq)]
pub(super) enum ScaleDrawRequest {
    Synchronize,
    Array {
        kind: i32,
        vertices: Vec<Vec<f64>>,
        indices: Vec<i32>,
    },
    Text3d {
        frame: [f64; 9],
        height: f32,
        horizontal: i32,
        vertical: i32,
        anchor: bool,
        plane: bool,
        text: String,
    },
    Text {
        point: [f64; 2],
        height: f32,
        alignment: i32,
        anchor: bool,
        text: String,
    },
}
pub(super) type ScaleDrawTrace = Arc<SharedState<Vec<ScaleDrawRequest>>>;
pub(super) fn recording_presentation(
    trace: &ScaleDrawTrace,
) -> Handle<dyn Graphic3dStructureSource> {
    recording_presentation_mode(trace, false)
}
pub(super) fn recording_oriented_presentation(
    trace: &ScaleDrawTrace,
) -> Handle<dyn Graphic3dStructureSource> {
    recording_presentation_mode(trace, true)
}
fn recording_presentation_mode(
    trace: &ScaleDrawTrace,
    oriented: bool,
) -> Handle<dyn Graphic3dStructureSource> {
    let manager =
        Handle::new(Graphic3dStructureManager::new(&NullGraphicDriver::new_handle()).unwrap());
    let prs = Graphic3dStructure::new(&manager, &Handle::default()).unwrap();
    install_scale_recording(&prs.cast(), trace, oriented);
    prs.cast()
}
/// Retain cached identity while observing actual groups; `install_scale_recording(&prs,&trace,true)`.
pub(super) fn install_scale_recording(
    prs: &Handle<dyn Graphic3dStructureSource>,
    trace: &ScaleDrawTrace,
    oriented: bool,
) {
    let recorder = Handle::new(ScaleRecordingStructure {
        transient: StandardTransient::default(),
        inner: prs.cstructure(),
        trace: trace.clone(),
        oriented,
    });
    wrap_existing_recording_groups(&recorder);
    prs.structure_base().fields.write().cstructure = recorder.cast();
}

struct ScaleRecordingStructure {
    transient: StandardTransient,
    inner: Handle<dyn Graphic3dCStructure>,
    trace: ScaleDrawTrace,
    oriented: bool,
}
rustycad_tkernel::impl_transient!(ScaleRecordingStructure,"ScaleRecordingStructure",StandardTransient,transient,interfaces:[dyn Graphic3dCStructure]);
impl Graphic3dCStructure for ScaleRecordingStructure {
    fn cstructure_base(&self) -> &Graphic3dCStructureBase {
        self.inner.cstructure_base()
    }
    fn on_visibility_changed(&self) {
        self.inner.on_visibility_changed();
    }
    fn clear(&self) {
        self.inner.clear();
    }
    fn connect(&self, other: &dyn Graphic3dCStructure) {
        self.inner.connect(other);
    }
    fn disconnect(&self, other: &dyn Graphic3dCStructure) {
        self.inner.disconnect(other);
    }
    fn graphic_highlight(&self, style: &Handle<dyn Graphic3dPresentationAttributesSource>) {
        self.inner.graphic_highlight(style);
    }
    fn graphic_unhighlight(&self) {
        self.inner.graphic_unhighlight();
    }
    fn shadow_link(
        &self,
        manager: &Handle<Graphic3dStructureManager>,
    ) -> Result<Handle<dyn Graphic3dCStructure>, Failure> {
        self.inner.shadow_link(manager)
    }
    fn remove_group(&self, group: &Handle<dyn Graphic3dGroup>) {
        self.inner.remove_group(group);
    }
    fn new_group(
        &self,
        structure: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<Handle<dyn Graphic3dGroup>, Failure> {
        let group: Handle<dyn Graphic3dGroup> = Handle::new(ScaleRecordingGroup {
            transient: StandardTransient::default(),
            inner: Handle::new(NullGroup::new(structure)).cast(),
            trace: self.trace.clone(),
            oriented: self.oriented,
        })
        .cast();
        self.cstructure_base()
            .fields
            .write()
            .groups
            .push(group.clone());
        Ok(group)
    }
}
struct ScaleRecordingGroup {
    transient: StandardTransient,
    inner: Handle<dyn Graphic3dGroup>,
    trace: ScaleDrawTrace,
    oriented: bool,
}
rustycad_tkernel::impl_transient!(ScaleRecordingGroup,"ScaleRecordingGroup",StandardTransient,transient,interfaces:[dyn Graphic3dGroup]);
impl Graphic3dGroup for ScaleRecordingGroup {
    fn group_base(&self) -> &Graphic3dGroupBase {
        self.inner.group_base()
    }
    fn aspects(&self) -> Handle<dyn Graphic3dAspectsSource> {
        self.inner.aspects()
    }
    fn set_group_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.inner.set_group_primitives_aspect(aspect);
    }
    fn set_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.inner.set_primitives_aspect(aspect);
    }
    fn synchronize_aspects(&self) {
        self.trace.write().push(ScaleDrawRequest::Synchronize);
        self.inner.synchronize_aspects();
    }
    fn replace_aspects(&self, map: &Graphic3dAspectsReplaceMap) {
        self.inner.replace_aspects(map);
    }
    fn set_stencil_test_options(&self, on: bool) {
        self.inner.set_stencil_test_options(on);
    }
    fn set_flipping_options(&self, on: bool, axis: &Axis2) {
        self.inner.set_flipping_options(on, axis);
    }
    fn add_text(&self, text: &Handle<Graphic3dText>, _: bool) {
        if self.oriented {
            self.trace.write().push(record_cube_text(text));
            return;
        }
        let point = text.orientation().location();
        self.trace.write().push(ScaleDrawRequest::Text {
            point: [point.x(), point.y()],
            height: text.height(),
            alignment: text.vertical_alignment() as i32,
            anchor: text.has_own_anchor_point(),
            text: String::from_utf8_lossy(text.text().to_c_string())
                .trim_end_matches('\0')
                .to_owned(),
        });
    }
    fn add_primitive_array_buffers(
        &self,
        kind: Graphic3dTypeOfPrimitiveArray,
        indices: &Handle<dyn Graphic3dIndexBufferSource>,
        vertices: &Handle<dyn Graphic3dBufferSource>,
        _: &Handle<Graphic3dBoundBuffer>,
        _: bool,
    ) {
        let indexes = record_scale_indices(indices);
        self.trace.write().push(ScaleDrawRequest::Array {
            kind: kind as i32,
            vertices: record_scale_vertices(vertices),
            indices: indexes,
        });
    }
}
pub(super) fn record_scale_vertices(vertices: &Handle<dyn Graphic3dBufferSource>) -> Vec<Vec<f64>> {
    (0..vertices.nb_elements())
        .map(|index| record_scale_vertex(vertices, index))
        .collect()
}
fn record_scale_vertex(vertices: &Handle<dyn Graphic3dBufferSource>, index: i32) -> Vec<f64> {
    let (_, offset, stride) = vertices
        .attribute_data(Graphic3dTypeOfAttribute::Pos)
        .unwrap();
    let point: Vec3<f32> = vertices.read_at(offset + index as usize * stride).unwrap();
    let mut values = [point.x, point.y, point.z].map(f64::from).to_vec();
    record_scale_color(vertices, index, &mut values);
    if let Some((_, offset, stride)) = vertices.attribute_data(Graphic3dTypeOfAttribute::Norm) {
        let normal: Vec3<f32> = vertices.read_at(offset + index as usize * stride).unwrap();
        values.extend([normal.x, normal.y, normal.z].map(f64::from));
    }
    values
}
fn record_scale_color(
    vertices: &Handle<dyn Graphic3dBufferSource>,
    index: i32,
    values: &mut Vec<f64>,
) {
    if let Some((_, offset, stride)) = vertices.attribute_data(Graphic3dTypeOfAttribute::Color) {
        let color: Vec4<u8> = vertices.read_at(offset + index as usize * stride).unwrap();
        values.extend([color.x, color.y, color.z, color.w].map(f64::from));
    }
}

fn record_cube_text(text: &Graphic3dText) -> ScaleDrawRequest {
    ScaleDrawRequest::Text3d {
        frame: record_cube_text_frame(text),
        height: text.height(),
        horizontal: text.horizontal_alignment() as i32,
        vertical: text.vertical_alignment() as i32,
        anchor: text.has_own_anchor_point(),
        plane: text.has_plane(),
        text: String::from_utf8_lossy(text.text().to_c_string())
            .trim_end_matches('\0')
            .to_owned(),
    }
}
fn record_cube_text_frame(text: &Graphic3dText) -> [f64; 9] {
    let frame = text.orientation();
    let location = frame.location();
    let normal = frame.direction();
    let x = frame.x_direction();
    [
        location.x(),
        location.y(),
        location.z(),
        normal.x(),
        normal.y(),
        normal.z(),
        x.x(),
        x.y(),
        x.z(),
    ]
}

fn wrap_existing_recording_groups(recorder: &ScaleRecordingStructure) {
    let mut fields = recorder.cstructure_base().fields.write();
    fields.groups = fields
        .groups
        .iter()
        .map(|group| {
            Handle::new(ScaleRecordingGroup {
                transient: StandardTransient::default(),
                inner: group.clone(),
                trace: recorder.trace.clone(),
                oriented: recorder.oriented,
            })
            .cast()
        })
        .collect();
}

fn record_scale_indices(indices: &Handle<dyn Graphic3dIndexBufferSource>) -> Vec<i32> {
    if indices.is_null() {
        Vec::new()
    } else {
        (0..indices.nb_elements())
            .map(|i| indices.index(i).unwrap())
            .collect()
    }
}
