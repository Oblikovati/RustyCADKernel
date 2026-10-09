// Rust translation of ModelingAlgorithms/TKBool/BRepFill/BRepFill_OffsetWire.hxx/.cxx (the
// constructors, Init and the accessors) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepFill_OffsetWire`: the parallels at a distance of the wires of a planar face,
//! trimmed by the bisecting locus of the face.
mod bad_edges;
mod holes;
mod kpart;
mod parallels;
mod perform;
mod perform_bilo;
mod spine;
mod trim;
mod vertices;
mod wires;

use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{DataMap, IndexedDataMap, List};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tktopalgo::brep_mat2d::{
    BRepMAT2dBisectingLocus, BRepMAT2dExplorer, BRepMAT2dLinkTopoBilo,
};
use rustycad_tktopalgo::mat::MatSide;

/// The parallels of the spine elements: the default-hashed (orientation sensitive)
/// indexed map of the native class.
pub(crate) type GeneratedMap = IndexedDataMap<Shape, List<Shape>>;

/// The offset of the wires of a planar face (`BRepFill_OffsetWire`);
/// `let mut ow = BRepFillOffsetWire::new(&face, JoinType::Arc, false)?; ow.perform(0.5, 0.)?;`.
pub struct BRepFillOffsetWire {
    my_spine: Face,
    my_work_spine: Face,
    my_offset: f64,
    my_is_open_result: bool,
    my_shape: Shape,
    my_is_done: bool,
    my_join_type: JoinType,
    my_map: GeneratedMap,
    my_bilo: BRepMAT2dBisectingLocus,
    my_link: BRepMAT2dLinkTopoBilo,
    my_map_spine: DataMap<Shape, Shape, ShapeMapHasher>,
    my_call_gen: bool,
    my_empty: List<Shape>,
}

impl Default for BRepFillOffsetWire {
    /// `BRepFill_OffsetWire()`: not done, no spine.
    fn default() -> Self {
        Self {
            my_spine: Face::default(),
            my_work_spine: Face::default(),
            my_offset: 0.,
            my_is_open_result: false,
            my_shape: Shape::default(),
            my_is_done: false,
            my_join_type: JoinType::Arc,
            my_map: GeneratedMap::default(),
            my_bilo: BRepMAT2dBisectingLocus::new(),
            my_link: BRepMAT2dLinkTopoBilo::new(),
            my_map_spine: DataMap::default(),
            my_call_gen: false,
            my_empty: List::default(),
        }
    }
}

impl BRepFillOffsetWire {
    /// `BRepFill_OffsetWire(Spine, Join, IsOpenResult)` (natively defaulted to
    /// GeomAbs_Arc and a closed result): `Init` of a new offset.
    pub fn new(spine: &Face, join: JoinType, is_open_result: bool) -> Result<Self, Failure> {
        let mut offset = Self::default();
        offset.init(spine, join, is_open_result)?;
        Ok(offset)
    }

    /// `Init(Spine, Join, IsOpenResult)`: the spine cut for the bisectors (its edges cut
    /// at the extrema of curvature and the inflexions) and, unless it is a single
    /// circle or open, its bisecting locus on the left and the links to its topology;
    /// `ow.init(&face, JoinType::Arc, false)?`.
    pub fn init(
        &mut self,
        spine: &Face,
        join: JoinType,
        is_open_result: bool,
    ) -> Result<(), Failure> {
        self.my_is_done = false;
        self.my_spine = topods::face(&spine.oriented(Orientation::Forward))?;
        self.my_join_type = join;
        self.my_is_open_result = is_open_result;
        self.my_map.clear(true);
        self.my_map_spine.clear(true);
        let mut exp = BRepMAT2dExplorer::new();
        exp.perform(&self.my_spine)?;
        self.my_spine = topods::face(&exp.modified_shape(self.my_spine.as_ref()))?;
        self.prepare_spine()?;
        let (mut a_shape, mut a_map, mut done) = (Shape::default(), GeneratedMap::default(), false);
        if kpart::k_part_circle(
            &self.my_work_spine,
            1.,
            self.my_is_open_result,
            0.,
            (&mut a_shape, &mut a_map, &mut done),
        )? {
            return Ok(());
        }
        exp.perform(&self.my_work_spine)?;
        self.my_bilo.compute(
            &mut exp,
            1,
            MatSide::Left,
            self.my_join_type,
            self.my_is_open_result,
        )?;
        self.my_link.perform(&exp, &self.my_bilo)
    }

    /// `IsDone()`; `ow.is_done()`.
    pub fn is_done(&self) -> bool {
        self.my_is_done
    }

    /// `Spine()`: the spine after the cut of the explorer; `ow.spine()`.
    pub fn spine(&self) -> &Face {
        &self.my_spine
    }

    /// `Shape()`: the offset wire, or a compound of wires; `ow.shape()`.
    pub fn shape(&self) -> &Shape {
        &self.my_shape
    }

    /// `JoinType()`; `ow.join_type()`.
    pub fn join_type(&self) -> JoinType {
        self.my_join_type
    }

    /// `GeneratedShapes(SpineShape)`: the parallels of an element of the spine (empty
    /// for none); the first call moves the parallels of the cut parts of an edge to the
    /// edge (in the iteration order of the spine map, which depends on the shape
    /// addresses natively); `let parallels = ow.generated_shapes(&edge)?;`.
    pub fn generated_shapes(&mut self, spine_shape: &Shape) -> Result<&List<Shape>, Failure> {
        if !self.my_call_gen {
            if !self.my_map_spine.is_empty() {
                // myMapSpine can be empty if passed by PerformWithBilo.
                self.merge_cut_parallels()?;
            }
            self.my_call_gen = true;
        }
        Ok(self.my_map.seek(spine_shape).unwrap_or(&self.my_empty))
    }

    /// The parallels of each cut part (in both orientations) appended to those of its
    /// original element, the part removed.
    fn merge_cut_parallels(&mut self) -> Result<(), Failure> {
        let pairs: Vec<(Shape, Shape)> = self
            .my_map_spine
            .items()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (key, value) in pairs {
            self.merge_part(&key, &value)?;
            self.merge_part(&key.reversed(), &value.reversed())?;
        }
        Ok(())
    }

    /// One part merged into its original element.
    fn merge_part(&mut self, key: &Shape, value: &Shape) -> Result<(), Failure> {
        if !self.my_map.contains(key) {
            return Ok(());
        }
        if !self.my_map.contains(value) {
            self.my_map.add(value.clone(), List::default());
        }
        if value.is_same(key) {
            return Ok(());
        }
        let mut moved: List<Shape> = std::mem::take(self.my_map.change_from_key(key)?);
        self.my_map.change_from_key(value)?.append_list(&mut moved);
        self.my_map.remove_key(key);
        Ok(())
    }
}
