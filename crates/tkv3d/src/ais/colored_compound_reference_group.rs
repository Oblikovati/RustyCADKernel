//! Named group recorder delegates real geometry bounds while observing actual primitive buffers.
use super::{
    color_scale_recording::record_scale_vertices,
    colored_compound_reference_graphics::ColoredCompoundGraphicsTrace,
};
use rustycad_tkernel::standard::{Handle, StandardTransient};
use rustycad_tkmath::gp::Axis2;
use rustycad_tkservice::{graphic3d::*, null_driver::NullGroup};
use std::{fmt::Write, sync::Arc};
pub(super) struct ColoredCompoundReferenceGroupFake {
    transient: StandardTransient,
    inner: NullGroup,
    trace: Arc<ColoredCompoundGraphicsTrace>,
}
rustycad_tkernel::impl_transient!(ColoredCompoundReferenceGroupFake,"Martensite_ColoredCompoundReferenceGroup",StandardTransient,transient,interfaces:[dyn Graphic3dGroup]);
impl ColoredCompoundReferenceGroupFake {
    pub(super) fn new(
        prs: &Handle<dyn Graphic3dStructureSource>,
        trace: &Arc<ColoredCompoundGraphicsTrace>,
    ) -> Self {
        Self {
            transient: Default::default(),
            inner: NullGroup::new(prs),
            trace: Arc::clone(trace),
        }
    }
}
impl Graphic3dGroup for ColoredCompoundReferenceGroupFake {
    fn group_base(&self) -> &Graphic3dGroupBase {
        let group = &self.inner;
        group.group_base()
    }
    fn aspects(&self) -> Handle<dyn Graphic3dAspectsSource> {
        let group = &self.inner;
        group.aspects()
    }
    fn set_group_primitives_aspect(
        &self,
        aspect: &dyn rustycad_tkernel::standard::HandleReference<dyn Graphic3dAspectsSource>,
    ) {
        let group = &self.inner;
        group.set_group_primitives_aspect(aspect);
    }
    fn set_primitives_aspect(
        &self,
        aspect: &dyn rustycad_tkernel::standard::HandleReference<dyn Graphic3dAspectsSource>,
    ) {
        let group = &self.inner;
        group.set_primitives_aspect(aspect);
    }
    fn synchronize_aspects(&self) {
        let group = &self.inner;
        group.synchronize_aspects();
    }
    fn replace_aspects(&self, aspects: &Graphic3dAspectsReplaceMap) {
        let group = &self.inner;
        group.replace_aspects(aspects);
    }
    fn set_stencil_test_options(&self, enabled: bool) {
        let group = &self.inner;
        group.set_stencil_test_options(enabled);
    }
    fn set_flipping_options(&self, enabled: bool, plane: &Axis2) {
        let group = &self.inner;
        group.set_flipping_options(enabled, plane);
    }
    fn add_primitive_array_buffers(
        &self,
        kind: Graphic3dTypeOfPrimitiveArray,
        indices: &Handle<dyn Graphic3dIndexBufferSource>,
        vertices: &Handle<dyn Graphic3dBufferSource>,
        bounds: &Handle<Graphic3dBoundBuffer>,
        evaluate: bool,
    ) {
        record_compound_reference_array(&self.trace, kind, vertices);
        self.inner
            .add_primitive_array_buffers(kind, indices, vertices, bounds, evaluate);
    }
}
fn record_compound_reference_array(
    trace: &ColoredCompoundGraphicsTrace,
    kind: Graphic3dTypeOfPrimitiveArray,
    vertices: &Handle<dyn Graphic3dBufferSource>,
) {
    let mut rows = trace.rows.lock().unwrap();
    write!(rows, "wf-array {} {}", kind as i32, vertices.nb_elements()).unwrap();
    for point in record_scale_vertices(vertices) {
        for coordinate in point {
            write!(rows, " {coordinate}").unwrap();
        }
    }
    rows.push('\n');
}
