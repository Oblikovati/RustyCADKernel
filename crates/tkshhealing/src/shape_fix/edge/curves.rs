use super::{ShapeFixEdge, range::temporary_same_range};
use crate::shape_analysis::ShapeAnalysisEdge;
use crate::shape_build::ShapeBuildEdge;
use crate::shape_extend::ShapeExtendStatus;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::toploc::Location;

impl ShapeFixEdge {
    /// Remove a face p-curve whose endpoints miss the vertices;
    /// `tool.fix_remove_pcurve(&edge, &face)?`.
    pub fn fix_remove_pcurve(&self, edge: &Edge, face: &Face) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.fix_remove_pcurve_on_surface(edge, &surface, &location)
    }

    /// Remove inconsistent stored p-curves on a located surface;
    /// `tool.fix_remove_pcurve_on_surface(&edge, &surface, &location)?`.
    pub fn fix_remove_pcurve_on_surface(
        &self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<bool, Failure> {
        self.reset_status();
        let result = ShapeAnalysisEdge::new()
            .check_vertices_with_pcurve_on_surface(edge, surface, location, -1.0, 0)?;
        if result {
            ShapeBuildEdge.remove_pcurve_on_located_surface(edge, surface, location)?;
        }
        // The native removal methods deliberately leave their own status at OK.
        Ok(result)
    }

    /// Remove a 3D curve whose endpoints miss the vertices;
    /// `tool.fix_remove_curve_3d(&edge)?`.
    pub fn fix_remove_curve_3d(&self, edge: &Edge) -> Result<bool, Failure> {
        self.reset_status();
        let result = ShapeAnalysisEdge::new().check_vertices_with_curve_3d(edge, -1.0, 0)?;
        if result {
            ShapeBuildEdge.remove_curve_3d(edge)?;
        }
        Ok(result)
    }

    /// Build missing 3D geometry from p-curves; `tool.fix_add_curve_3d(&edge)?`.
    pub fn fix_add_curve_3d(&self, edge: &Edge) -> Result<bool, Failure> {
        self.reset_status();
        if BRepTool::degenerated(edge)? || ShapeAnalysisEdge::new().has_curve_3d(edge)? {
            return Ok(false);
        }
        if !BRepTool::same_range(edge)? {
            temporary_same_range(edge, precision::p_confusion())?;
        }
        if !ShapeBuildEdge.build_curve_3d(edge) {
            self.add_status(ShapeExtendStatus::Fail1);
            return Ok(false);
        }
        self.add_status(ShapeExtendStatus::Done1);
        Ok(true)
    }

    /// Reverse a p-curve when the 3D curve requires it; `tool.fix_reversed_2d(&edge, &face)?`.
    pub fn fix_reversed_2d(&self, edge: &Edge, face: &Face) -> Result<bool, Failure> {
        let (surface, location) = BRepTool::surface_with_location(face)?;
        self.fix_reversed_2d_on_surface(edge, &surface, &location)
    }

    /// Reverse the existing p-curve in place, preserving seam representations;
    /// `tool.fix_reversed_2d_on_surface(&edge, &surface, &location)?`.
    pub fn fix_reversed_2d_on_surface(
        &self,
        edge: &Edge,
        surface: &Handle<dyn GeomSurface>,
        location: &Location,
    ) -> Result<bool, Failure> {
        self.reset_status();
        let mut analyzer = ShapeAnalysisEdge::new();
        analyzer.check_curve_3d_with_pcurve_on_surface(edge, surface, location)?;
        for flag in [ShapeExtendStatus::Fail1, ShapeExtendStatus::Fail2] {
            if analyzer.status_flag(flag) {
                self.add_status(flag);
            }
        }
        if !analyzer.status_flag(ShapeExtendStatus::Done) {
            return Ok(false);
        }
        reverse_pcurve(edge, surface, location, &analyzer)?;
        self.add_status(ShapeExtendStatus::Done1);
        Ok(true)
    }
}

fn reverse_pcurve(
    edge: &Edge,
    surface: &Handle<dyn GeomSurface>,
    location: &Location,
    analyzer: &ShapeAnalysisEdge,
) -> Result<(), Failure> {
    let (curve, first, last) = analyzer.pcurve_on_surface(edge, surface, location, false)?;
    let new_first = curve.reversed_parameter(last)?;
    let new_last = curve.reversed_parameter(first)?;
    curve.reverse()?;
    let builder = BRepBuilder::default();
    builder.range_on_surface(edge, surface, location, new_first, new_last)?;
    let (first, last) = BRepTool::range(edge)?;
    if first != new_first || last != new_last {
        builder.same_range(edge, false)?;
        builder.same_parameter(edge, false)?;
    }
    Ok(())
}
