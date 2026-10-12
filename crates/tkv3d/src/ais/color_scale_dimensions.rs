//! Native dimensions and text metrics: defaults without context, live driver values with one.
use super::{AisColorScale, AisInteractiveObjectMembers};
use rustycad_tkernel::{
    standard::{Failure, exceptions::null_object},
    tcollection::{AsciiString, ExtendedString},
};
impl AisColorScale {
    /// Return bar dimensions; `scale.get_size()`.
    pub fn get_size(&self) -> (i32, i32) {
        let f = self.fields.read();
        (f.layout.breadth, f.layout.height)
    }
    /// Retain dimensions without scheduling update; `scale.set_size(100,300)`.
    pub fn set_size(&self, breadth: i32, height: i32) {
        let mut f = self.fields.write();
        f.layout.breadth = breadth;
        f.layout.height = height;
    }
    /// Return bottom-left position; `scale.get_position()`.
    pub fn get_position(&self) -> (f64, f64) {
        let f = self.fields.read();
        (f64::from(f.layout.x), f64::from(f.layout.y))
    }
    /// Retain integer pixel position; `scale.set_position(20,30)`.
    pub fn set_position(&self, x: i32, y: i32) {
        let mut f = self.fields.write();
        f.layout.x = x;
        f.layout.y = y;
    }
    /// Native width default is 10 without a context; `scale.text_width(&text)?`.
    pub fn text_width(&self, text: &ExtendedString) -> Result<i32, Failure> {
        Ok(self.text_size(text, self.get_text_height())?[0])
    }
    /// Native ascent+descent default is 2 without a context; `scale.text_height(&text)?`.
    pub fn text_height(&self, text: &ExtendedString) -> Result<i32, Failure> {
        let [_, a, d] = self.text_size(text, self.get_text_height())?;
        Ok(a + d)
    }
    /// Measure against the first active view's driver, narrowing by integer truncation; `scale.text_size(&text,20)?`.
    pub fn text_size(&self, text: &ExtendedString, height: i32) -> Result<[i32; 3], Failure> {
        if !self.has_interactive_context() {
            return Ok([10, 1, 1]);
        }
        let context = self.get_context();
        let context = context.get().ok_or_else(|| {
            null_object("AIS_ColorScale context=expired; expected a live context")
        })?;
        measure_scale_text(context, text, height)
    }
}

fn measure_scale_text(
    context: &super::AisInteractiveContext,
    text: &ExtendedString,
    height: i32,
) -> Result<[i32; 3], Failure> {
    let viewer = context.current_viewer().observe();
    let cursor = viewer.active_view_iterator();
    if !cursor.more() {
        return Err(null_object(
            "AIS_ColorScale active views=0; expected an active view for text metrics",
        ));
    }
    let view = rustycad_tkernel::standard::HandleReference::observe(&cursor.value_ref());
    let view = view
        .get()
        .ok_or_else(|| null_object("AIS_ColorScale active view=null; expected a live view"))?;
    scale_driver_text_metrics(&viewer, view, text, height)
}

fn scale_driver_text_metrics(
    viewer: &super::super::v3d::V3dViewer,
    view: &dyn super::super::v3d::V3dViewSource,
    text: &ExtendedString,
    height: i32,
) -> Result<[i32; 3], Failure> {
    let ascii = AsciiString::from_extended(text, None);
    let text = String::from_utf8_lossy(ascii.as_bytes());
    Ok(viewer
        .driver()
        .text_size(&view.view_ref().observe(), &text, height as f32)
        .map(|value| value as i32))
}
