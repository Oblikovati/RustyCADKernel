//! The hidden line presentations of a shape for a camera: `StdPrs_HLRShapeI` (the
//! interface), `StdPrs_HLRShape` (exact HLRBRep_Algo through `StdPrs_HLRToolShape`) and
//! `StdPrs_HLRPolyShape` (HLRBRep_PolyAlgo on the triangulation).
use super::{StdPrsDeflectionCurve, StdPrsToolTriangulatedShape, StdPrsWfShape};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhlr::hlr_algo::{EdgeIterator, EdgeStatus, Projector};
use rustycad_tkhlr::hlr_brep::{Algo, Data, PolyAlgo};
use rustycad_tkmath::gp::{Axis3, Point3, Transform3};
use rustycad_tkservice::graphic3d::*;

/// `StdPrs_HLRShapeI`: computes the hidden line presentation of a shape.
pub trait StdPrsHlrShapeI: Transient {
    /// `ComputeHLR` (pure virtual): the seen (and hidden when the drawer asks) lines of the
    /// shape for the camera; `algo.compute_hlr(&prs, &shape, &drawer, &camera)?`.
    fn compute_hlr(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
        camera: &Handle<Graphic3dCamera>,
    ) -> Result<(), Failure>;
}
rustycad_tkernel::impl_transient_interface!(
    dyn StdPrsHlrShapeI,
    "StdPrs_HLRShapeI",
    StandardTransient
);

/// The HLR projector of a camera: the frame at the camera center looking back along the
/// direction, X = Up ^ back, perspective unless orthographic, with the camera scale as focus.
/// Example: `std_prs_hlr_projector(&camera)?`.
pub fn std_prs_hlr_projector(camera: &Graphic3dCamera) -> Result<Projector, Failure> {
    let back = camera.direction().reversed();
    let x_dir = camera.up().crossed(back)?;
    let frame = Axis3::new(camera.center(), back, x_dir)?;
    let mut transform = Transform3::default();
    transform.set_transformation_to(frame);
    Projector::with_transformation(transform, !camera.is_orthographic(), camera.scale())
}

/// `StdPrs_HLRToolShape`: the visible and hidden parts of the edges of a shape after the
/// exact hidden line removal (no isolines).
/// Example: `let mut tool = StdPrsHlrToolShape::new(&shape, &projector)?;`.
pub struct StdPrsHlrToolShape {
    data: Handle<Data>,
    current_edge: i32,
    parts: Vec<(f64, f64)>,
    part: usize,
}

impl StdPrsHlrToolShape {
    /// Runs HLRBRep_Algo on the shape (0 isolines) with the projector.
    /// Example: `StdPrsHlrToolShape::new(&shape, &projector)?`.
    pub fn new(shape: &Shape, projector: &Projector) -> Result<Self, Failure> {
        let hider = Algo::new()?;
        hider.add(shape, 0);
        hider.set_projector(projector);
        hider.update()?;
        hider.hide()?;
        Ok(Self {
            data: hider.data_structure(),
            current_edge: 0,
            parts: Vec::new(),
            part: 0,
        })
    }
    /// `NbEdges`; `tool.nb_edges()?`.
    pub fn nb_edges(&self) -> Result<i32, Failure> {
        Ok(self.data.get().ok_or_else(no_data)?.nb_edges())
    }
    fn status(&self, edge: i32) -> Result<EdgeStatus, Failure> {
        let data = self.data.get().ok_or_else(no_data)?;
        let edges = data.e_data_array();
        let edge_data = edges
            .value(edge)
            .map_err(|e| e.into_failure("HLRBRep_Data::EDataArray"))?;
        Ok(edge_data.status().clone())
    }
    /// `InitVisible`: the visible parts of an edge; `tool.init_visible(1)?`.
    pub fn init_visible(&mut self, edge_number: i32) -> Result<(), Failure> {
        let status = self.status(edge_number)?;
        let mut iterator = EdgeIterator::new();
        iterator.init_visible(&status);
        self.parts.clear();
        while iterator.more_visible() {
            let (start, _, end, _) = iterator.visible()?;
            self.parts.push((start, end));
            iterator.next_visible();
        }
        (self.part, self.current_edge) = (0, edge_number);
        Ok(())
    }
    /// `MoreVisible`; `tool.more_visible()`.
    pub fn more_visible(&self) -> bool {
        self.part < self.parts.len()
    }
    /// `NextVisible`; `tool.next_visible()`.
    pub fn next_visible(&mut self) {
        self.part += 1;
    }
    /// `Visible`: the curve of the current edge and the bounds of the current part.
    /// Example: `let (curve, u1, u2) = tool.visible()?;`.
    pub fn visible(&self) -> Result<(BRepAdaptorCurve, f64, f64), Failure> {
        self.current_part()
    }
    /// `InitHidden`: the hidden parts of an edge; `tool.init_hidden(1)?`.
    pub fn init_hidden(&mut self, edge_number: i32) -> Result<(), Failure> {
        let status = self.status(edge_number)?;
        let mut iterator = EdgeIterator::new();
        iterator.init_hidden(&status)?;
        self.parts.clear();
        while iterator.more_hidden() {
            let (start, _, end, _) = iterator.hidden();
            self.parts.push((start, end));
            iterator.next_hidden()?;
        }
        (self.part, self.current_edge) = (0, edge_number);
        Ok(())
    }
    /// `MoreHidden`; `tool.more_hidden()`.
    pub fn more_hidden(&self) -> bool {
        self.part < self.parts.len()
    }
    /// `NextHidden`; `tool.next_hidden()`.
    pub fn next_hidden(&mut self) {
        self.part += 1;
    }
    /// `Hidden`: the curve of the current edge and the bounds of the current part.
    /// Example: `let (curve, u1, u2) = tool.hidden()?;`.
    pub fn hidden(&self) -> Result<(BRepAdaptorCurve, f64, f64), Failure> {
        self.current_part()
    }
    fn current_part(&self) -> Result<(BRepAdaptorCurve, f64, f64), Failure> {
        let (start, end) = *self.parts.get(self.part).ok_or_else(|| {
            Failure::with_message(
                "Standard_OutOfRange",
                "StdPrs_HLRToolShape: no current part",
            )
        })?;
        let data = self.data.get().ok_or_else(no_data)?;
        let edges = data.e_data_array();
        let edge = edges
            .value(self.current_edge)
            .map_err(|e| e.into_failure("HLRBRep_Data::EDataArray"))?;
        let curve = BRepAdaptorCurve::with_edge(&edge.geometry().curve().edge())?;
        Ok((curve, start, end))
    }
}

fn no_data() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "StdPrs_HLRToolShape: no HLRBRep_Data",
    )
}

/// `StdPrs_HLRShape`: the exact hidden line removal, each part drawn with the deflection
/// curve into the current group (seen lines, then hidden ones when asked).
/// Example: `StdPrsHlrShape::default().compute_hlr(&prs, &shape, &drawer, &camera)?`.
#[derive(Default)]
pub struct StdPrsHlrShape {
    transient: StandardTransient,
}
rustycad_tkernel::impl_transient!(StdPrsHlrShape, "StdPrs_HLRShape", dyn StdPrsHlrShapeI, transient,
    interfaces: [dyn StdPrsHlrShapeI]);

impl StdPrsHlrShapeI for StdPrsHlrShape {
    fn compute_hlr(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
        camera: &Handle<Graphic3dCamera>,
    ) -> Result<(), Failure> {
        let camera = camera.get().ok_or_else(null_camera)?;
        let mut tool = StdPrsHlrToolShape::new(shape, &std_prs_hlr_projector(camera)?)?;
        let nb_edges = tool.nb_edges()?;
        let deviation = drawer.maximal_chordial_deviation();
        let angle = drawer.deviation_angle();
        let group = prs.current_group()?;
        group.set_primitives_aspect(&drawer.seen_line_aspect().aspect().cast());
        // The native point sequence is shared by every call.
        let mut points: Sequence<Point3> = Sequence::default();
        for edge in 1..=nb_edges {
            tool.init_visible(edge)?;
            while tool.more_visible() {
                let (curve, u1, u2) = tool.visible()?;
                StdPrsDeflectionCurve::add_range_points(
                    prs,
                    &curve,
                    u1,
                    u2,
                    deviation,
                    &mut points,
                    angle,
                    true,
                )?;
                tool.next_visible();
            }
        }
        if drawer.draw_hidden_line() {
            group.set_primitives_aspect(&drawer.hidden_line_aspect().aspect().cast());
            for edge in 1..=nb_edges {
                tool.init_hidden(edge)?;
                while tool.more_hidden() {
                    let (curve, u1, u2) = tool.hidden()?;
                    StdPrsDeflectionCurve::add_range_points(
                        prs,
                        &curve,
                        u1,
                        u2,
                        deviation,
                        &mut points,
                        angle,
                        true,
                    )?;
                    tool.next_hidden();
                }
            }
        }
        Ok(())
    }
}

fn null_camera() -> Failure {
    Failure::with_message("Standard_NullObject", "StdPrs_HLRShapeI: null camera")
}

/// `StdPrs_HLRPolyShape`: the hidden line removal on the triangulation; the free vertices
/// of a compound are drawn as wireframe first, the shape is tessellated when the drawer
/// triangulates automatically, and the seen and hidden segments that are not interior
/// regular lines (or are outlines) become one segment array each.
/// Example: `StdPrsHlrPolyShape::default().compute_hlr(&prs, &shape, &drawer, &camera)?`.
#[derive(Default)]
pub struct StdPrsHlrPolyShape {
    transient: StandardTransient,
}
rustycad_tkernel::impl_transient!(StdPrsHlrPolyShape, "StdPrs_HLRPolyShape", dyn StdPrsHlrShapeI, transient,
    interfaces: [dyn StdPrsHlrShapeI]);

impl StdPrsHlrShapeI for StdPrsHlrPolyShape {
    fn compute_hlr(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        shape: &Shape,
        drawer: &Handle<Prs3dDrawer>,
        camera: &Handle<Graphic3dCamera>,
    ) -> Result<(), Failure> {
        let camera = camera.get().ok_or_else(null_camera)?;
        let projector = std_prs_hlr_projector(camera)?;
        let group = prs.current_group()?;
        if shape.shape_type()? == ShapeEnum::Compound {
            for vertex in Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Edge)? {
                StdPrsWfShape::add(prs, &vertex, drawer)?;
            }
        }
        if drawer.is_auto_triangulation() {
            StdPrsToolTriangulatedShape::tessellate(shape, drawer)?;
        }
        let hider = PolyAlgo::with_shape(shape)?;
        hider.set_projector(&projector);
        hider.update()?;
        let (mut seen, mut hidden) = (Vec::new(), Vec::new());
        hider.init_hide()?;
        while hider.more_hide() {
            let mut status = EdgeStatus::default();
            let segment = hider.hide(&mut status)?;
            let (p1, p2) = (segment.points.pnt1, segment.points.pnt2);
            let delta = p2 - p1;
            let kept = !segment.regn || segment.outl;
            let at = |t: f64| Point3::from(p1 + delta * t);
            let mut iterator = EdgeIterator::new();
            iterator.init_visible(&status);
            while iterator.more_visible() {
                let (start, _, end, _) = iterator.visible()?;
                seen.push((at(start), at(end), kept));
                iterator.next_visible();
            }
            iterator.init_hidden(&status)?;
            while iterator.more_hidden() {
                let (start, _, end, _) = iterator.hidden();
                hidden.push((at(start), at(end), kept));
                iterator.next_hidden()?;
            }
            hider.next_hide()?;
        }
        if drawer.draw_hidden_line() {
            add_segments(
                &group,
                &hidden,
                &drawer.hidden_line_aspect().aspect().cast(),
            )?;
        }
        add_segments(&group, &seen, &drawer.seen_line_aspect().aspect().cast())
    }
}

/// One segment array of the kept segments, with its aspect; nothing without one.
fn add_segments(
    group: &Handle<dyn Graphic3dGroup>,
    segments: &[(Point3, Point3, bool)],
    aspect: &Handle<dyn Graphic3dAspectsSource>,
) -> Result<(), Failure> {
    let count = segments.iter().filter(|s| s.2).count() as i32;
    if count == 0 {
        return Ok(());
    }
    let array = Handle::new(Graphic3dArrayOfPrimitives::new(
        Graphic3dTypeOfPrimitiveArray::Segments,
        count * 2,
        0,
        0,
        0,
    )?);
    for (p1, p2, _) in segments.iter().filter(|s| s.2) {
        array.add_vertex_pnt(p1)?;
        array.add_vertex_pnt(p2)?;
    }
    group.set_primitives_aspect(aspect);
    group.add_primitive_array(&array, true);
    Ok(())
}
