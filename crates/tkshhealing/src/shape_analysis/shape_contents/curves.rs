use super::*;
use rustycad_tkbrep::topods::{Edge, Wire};
use rustycad_tkg2d::geom2d::{Geom2dOffsetCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomOffsetCurve, GeomTrimmedCurve};
impl ShapeAnalysisShapeContents {
    pub(super) fn analyze_wire(&mut self, wire: &Wire, face: &Face) -> Result<i32, Failure> {
        let mut edges = Explorer::new(wire, ShapeEnum::Edge, ShapeEnum::Shape)?;
        let mut seams = 0;
        while edges.more() {
            let edge = topods::edge(edges.current())?;
            if BRepTool::is_closed_on_face(&edge, face)? {
                seams += 1;
            }
            self.record_trimmed_spatial(&edge, face)?;
            self.record_planar(&edge, face)?;
            edges.next()?;
        }
        Ok(seams)
    }
    fn record_trimmed_spatial(&mut self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        let (curve, _, _) = BRepTool::curve(edge)?;
        if !curve.cast::<GeomTrimmedCurve>().is_null() {
            self.counts.trimmed_curves_3d += 1;
            self.recordings.trimmed_3d.append(face);
        }
        Ok(())
    }
    fn record_planar(&mut self, edge: &Edge, face: &Face) -> Result<(), Failure> {
        let curve = BRepTool::curve_on_face(edge, face)?.pcurve;
        if curve.is_null() {
            self.counts.missing_pcurves += 1;
        } else if !curve.cast::<Geom2dOffsetCurve>().is_null() {
            self.counts.offset_curves += 1;
            self.recordings.offset_curve.append(face);
        } else if !curve.cast::<Geom2dTrimmedCurve>().is_null() {
            self.counts.trimmed_curves_2d += 1;
            self.recordings.trimmed_2d.append(face);
        }
        Ok(())
    }
    pub(super) fn analyze_spatial_curve(&mut self, edge: &Edge) -> Result<(), Failure> {
        let curve = BRepTool::curve_with_location(edge)?.curve;
        if curve.is_null() {
            return Ok(());
        }
        if !curve.cast::<GeomOffsetCurve>().is_null() {
            self.counts.offset_curves += 1;
            self.recordings.offset_curve.append(edge);
        }
        if !curve.is_cn(1)? {
            self.counts.c0_curves += 1;
        }
        Ok(())
    }
}
