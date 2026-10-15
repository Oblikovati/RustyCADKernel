// Rust translation of Draw/TKDraw/DrawTrSurf/DrawTrSurf_Polygon3D.hxx/.cxx and
// DrawTrSurf_Polygon2D.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `DrawTrSurf_Polygon3D` and `DrawTrSurf_Polygon2D`: drawable polylines.
//!
//! `Dump` and `Save` hand the TKMath Poly writers the number text of the DRAW stream
//! (precision 15 for `dump`, scientific 15 digits for `Save`), as the native writers
//! inherit the `std::ostream` state.
use super::curve_common::PlotPoint;
use super::geometry::geometry;
use crate::draw::{
    DrawColor, DrawColorKind, DrawDisplay, DrawIStream, DrawInterpretor, DrawMarkerShape,
    DrawOStream, DrawSession, Drawable3D, Drawable3DBase, DrawableHandle,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::poly::{self, Polygon2D, Polygon3D};
use std::cell::Cell;
use std::rc::Rc;

/// The default marker size of `Draw_Display::DrawMarker`, in pixels.
const NODE_MARKER_SIZE: i32 = 5;

/// The polyline through `nodes` in yellow, segment by segment, with an `X` at each
/// node when `show_nodes`.
fn draw_nodes<P: PlotPoint>(display: &mut DrawDisplay<'_>, nodes: &[P], show_nodes: bool) {
    display.set_color(DrawColor::new(DrawColorKind::Jaune));
    for pair in nodes.windows(2) {
        pair[0].move_on(display);
        pair[1].draw_on(display);
    }
    if show_nodes {
        for node in nodes {
            node.mark_on(display, DrawMarkerShape::X, NODE_MARKER_SIZE);
        }
    }
}

/// `DrawTrSurf_Polygon3D`: a `Poly_Polygon3D` drawn as a yellow polyline.
///
/// ```
/// use rustycad_tkdraw::draw_tr_surf::DrawTrSurfPolygon3D;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::poly::Polygon3D;
/// let polygon = DrawTrSurfPolygon3D::new(Handle::default());
/// assert!(!polygon.show_nodes());
/// ```
pub struct DrawTrSurfPolygon3D {
    base: Drawable3DBase,
    polygon: Handle<Polygon3D>,
    show_nodes: Cell<bool>,
}

impl DrawTrSurfPolygon3D {
    /// `DrawTrSurf_Polygon3D(P)`: nodes not marked.
    pub fn new(polygon: Handle<Polygon3D>) -> Self {
        Self {
            base: Drawable3DBase::new(),
            polygon,
            show_nodes: Cell::new(false),
        }
    }

    /// `Polygon3D`.
    pub fn polygon(&self) -> Handle<Polygon3D> {
        self.polygon.clone()
    }

    /// `ShowNodes()`.
    pub fn show_nodes(&self) -> bool {
        self.show_nodes.get()
    }

    /// `ShowNodes(bool)`.
    pub fn set_show_nodes(&self, show: bool) {
        self.show_nodes.set(show);
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_Polygon3D", Self::restore);
    }

    /// `Restore`: `Poly::ReadPolygon3D`; text that does not read gives a drawable
    /// without polygon.
    pub fn restore(
        _session: &DrawSession,
        stream: &mut DrawIStream,
    ) -> Result<Option<DrawableHandle>, Failure> {
        let polygon = stream.read_buffered(|input| poly::read_polygon3d(input));
        Ok(Some(Rc::new(Self::new(polygon.unwrap_or_default()))))
    }
}

impl Drawable3D for DrawTrSurfPolygon3D {
    fn base(&self) -> &Drawable3DBase {
        &self.base
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_Polygon3D"
    }

    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        let nodes = geometry(&self.polygon)?.nodes();
        draw_nodes(display, nodes.as_slice(), self.show_nodes());
        Ok(())
    }

    /// `Copy`: a new drawable on the same polygon.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        Ok(Rc::new(Self::new(self.polygon.clone())))
    }

    /// `Dump`: `Poly::Dump`.
    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let polygon = geometry(&self.polygon)?;
        let real_text = stream.real_text();
        stream
            .write_bytes_through(|out| poly::write_polygon3d_with(polygon, out, false, &*real_text))
    }

    /// `Save`: `Poly::Write` in scientific notation with 15 digits; the flags are left
    /// set afterwards, as the native `setf(aFlags)` (which only adds flags) leaves them.
    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let polygon = geometry(&self.polygon)?;
        stream.set_scientific(true).set_precision(15);
        let real_text = stream.real_text();
        stream
            .write_bytes_through(|out| poly::write_polygon3d_with(polygon, out, true, &*real_text))
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("polygon3D")?;
        Ok(())
    }
}

/// `DrawTrSurf_Polygon2D`: a `Poly_Polygon2D` drawn as a yellow polyline (a 2D
/// object).
pub struct DrawTrSurfPolygon2D {
    base: Drawable3DBase,
    polygon: Handle<Polygon2D>,
    show_nodes: Cell<bool>,
}

impl DrawTrSurfPolygon2D {
    /// `DrawTrSurf_Polygon2D(P)`: nodes not marked.
    pub fn new(polygon: Handle<Polygon2D>) -> Self {
        Self {
            base: Drawable3DBase::new(),
            polygon,
            show_nodes: Cell::new(false),
        }
    }

    /// `Polygon2D`.
    pub fn polygon(&self) -> Handle<Polygon2D> {
        self.polygon.clone()
    }

    /// `ShowNodes()`.
    pub fn show_nodes(&self) -> bool {
        self.show_nodes.get()
    }

    /// `ShowNodes(bool)`.
    pub fn set_show_nodes(&self, show: bool) {
        self.show_nodes.set(show);
    }

    /// `RegisterFactory` (the `Draw_Drawable3D_FACTORY` macro).
    pub fn register_factory(session: &DrawSession) {
        session.register_factory("DrawTrSurf_Polygon2D", Self::restore);
    }

    /// `Restore`: `Poly::ReadPolygon2D`; text that does not read gives a drawable
    /// without polygon.
    pub fn restore(
        _session: &DrawSession,
        stream: &mut DrawIStream,
    ) -> Result<Option<DrawableHandle>, Failure> {
        let polygon = stream.read_buffered(|input| poly::read_polygon2d(input));
        Ok(Some(Rc::new(Self::new(polygon.unwrap_or_default()))))
    }
}

impl Drawable3D for DrawTrSurfPolygon2D {
    fn base(&self) -> &Drawable3DBase {
        &self.base
    }

    fn type_name(&self) -> &'static str {
        "DrawTrSurf_Polygon2D"
    }

    fn draw_on(&self, display: &mut DrawDisplay<'_>) -> Result<(), Failure> {
        let nodes = geometry(&self.polygon)?.nodes();
        draw_nodes(display, nodes.as_slice(), self.show_nodes());
        Ok(())
    }

    fn is_3d(&self) -> bool {
        false
    }

    /// `Copy`: a new drawable on the same polygon.
    fn copy(&self, _this: &DrawableHandle) -> Result<DrawableHandle, Failure> {
        Ok(Rc::new(Self::new(self.polygon.clone())))
    }

    /// `Dump`: `Poly::Dump`.
    fn dump(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let polygon = geometry(&self.polygon)?;
        let real_text = stream.real_text();
        stream
            .write_bytes_through(|out| poly::write_polygon2d_with(polygon, out, false, &*real_text))
    }

    /// `Save`: `Poly::Write` in scientific notation with 15 digits; the flags are left
    /// set afterwards, as the native `setf(aFlags)` (which only adds flags) leaves them.
    fn save(&self, stream: &mut DrawOStream) -> Result<(), Failure> {
        let polygon = geometry(&self.polygon)?;
        stream.set_scientific(true).set_precision(15);
        let real_text = stream.real_text();
        stream
            .write_bytes_through(|out| poly::write_polygon2d_with(polygon, out, true, &*real_text))
    }

    fn whatis(&self, di: &DrawInterpretor) -> Result<(), Failure> {
        di.append("polygon2D")?;
        Ok(())
    }
}
