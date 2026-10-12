//! A named group fake records real buffer calls and delegates native bounds/emptiness to the parent.
use super::color_scale_recording::record_scale_vertices;
use rustycad_tkernel::{
    standard::Failure as XrProbeFailure,
    standard::{Handle, SharedState, StandardTransient, raise_native_failure},
};
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::{graphic3d::*, null_driver::NullGroup};
use std::sync::Arc;
pub(super) struct XrRecordingEvents {
    pub callbacks: SharedState<Vec<i32>>,
    pub fail_aspect: SharedState<bool>,
    pub fail_new_group: SharedState<i32>,
}
impl XrRecordingEvents {
    pub fn new() -> Self {
        // -1 disables checked group-allocation faults; ordinary groups still use the native factory.
        Self {
            callbacks: SharedState::default(),
            fail_aspect: SharedState::new(false),
            fail_new_group: SharedState::new(-1),
        }
    }
}
#[derive(Default)]
struct XrRecordedGroupState {
    arrays: Vec<Vec<f64>>,
    clears: i32,
}
pub(super) struct XrRecordingGroup {
    transient: StandardTransient,
    inner: NullGroup,
    events: Arc<XrRecordingEvents>,
    index: i32,
    recorded: SharedState<XrRecordedGroupState>,
}
rustycad_tkernel::impl_transient!(XrRecordingGroup,"Martensite_XrRecordingGroup",StandardTransient,transient,interfaces:[dyn Graphic3dGroup]);
impl XrRecordingGroup {
    pub fn new(
        structure: &Handle<dyn Graphic3dStructureSource>,
        events: Arc<XrRecordingEvents>,
        index: i32,
    ) -> Self {
        // Keep the real group bounds and parent structure link; no canned geometry is returned.
        Self {
            transient: StandardTransient::default(),
            inner: NullGroup::new(structure),
            events,
            index,
            recorded: SharedState::default(),
        }
    }
    pub fn snapshot(&self) -> Vec<f64> {
        let bounds = self.bounding_box();
        let state = self.recorded.read();
        let mut row = vec![
            f64::from(self.is_empty()),
            f64::from(bounds.is_valid()),
            f64::from(state.clears),
            state.arrays.len() as f64,
        ];
        if bounds.is_valid() {
            for axis in 0..3 {
                row.extend([
                    f64::from(bounds.corner_min()[axis]),
                    f64::from(bounds.corner_max()[axis]),
                ]);
            }
        }
        for array in &state.arrays {
            row.extend(array);
        }
        row
    }
}
impl Graphic3dGroup for XrRecordingGroup {
    fn group_base(&self) -> &Graphic3dGroupBase {
        // Delegate to the actual native-style bounds and lifetime state.
        self.inner.group_base()
    }
    fn aspects(&self) -> Handle<dyn Graphic3dAspectsSource> {
        // Aspect identity remains the exact submitted parent handle.
        self.inner.aspects()
    }
    fn set_group_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        self.events.callbacks.write().push(100 + self.index);
        if *self.events.fail_aspect.read() {
            raise_native_failure::<()>(Err(XrProbeFailure::new(
                "XR aspect callback=true; expected successful assignment",
            )));
        }
        self.inner.set_group_primitives_aspect(aspect);
    }
    fn clear(&self, update: bool) {
        self.events.callbacks.write().push(200 + self.index);
        let mut recorded = self.recorded.write();
        recorded.arrays.clear();
        recorded.clears += 1;
        drop(recorded);
        self.inner.clear(update);
    }
    fn set_primitives_aspect(&self, aspect: &Handle<dyn Graphic3dAspectsSource>) {
        // Non-XR driver hooks still delegate to the same retained parent aspects.
        self.inner.set_primitives_aspect(aspect);
    }
    fn synchronize_aspects(&self) {
        // A CPU fake has no GPU resources, like the oracle's null driver.
        self.inner.synchronize_aspects();
    }
    fn replace_aspects(&self, map: &Graphic3dAspectsReplaceMap) {
        // Use the genuine aspect identity map if inherited behavior replaces it.
        self.inner.replace_aspects(map);
    }
    fn set_stencil_test_options(&self, value: bool) {
        // Driver state remains independently stored by the named parent fake.
        self.inner.set_stencil_test_options(value);
    }
    fn set_flipping_options(&self, value: bool, axis: &Axis2) {
        // XR does not invent a clipping/flipping policy.
        self.inner.set_flipping_options(value, axis);
    }
    fn add_primitive_array_buffers(
        &self,
        kind: Graphic3dTypeOfPrimitiveArray,
        indices: &Handle<dyn Graphic3dIndexBufferSource>,
        vertices: &Handle<dyn Graphic3dBufferSource>,
        bounds: &Handle<Graphic3dBoundBuffer>,
        evaluate: bool,
    ) {
        self.events.callbacks.write().push(300 + self.index);
        self.inner
            .add_primitive_array_buffers(kind, indices, vertices, bounds, evaluate);
        self.recorded
            .write()
            .arrays
            .push(xr_recorded_array(kind, indices, vertices, evaluate));
    }
}
fn xr_recorded_array(
    kind: Graphic3dTypeOfPrimitiveArray,
    indices: &Handle<dyn Graphic3dIndexBufferSource>,
    vertices: &Handle<dyn Graphic3dBufferSource>,
    evaluate: bool,
) -> Vec<f64> {
    let colors = vertices
        .attribute_data(Graphic3dTypeOfAttribute::Color)
        .is_some();
    let count = indices.get().map_or(0, |value| value.nb_elements());
    let mut row = vec![
        f64::from(kind as i32),
        f64::from(evaluate),
        f64::from(vertices.nb_elements()),
        f64::from(count),
        f64::from(colors),
    ];
    for vertex in record_scale_vertices(vertices) {
        row.extend(vertex);
    }
    for index in 0..count {
        row.push(f64::from(indices.index(index).unwrap()));
    }
    row
}
