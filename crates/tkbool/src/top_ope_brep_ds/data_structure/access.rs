// Rust translation of the accessors of TopOpeBRepDS_DataStructure (the geometries and
// their interferences, the shapes by index, the section edges, the new surfaces and the
// state maps in ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_DataStructure.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The accessors of the data structure. An unknown geometry gives the empty one of the
//! structure (mutable natively, so a change to it stays visible), as natively.
use super::{MapOfShapeWithState, TopOpeBRepDSDataStructure};
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSCurve, TopOpeBRepDSPoint, TopOpeBRepDSShapeWithState,
    TopOpeBRepDSSurface,
};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::IndexedMap;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;

impl TopOpeBRepDSDataStructure {
    /// `Surface(I)`: the surface, or the empty one; `ds.surface(1)`.
    pub fn surface(&self, i: i32) -> &TopOpeBRepDSSurface {
        self.my_surfaces
            .seek(&i)
            .map_or(&self.my_empty_surface, |d| &d.my_surface)
    }

    /// `ChangeSurface(I)`; `ds.change_surface(1).change_keep(true)`.
    pub fn change_surface(&mut self, i: i32) -> &mut TopOpeBRepDSSurface {
        match self.my_surfaces.change_seek(&i) {
            Some(d) => &mut d.my_surface,
            None => &mut self.my_empty_surface,
        }
    }

    /// `Curve(I)`: the curve, or the empty one; `ds.curve(1)`.
    pub fn curve(&self, i: i32) -> &TopOpeBRepDSCurve {
        self.my_curves
            .seek(&i)
            .map_or(&self.my_empty_curve, |d| &d.my_curve)
    }

    /// `ChangeCurve(I)`; `ds.change_curve(1).change_keep(false)`.
    pub fn change_curve(&mut self, i: i32) -> &mut TopOpeBRepDSCurve {
        match self.my_curves.change_seek(&i) {
            Some(d) => &mut d.my_curve,
            None => &mut self.my_empty_curve,
        }
    }

    /// `Point(I)`: the point (the empty one when removed); Standard_ProgramError out of
    /// 1..=NbPoints; `ds.point(1)?`.
    pub fn point(&self, i: i32) -> Result<&TopOpeBRepDSPoint, Failure> {
        self.check_point_index(i)?;
        Ok(self
            .my_points
            .seek(&i)
            .map_or(&self.my_empty_point, |d| &d.my_point))
    }

    /// `ChangePoint(I)`; `ds.change_point(1)?.change_keep(false)`.
    pub fn change_point(&mut self, i: i32) -> Result<&mut TopOpeBRepDSPoint, Failure> {
        self.check_point_index(i)?;
        Ok(match self.my_points.change_seek(&i) {
            Some(d) => &mut d.my_point,
            None => &mut self.my_empty_point,
        })
    }

    /// `SurfaceInterferences(I)`; `ds.surface_interferences(1)`.
    pub fn surface_interferences(&self, i: i32) -> &ListOfInterference {
        self.my_surfaces
            .seek(&i)
            .map_or(&self.my_empty_list_of_interference, |d| d.interferences())
    }

    /// `ChangeSurfaceInterferences(I)`: the shared empty list for an unknown surface;
    /// `ds.change_surface_interferences(1).append(i)`.
    pub fn change_surface_interferences(&mut self, i: i32) -> &mut ListOfInterference {
        match self.my_surfaces.change_seek(&i) {
            Some(d) => d.change_interferences(),
            None => &mut self.my_empty_list_of_interference,
        }
    }

    /// `CurveInterferences(I)`; `ds.curve_interferences(1)`.
    pub fn curve_interferences(&self, i: i32) -> &ListOfInterference {
        self.my_curves
            .seek(&i)
            .map_or(&self.my_empty_list_of_interference, |d| d.interferences())
    }

    /// `ChangeCurveInterferences(I)`; `ds.change_curve_interferences(1).append(i)`.
    pub fn change_curve_interferences(&mut self, i: i32) -> &mut ListOfInterference {
        match self.my_curves.change_seek(&i) {
            Some(d) => d.change_interferences(),
            None => &mut self.my_empty_list_of_interference,
        }
    }

    /// `PointInterferences(I)`; `ds.point_interferences(1)`.
    pub fn point_interferences(&self, i: i32) -> &ListOfInterference {
        self.my_points
            .seek(&i)
            .map_or(&self.my_empty_list_of_interference, |d| d.interferences())
    }

    /// `ChangePointInterferences(I)`; `ds.change_point_interferences(1).append(i)`.
    pub fn change_point_interferences(&mut self, i: i32) -> &mut ListOfInterference {
        match self.my_points.change_seek(&i) {
            Some(d) => d.change_interferences(),
            None => &mut self.my_empty_list_of_interference,
        }
    }

    /// `Shape(I, FindKeep)` (FindKeep natively defaulted to true): the shape, or the
    /// null shape when it is not kept or unknown; `ds.shape(1, true)`.
    pub fn shape(&self, i: i32, find_keep: bool) -> &Shape {
        if self.keep_shape_index(i, find_keep)
            && let Ok(s) = self.my_shapes.find_key(i as usize)
        {
            return s;
        }
        &self.my_empty_shape
    }

    /// `Shape(S, FindKeep)`: the index of the shape, 0 when it is not kept or unknown;
    /// `ds.shape_index(&face, true)`.
    pub fn shape_index(&self, s: &Shape, find_keep: bool) -> i32 {
        if self.has_shape(s, find_keep) {
            return self.my_shapes.find_index(s) as i32;
        }
        0
    }

    /// `SectionEdge(I, FindKeep)`: the section edge of index `i` when its shape is kept,
    /// else the null edge; Standard_OutOfRange for an unknown index;
    /// `ds.section_edge(1, true)?`.
    pub fn section_edge(&self, i: i32, find_keep: bool) -> Result<Edge, Failure> {
        let s = self
            .my_section_edges
            .find_key(usize::try_from(i).unwrap_or(0))?;
        if self.has_shape(s, find_keep) {
            return topods::edge(s);
        }
        topods::edge(&self.my_empty_shape)
    }

    /// `SectionEdge(E, FindKeep)`: the section index of a kept edge, else 0;
    /// `ds.section_edge_index(&e, true)`.
    pub fn section_edge_index(&self, e: &Edge, find_keep: bool) -> i32 {
        if self.keep_shape(e.as_ref(), find_keep) {
            return self.my_section_edges.find_index(e.as_ref()) as i32;
        }
        0
    }

    /// `IsSectionEdge(E, FindKeep)`; `ds.is_section_edge(&e, true)`.
    pub fn is_section_edge(&self, e: &Edge, find_keep: bool) -> bool {
        self.keep_shape(e.as_ref(), find_keep) && self.my_section_edges.contains(e.as_ref())
    }

    /// `HasGeometry(S)`: a kept shape with interferences; `ds.has_geometry(&face)`.
    pub fn has_geometry(&self, s: &Shape) -> bool {
        self.has_shape(s, true) && !self.shape_interferences(s, true).is_empty()
    }

    /// `HasShape(S, FindKeep)` (FindKeep natively defaulted to true): `KeepShape`;
    /// `ds.has_shape(&face, true)`.
    pub fn has_shape(&self, s: &Shape, find_keep: bool) -> bool {
        self.keep_shape(s, find_keep)
    }

    /// `SetNewSurface(F, S)`; `ds.set_new_surface(&face, &surface)`.
    pub fn set_new_surface(&mut self, f: &Shape, s: &Handle<dyn GeomSurface>) {
        self.my_new_surface.bind(f.clone(), s.clone());
    }

    /// `HasNewSurface(F)`; `ds.has_new_surface(&face)`.
    pub fn has_new_surface(&self, f: &Shape) -> bool {
        self.my_new_surface.is_bound(f)
    }

    /// `NewSurface(F)`: the new surface of the face, null without one;
    /// `ds.new_surface(&face)`.
    pub fn new_surface(&self, f: &Shape) -> &Handle<dyn GeomSurface> {
        self.my_new_surface
            .seek(f)
            .unwrap_or(&self.my_empty_gsurface)
    }

    /// `ChangeMapOfShapeWithStateObj()`; `ds.change_map_of_shape_with_state_obj()`.
    pub fn change_map_of_shape_with_state_obj(&mut self) -> &mut MapOfShapeWithState {
        &mut self.my_map_of_shape_with_state_obj
    }

    /// `ChangeMapOfShapeWithStateTool()`; `ds.change_map_of_shape_with_state_tool()`.
    pub fn change_map_of_shape_with_state_tool(&mut self) -> &mut MapOfShapeWithState {
        &mut self.my_map_of_shape_with_state_tool
    }

    /// `ChangeMapOfShapeWithState(aShape, aFlag)`: the object or tool map holding the
    /// shape and true, else a dummy map and false (a function static shared by all the
    /// data structures natively, one per data structure here);
    /// `let (map, found) = ds.change_map_of_shape_with_state(&face);`.
    pub fn change_map_of_shape_with_state(
        &mut self,
        a_shape: &Shape,
    ) -> (&mut MapOfShapeWithState, bool) {
        if self.my_map_of_shape_with_state_obj.contains(a_shape) {
            return (&mut self.my_map_of_shape_with_state_obj, true);
        }
        if self.my_map_of_shape_with_state_tool.contains(a_shape) {
            return (&mut self.my_map_of_shape_with_state_tool, true);
        }
        (&mut self.my_dummy_map_of_shape_with_state, false)
    }

    /// `GetShapeWithState(aShape)`: from the object, else the tool map, else a default
    /// one; `ds.get_shape_with_state(&face)`.
    pub fn get_shape_with_state(&self, a_shape: &Shape) -> &TopOpeBRepDSShapeWithState {
        if let Some(s) = self.my_map_of_shape_with_state_obj.seek(a_shape) {
            return s;
        }
        if let Some(s) = self.my_map_of_shape_with_state_tool.seek(a_shape) {
            return s;
        }
        &self.my_dummy_shape_with_state
    }

    /// `ChangeMapOfRejectedShapesObj()`; `ds.change_map_of_rejected_shapes_obj()`.
    pub fn change_map_of_rejected_shapes_obj(&mut self) -> &mut IndexedMap<Shape, ShapeMapHasher> {
        &mut self.my_map_of_rejected_shapes_obj
    }

    /// `ChangeMapOfRejectedShapesTool()`; `ds.change_map_of_rejected_shapes_tool()`.
    pub fn change_map_of_rejected_shapes_tool(&mut self) -> &mut IndexedMap<Shape, ShapeMapHasher> {
        &mut self.my_map_of_rejected_shapes_tool
    }
}
