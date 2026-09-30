// Rust translation of the generic ModelingAlgorithms/TKGeomAlgo/Intf/Intf_InterferencePolygonPolyhedron.gxx
// (declared per instance, e.g. IntCurveSurface_TheInterferenceOfHInter.hxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Intf_InterferencePolygonPolyhedron: the interference between a polygon, or
//! straight lines, and a polyhedron, generic over the tool traits of its two
//! arguments (docs/geomalgo-design.md section 2).
mod intersect;
mod polygon;

use super::interference::Interference;
use super::tool::Tool;
use crate::intf::nonzero_tolerance;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::{BoundSortBox, BoundingBox3};
use rustycad_tkmath::gp::{Line3, Point3, Vector3};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// The `ToolPolygon3d` of the template: static access to a 3d polygon (natively
/// IntCurveSurface_ThePolygonToolOfHInter and HLRBRep_ThePolygonToolOfInterCSurf).
pub trait Polygon3dTool {
    /// The polygon type (`Polygon3d`).
    type Polygon: ?Sized;
    /// Bounding: the box of the polygon.
    fn bounding(polygon: &Self::Polygon) -> &BoundingBox3;
    /// DeflectionOverEstimation: the tolerance of the polygon.
    fn deflection_over_estimation(polygon: &Self::Polygon) -> f64;
    /// Closed: whether the polygon is closed.
    fn closed(polygon: &Self::Polygon) -> bool;
    /// NbSegments: the number of segments.
    fn nb_segments(polygon: &Self::Polygon) -> i32;
    /// BeginOfSeg: the first point of segment `index` (one-based).
    fn begin_of_seg(polygon: &Self::Polygon, index: i32) -> Result<Point3, Failure>;
    /// EndOfSeg: the last point of segment `index`.
    fn end_of_seg(polygon: &Self::Polygon, index: i32) -> Result<Point3, Failure>;
}

/// The `ToolPolyh` of the template: static access to a triangulated polyhedron
/// (natively IntCurveSurface_ThePolyhedronToolOfHInter and its HLRBRep twin).
pub trait PolyhedronTool {
    /// The polyhedron type (`Polyhedron`).
    type Polyhedron: ?Sized;
    /// Bounding: the box of the polyhedron.
    fn bounding(polyhedron: &Self::Polyhedron) -> &BoundingBox3;
    /// ComponentsBounding: the box of each triangle, indexed like the triangles.
    fn components_bounding(polyhedron: &Self::Polyhedron) -> &Array1<'static, BoundingBox3>;
    /// DeflectionOverEstimation: the tolerance of the polyhedron.
    fn deflection_over_estimation(polyhedron: &Self::Polyhedron) -> f64;
    /// Triangle: the three point indices of triangle `index`.
    fn triangle(polyhedron: &Self::Polyhedron, index: i32) -> Result<[i32; 3], Failure>;
    /// Point: the point of index `index`.
    fn point(polyhedron: &Self::Polyhedron, index: i32) -> Result<Point3, Failure>;
    /// TriConnex: the triangle across the edge `pivot pedge` of triangle `triang` and
    /// its third point, as (returned value, TriCon, OtherP).
    fn tri_connex(
        polyhedron: &Self::Polyhedron,
        triang: i32,
        pivot: i32,
        pedge: i32,
    ) -> Result<(i32, i32, i32), Failure>;
    /// IsOnBound: whether the edge `index1 index2` is a boundary edge.
    fn is_on_bound(
        polyhedron: &Self::Polyhedron,
        index1: i32,
        index2: i32,
    ) -> Result<bool, Failure>;
    /// GetBorderDeflection: the deflection of the boundary edges.
    fn get_border_deflection(polyhedron: &Self::Polyhedron) -> f64;
}

/// The tool types of an instance, held without owning a value of them.
type ToolMarkers<PT, HT> = PhantomData<(fn() -> PT, fn() -> HT)>;

/// Intf_InterferencePolygonPolyhedron<Polygon3d, ToolPolygon3d, Polyhedron, ToolPolyh>:
/// section points between a polygon (or lines) and a polyhedron, read through the
/// `Interference` accessors (`Deref`). `iLin` is the segment (or line) index being
/// intersected, which the section points record as their first address.
///
/// ```
/// use rustycad_tkgeomalgo::intf::{InterferencePolygonPolyhedron, Polygon3dTool, PolyhedronTool};
/// fn empty<PT: Polygon3dTool, HT: PolyhedronTool>() -> usize {
///     InterferencePolygonPolyhedron::<PT, HT>::new().nb_section_points() as usize
/// }
/// ```
pub struct InterferencePolygonPolyhedron<PT: Polygon3dTool, HT: PolyhedronTool> {
    base: Interference,
    begin_of_closed_polygon: bool,
    i_lin: i32,
    tools: ToolMarkers<PT, HT>,
}

impl<PT: Polygon3dTool, HT: PolyhedronTool> Default for InterferencePolygonPolyhedron<PT, HT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<PT: Polygon3dTool, HT: PolyhedronTool> Clone for InterferencePolygonPolyhedron<PT, HT> {
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            begin_of_closed_polygon: self.begin_of_closed_polygon,
            i_lin: self.i_lin,
            tools: PhantomData,
        }
    }
}

impl<PT: Polygon3dTool, HT: PolyhedronTool> Deref for InterferencePolygonPolyhedron<PT, HT> {
    type Target = Interference;
    fn deref(&self) -> &Interference {
        &self.base
    }
}

impl<PT: Polygon3dTool, HT: PolyhedronTool> DerefMut for InterferencePolygonPolyhedron<PT, HT> {
    /// The public members of Intf_Interference (Insert) apply to the derived object.
    fn deref_mut(&mut self) -> &mut Interference {
        &mut self.base
    }
}

impl<PT: Polygon3dTool, HT: PolyhedronTool> InterferencePolygonPolyhedron<PT, HT> {
    /// Intf_InterferencePolygonPolyhedron(): an empty interference.
    /// `InterferencePolygonPolyhedron::<PT, HT>::new()`.
    pub fn new() -> Self {
        Self {
            base: Interference::new(false),
            begin_of_closed_polygon: false,
            i_lin: 0,
            tools: PhantomData,
        }
    }

    /// Constructor (thePolyg, thePolyh): computes the polygon interference, with a
    /// grid of the polyhedron built for it; `InterferencePolygonPolyhedron::<PT, HT>::with_polygon(&pg, &ph)?`.
    pub fn with_polygon(
        polygon: &PT::Polygon,
        polyhedron: &HT::Polyhedron,
    ) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.perform_polygon(polygon, polyhedron)?;
        Ok(interference)
    }

    /// Constructor (thePolyg, thePolyh, theBoundSB): the same with a prepared grid of
    /// the polyhedron's triangles; `...::with_polygon_grid(&pg, &ph, &mut grid)?`.
    pub fn with_polygon_grid(
        polygon: &PT::Polygon,
        polyhedron: &HT::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.perform_polygon_grid(polygon, polyhedron, grid)?;
        Ok(interference)
    }

    /// Constructor (theLin, thePolyh): computes the interference of a straight line;
    /// `...::with_line(line, &ph)?`.
    pub fn with_line(line: Line3, polyhedron: &HT::Polyhedron) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.perform_line(line, polyhedron)?;
        Ok(interference)
    }

    /// Constructor (theLins, thePolyh): computes the interference of straight lines;
    /// `...::with_lines(&lines, &ph)?`.
    pub fn with_lines(
        lines: &Array1<'_, Line3>,
        polyhedron: &HT::Polyhedron,
    ) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.perform_lines(lines, polyhedron)?;
        Ok(interference)
    }

    /// Constructor (theLin, thePolyh, theBoundSB); `...::with_line_grid(line, &ph, &mut grid)?`.
    pub fn with_line_grid(
        line: Line3,
        polyhedron: &HT::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.perform_line_grid(line, polyhedron, grid)?;
        Ok(interference)
    }

    /// Constructor (theLins, thePolyh, theBoundSB); `...::with_lines_grid(&lines, &ph, &mut grid)?`.
    pub fn with_lines_grid(
        lines: &Array1<'_, Line3>,
        polyhedron: &HT::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<Self, Failure> {
        let mut interference = Self::new();
        interference.perform_lines_grid(lines, polyhedron, grid)?;
        Ok(interference)
    }

    /// Perform(thePolyg, thePolyh): clears the results; the tolerance is the sum of the
    /// deflections (`Epsilon(1000.)` when null); computes when the boxes meet;
    /// `interference.perform_polygon(&pg, &ph)?`.
    pub fn perform_polygon(
        &mut self,
        polygon: &PT::Polygon,
        polyhedron: &HT::Polyhedron,
    ) -> Result<(), Failure> {
        self.start_polygon(polygon, polyhedron);
        if PT::bounding(polygon).is_out_box(HT::bounding(polyhedron)) {
            return Ok(());
        }
        self.interference(polygon, polyhedron)
    }

    /// Perform(thePolyg, thePolyh, theBoundSB); `interference.perform_polygon_grid(&pg, &ph, &mut grid)?`.
    pub fn perform_polygon_grid(
        &mut self,
        polygon: &PT::Polygon,
        polyhedron: &HT::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<(), Failure> {
        self.start_polygon(polygon, polyhedron);
        if PT::bounding(polygon).is_out_box(HT::bounding(polyhedron)) {
            return Ok(());
        }
        self.interference_grid(polygon, polyhedron, grid)
    }

    /// Perform(theLin, thePolyh): clears the results; the tolerance is the polyhedron's
    /// deflection; intersects the line with the triangles whose boxes meet the line's
    /// box in the polyhedron box; `interference.perform_line(line, &ph)?`.
    pub fn perform_line(
        &mut self,
        line: Line3,
        polyhedron: &HT::Polyhedron,
    ) -> Result<(), Failure> {
        self.start_lines(polyhedron);
        let mut grid: BoundSortBox = polyhedron_grid::<HT>(polyhedron)?;
        self.i_lin = 0;
        self.intersect_line(line, polyhedron, &mut grid)
    }

    /// Perform(theLins, thePolyh): the same for every line, numbered from 1;
    /// `interference.perform_lines(&lines, &ph)?`.
    pub fn perform_lines(
        &mut self,
        lines: &Array1<'_, Line3>,
        polyhedron: &HT::Polyhedron,
    ) -> Result<(), Failure> {
        self.start_lines(polyhedron);
        let mut grid: BoundSortBox = polyhedron_grid::<HT>(polyhedron)?;
        self.intersect_lines(lines, polyhedron, &mut grid)
    }

    /// Perform(theLin, thePolyh, theBoundSB); `interference.perform_line_grid(line, &ph, &mut grid)?`.
    pub fn perform_line_grid(
        &mut self,
        line: Line3,
        polyhedron: &HT::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<(), Failure> {
        self.start_lines(polyhedron);
        self.i_lin = 0;
        self.intersect_line(line, polyhedron, grid)
    }

    /// Perform(theLins, thePolyh, theBoundSB): the lines are read at indices
    /// 1..=Length whatever the array's lower bound (Standard_OutOfRange otherwise), as
    /// natively; `interference.perform_lines_grid(&lines, &ph, &mut grid)?`.
    pub fn perform_lines_grid(
        &mut self,
        lines: &Array1<'_, Line3>,
        polyhedron: &HT::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<(), Failure> {
        self.start_lines(polyhedron);
        self.intersect_lines(lines, polyhedron, grid)
    }

    /// The native `for (iLin = 1; iLin <= theLins.Length(); iLin++)` loop, which
    /// leaves `iLin` at Length + 1.
    fn intersect_lines(
        &mut self,
        lines: &Array1<'_, Line3>,
        polyhedron: &HT::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<(), Failure> {
        self.i_lin = 1;
        while self.i_lin <= lines.len() as i32 {
            let line: Line3 = *lines
                .value(self.i_lin)
                .map_err(|error| error.into_failure("NCollection_Array1::Value"))?;
            self.intersect_line(line, polyhedron, grid)?;
            self.i_lin += 1;
        }
        Ok(())
    }

    fn start_polygon(&mut self, polygon: &PT::Polygon, polyhedron: &HT::Polyhedron) {
        self.base.self_interference(false);
        let tolerance: f64 =
            PT::deflection_over_estimation(polygon) + HT::deflection_over_estimation(polyhedron);
        self.base.tolerance = nonzero_tolerance(tolerance);
    }

    fn start_lines(&mut self, polyhedron: &HT::Polyhedron) {
        self.base.self_interference(false);
        self.base.tolerance = nonzero_tolerance(HT::deflection_over_estimation(polyhedron));
        self.begin_of_closed_polygon = false;
    }

    /// The line from its location to one unit along it, infinite, against every
    /// triangle the grid selects for the line's box in the polyhedron box.
    fn intersect_line(
        &mut self,
        line: Line3,
        polyhedron: &HT::Polyhedron,
        grid: &mut BoundSortBox,
    ) -> Result<(), Failure> {
        let mut line_box = BoundingBox3::default();
        Tool::new().lin_box(line, HT::bounding(polyhedron), &mut line_box);
        let triangles: Vec<i32> = grid.compare_box(&line_box)?.iter().copied().collect();
        let end: Point3 = line.location().translated(Vector3::from(line.direction()));
        for triangle in triangles {
            self.intersect(line.location(), end, true, triangle, polyhedron)?;
        }
        Ok(())
    }
}

/// `PolyhGrid.Initialize(ToolPolyh::Bounding(thePolyh), ToolPolyh::ComponentsBounding(thePolyh))`.
fn polyhedron_grid<HT: PolyhedronTool>(
    polyhedron: &HT::Polyhedron,
) -> Result<BoundSortBox, Failure> {
    let mut grid = BoundSortBox::default();
    grid.initialize_with_enclosing(
        HT::bounding(polyhedron),
        HT::components_bounding(polyhedron).clone(),
    )?;
    Ok(grid)
}
