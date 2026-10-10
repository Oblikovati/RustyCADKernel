// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_DataStructure.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_DataStructure`: the points, curves and surfaces created by a
//! topological operation, the shapes they interfere with, the same domain relations of
//! the shapes and the section edges.
mod access;
mod curve_explorer;
mod explorer_core;
mod explorer_facade;
mod geometry;
mod point_explorer;
mod same_domain;
mod shapes;

pub use curve_explorer::TopOpeBRepDSCurveExplorer;
pub use point_explorer::TopOpeBRepDSPointExplorer;

use super::{
    ListOfInterference, TopOpeBRepDSCurve, TopOpeBRepDSCurveData, TopOpeBRepDSInterference,
    TopOpeBRepDSPoint, TopOpeBRepDSPointData, TopOpeBRepDSShapeData, TopOpeBRepDSShapeWithState,
    TopOpeBRepDSSurface, TopOpeBRepDSSurfaceData,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, IndexedMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;

/// The shapes with their split parts by state.
pub type MapOfShapeWithState = IndexedDataMap<Shape, TopOpeBRepDSShapeWithState, ShapeMapHasher>;

/// The data structure of a topological operation (`TopOpeBRepDS_DataStructure`);
/// geometries are indexed from 1 in the order they are added, shapes by their index in
/// the shape map; `let mut ds = TopOpeBRepDSDataStructure::new(); ds.add_shape(&face);`.
#[derive(Clone, Default)]
pub struct TopOpeBRepDSDataStructure {
    my_nb_surfaces: i32,
    my_surfaces: DataMap<i32, TopOpeBRepDSSurfaceData>,
    my_nb_curves: i32,
    my_curves: DataMap<i32, TopOpeBRepDSCurveData>,
    my_equivalent_curve_points: DataMap<i32, i32>,
    my_nb_points: i32,
    my_points: DataMap<i32, TopOpeBRepDSPointData>,
    my_shapes: IndexedDataMap<Shape, TopOpeBRepDSShapeData, ShapeMapHasher>,
    my_section_edges: IndexedMap<Shape, ShapeMapHasher>,
    /// `myEmptyListOfInterference`: returned (mutable) for unknown entities, so what
    /// is appended to it stays visible to the next unknown entity, as natively.
    my_empty_list_of_interference: ListOfInterference,
    my_empty_list_of_shape: List<Shape>,
    my_empty_shape: Shape,
    my_empty_point: TopOpeBRepDSPoint,
    my_empty_surface: TopOpeBRepDSSurface,
    my_empty_curve: TopOpeBRepDSCurve,
    my_empty_gsurface: Handle<dyn GeomSurface>,
    my_new_surface: DataMap<Shape, Handle<dyn GeomSurface>, ShapeMapHasher>,
    my_isfafa: bool,
    my_map_of_shape_with_state_obj: MapOfShapeWithState,
    my_map_of_shape_with_state_tool: MapOfShapeWithState,
    my_map_of_rejected_shapes_obj: IndexedMap<Shape, ShapeMapHasher>,
    my_map_of_rejected_shapes_tool: IndexedMap<Shape, ShapeMapHasher>,
    /// The function static `dummy` map of ChangeMapOfShapeWithState natively.
    my_dummy_map_of_shape_with_state: MapOfShapeWithState,
    /// The function static `dummy` of GetShapeWithState natively.
    my_dummy_shape_with_state: TopOpeBRepDSShapeWithState,
}

/// Standard_ProgramError of the data structure.
pub(crate) fn program_error(message: &str) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        message,
        "TopOpeBRepDS_DataStructure",
    )
}

/// Removes the first interference that is the same object as `i`.
fn remove_first_same(l: &mut ListOfInterference, i: &Handle<TopOpeBRepDSInterference>) {
    let address = i.object_address();
    let mut removed = false;
    l.retain(|x| {
        if removed || x.object_address() != address {
            return true;
        }
        removed = true;
        false
    });
}

impl TopOpeBRepDSDataStructure {
    /// `TopOpeBRepDS_DataStructure()`: empty; `TopOpeBRepDSDataStructure::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `Init()`: no geometry, shape, state map, rejected shape or section edge (the new
    /// surfaces and the isfafa flag are kept, as natively); `ds.init()`.
    pub fn init(&mut self) {
        (self.my_nb_surfaces, self.my_nb_curves, self.my_nb_points) = (0, 0, 0);
        self.my_surfaces.clear(false);
        self.my_curves.clear(false);
        self.my_equivalent_curve_points.clear(false);
        self.my_points.clear(false);
        self.my_shapes.clear(false);
        self.my_map_of_shape_with_state_obj.clear(false);
        self.my_map_of_shape_with_state_tool.clear(false);
        self.my_map_of_rejected_shapes_obj.clear(false);
        self.my_map_of_rejected_shapes_tool.clear(false);
        self.init_section_edges();
    }

    /// `NbSurfaces()`; `ds.nb_surfaces()`.
    pub fn nb_surfaces(&self) -> i32 {
        self.my_nb_surfaces
    }

    /// `NbCurves()`; `ds.nb_curves()`.
    pub fn nb_curves(&self) -> i32 {
        self.my_nb_curves
    }

    /// `ChangeNbCurves(N)`; `ds.change_nb_curves(3)`.
    pub fn change_nb_curves(&mut self, n: i32) {
        self.my_nb_curves = n;
    }

    /// `NbPoints()`; `ds.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.my_nb_points
    }

    /// `NbShapes()`; `ds.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        self.my_shapes.len() as i32
    }

    /// `NbSectionEdges()`; `ds.nb_section_edges()`.
    pub fn nb_section_edges(&self) -> i32 {
        self.my_section_edges.len() as i32
    }

    /// `Isfafa(isfafa)`; `ds.set_isfafa(true)`.
    pub fn set_isfafa(&mut self, isfafa: bool) {
        self.my_isfafa = isfafa;
    }

    /// `Isfafa()`; `ds.isfafa()`.
    pub fn isfafa(&self) -> bool {
        self.my_isfafa
    }

    /// `FindInterference(IT, I)` (private): the position of the interference object in
    /// the list.
    fn find_interference(
        l: &ListOfInterference,
        i: &Handle<TopOpeBRepDSInterference>,
    ) -> Option<usize> {
        l.iter()
            .position(|x| x.object_address() == i.object_address())
    }
}
