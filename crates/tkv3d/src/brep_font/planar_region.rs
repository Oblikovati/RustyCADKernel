//! `BRepFont_PlanarRegion`: the regularized planar topology of one glyph (vertex, curve,
//! oriented use, loop and material-region pools addressed by compact typed indices).
use super::planar_region_curve::BRepFontPlanarRegionCurve;
use rustycad_tkernel::ncollection::LinearVector;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Xy;
use std::marker::PhantomData;

/// `BRepFont_PlanarRegion::Entity`: the pool an index addresses.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRepFontPlanarRegionEntity {
    /// `Vertex`.
    Vertex = 0,
    /// `Curve`.
    Curve = 1,
    /// `Loop`.
    Loop = 2,
}

/// The pool marker of a typed index (the `TheEntity` template argument of `Id`).
pub trait BRepFontPlanarRegionEntityKind {
    /// The pool this marker selects; `BRepFontVertexEntity::ENTITY`.
    const ENTITY: BRepFontPlanarRegionEntity;
}

/// Marker of the vertex pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BRepFontVertexEntity;
/// Marker of the curve pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BRepFontCurveEntity;
/// Marker of the loop pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BRepFontLoopEntity;

impl BRepFontPlanarRegionEntityKind for BRepFontVertexEntity {
    const ENTITY: BRepFontPlanarRegionEntity = BRepFontPlanarRegionEntity::Vertex;
}
impl BRepFontPlanarRegionEntityKind for BRepFontCurveEntity {
    const ENTITY: BRepFontPlanarRegionEntity = BRepFontPlanarRegionEntity::Curve;
}
impl BRepFontPlanarRegionEntityKind for BRepFontLoopEntity {
    const ENTITY: BRepFontPlanarRegionEntity = BRepFontPlanarRegionEntity::Loop;
}

/// `BRepFont_PlanarRegion::Id<TheEntity>`: a strongly typed compact index into one pool.
/// Indices of different pools do not compare. Example: `BRepFontCurveId::new(7).value() == 7`.
#[derive(Debug)]
pub struct BRepFontPlanarRegionId<E> {
    value: u32,
    entity: PhantomData<E>,
}

impl<E> Clone for BRepFontPlanarRegionId<E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<E> Copy for BRepFontPlanarRegionId<E> {}

impl<E> PartialEq for BRepFontPlanarRegionId<E> {
    /// `operator==` (and `operator!=` as its negation): equal values.
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<E> Eq for BRepFontPlanarRegionId<E> {}

impl<E> std::hash::Hash for BRepFontPlanarRegionId<E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<E> BRepFontPlanarRegionId<E> {
    /// `Id(theValue)`; `BRepFontVertexId::new(3)`.
    pub const fn new(value: u32) -> Self {
        Self {
            value,
            entity: PhantomData,
        }
    }
    /// `Value`; `BRepFontVertexId::new(3).value() == 3`.
    pub const fn value(&self) -> u32 {
        self.value
    }
}

/// `BRepFont_PlanarRegion::VertexId`.
pub type BRepFontVertexId = BRepFontPlanarRegionId<BRepFontVertexEntity>;
/// `BRepFont_PlanarRegion::CurveId`.
pub type BRepFontCurveId = BRepFontPlanarRegionId<BRepFontCurveEntity>;
/// `BRepFont_PlanarRegion::LoopId`.
pub type BRepFontLoopId = BRepFontPlanarRegionId<BRepFontLoopEntity>;

/// `BRepFont_PlanarRegion::Use`: the oriented use of a normalized curve interval by one loop.
/// The parameters increase in the curve's [0, 1] domain; the orientation is kept apart so
/// adapters can share the curve geometry.
/// Example: `BRepFontPlanarRegionUse::new(curve, first, last, 0.25, 0.75, true)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRepFontPlanarRegionUse {
    curve: BRepFontCurveId,
    first_vertex: BRepFontVertexId,
    last_vertex: BRepFontVertexId,
    first_parameter: f64,
    last_parameter: f64,
    is_reversed: bool,
}

impl BRepFontPlanarRegionUse {
    /// `Use(theCurve, theFirstVertex, theLastVertex, theFirstParameter, theLastParameter,
    /// theIsReversed)`; `BRepFontPlanarRegionUse::new(c, a, b, 0.0, 1.0, false)`.
    pub fn new(
        curve: BRepFontCurveId,
        first_vertex: BRepFontVertexId,
        last_vertex: BRepFontVertexId,
        first_parameter: f64,
        last_parameter: f64,
        is_reversed: bool,
    ) -> Self {
        Self {
            curve,
            first_vertex,
            last_vertex,
            first_parameter,
            last_parameter,
            is_reversed,
        }
    }
    /// `Use(theCurve, theFirstVertex, theLastVertex)` with the default whole forward range;
    /// `BRepFontPlanarRegionUse::forward(c, a, b)`.
    pub fn forward(
        curve: BRepFontCurveId,
        first: BRepFontVertexId,
        last: BRepFontVertexId,
    ) -> Self {
        Self::new(curve, first, last, 0.0, 1.0, false)
    }
    /// `CurveIndex`; `use_.curve_index().value()`.
    pub fn curve_index(&self) -> &BRepFontCurveId {
        &self.curve
    }
    /// `FirstVertex`; `use_.first_vertex()`.
    pub fn first_vertex(&self) -> &BRepFontVertexId {
        &self.first_vertex
    }
    /// `LastVertex`; `use_.last_vertex()`.
    pub fn last_vertex(&self) -> &BRepFontVertexId {
        &self.last_vertex
    }
    /// `FirstParameter`; `use_.first_parameter()`.
    pub fn first_parameter(&self) -> f64 {
        self.first_parameter
    }
    /// `LastParameter`; `use_.last_parameter()`.
    pub fn last_parameter(&self) -> f64 {
        self.last_parameter
    }
    /// `IsReversed`; `use_.is_reversed()`.
    pub fn is_reversed(&self) -> bool {
        self.is_reversed
    }
}

/// `BRepFont_PlanarRegion::Loop`: an ordered closed-loop range in `Uses()`.
/// Example: `BRepFontPlanarRegionLoop::new(0, 4, 200.0, bounds)`.
#[derive(Clone, Debug)]
pub struct BRepFontPlanarRegionLoop {
    first_use: u32,
    nb_uses: u32,
    signed_area: f64,
    bounds: BoundingBox2,
}

impl BRepFontPlanarRegionLoop {
    /// `Loop(theFirstUse, theNbUses, theSignedArea, theBounds)`;
    /// `BRepFontPlanarRegionLoop::new(0, 4, 200.0, bounds)`.
    pub fn new(first_use: u32, nb_uses: u32, signed_area: f64, bounds: BoundingBox2) -> Self {
        Self {
            first_use,
            nb_uses,
            signed_area,
            bounds,
        }
    }
    /// `FirstUse`; `loop_.first_use()`.
    pub fn first_use(&self) -> u32 {
        self.first_use
    }
    /// `NbUses`; `loop_.nb_uses()`.
    pub fn nb_uses(&self) -> u32 {
        self.nb_uses
    }
    /// `SignedArea`: positive for a counter-clockwise loop; `loop_.signed_area()`.
    pub fn signed_area(&self) -> f64 {
        self.signed_area
    }
    /// `Bounds`; `loop_.bounds()`.
    pub fn bounds(&self) -> &BoundingBox2 {
        &self.bounds
    }
}

/// `BRepFont_PlanarRegion::Region`: one material region, an outer loop and a packed range
/// of hole-loop ids. Example: `BRepFontPlanarRegionRegion::new(BRepFontLoopId::new(0), 0, 1)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BRepFontPlanarRegionRegion {
    outer_loop: BRepFontLoopId,
    first_hole: u32,
    nb_holes: u32,
}

impl BRepFontPlanarRegionRegion {
    /// `Region(theOuterLoop, theFirstHole, theNbHoles)`;
    /// `BRepFontPlanarRegionRegion::new(BRepFontLoopId::new(0), 0, 1)`.
    pub fn new(outer_loop: BRepFontLoopId, first_hole: u32, nb_holes: u32) -> Self {
        Self {
            outer_loop,
            first_hole,
            nb_holes,
        }
    }
    /// `OuterLoop`; `region.outer_loop()`.
    pub fn outer_loop(&self) -> &BRepFontLoopId {
        &self.outer_loop
    }
    /// `FirstHole`; `region.first_hole()`.
    pub fn first_hole(&self) -> u32 {
        self.first_hole
    }
    /// `NbHoles`; `region.nb_holes()`.
    pub fn nb_holes(&self) -> u32 {
        self.nb_holes
    }
}

/// `BRepFont_PlanarRegion`: regularized planar glyph topology. Only the regularizer
/// (the native friend) fills it; readers see the pools through the getters.
/// Example: `let region = result.planar_region().unwrap(); region.loops().len()`.
#[derive(Clone, Debug, Default)]
pub struct BRepFontPlanarRegion {
    pub(super) units_per_em: f64,
    pub(super) tolerance: f64,
    pub(super) is_single_stroke: bool,
    pub(super) vertices: LinearVector<Xy>,
    pub(super) curves: LinearVector<BRepFontPlanarRegionCurve>,
    pub(super) uses: LinearVector<BRepFontPlanarRegionUse>,
    pub(super) loops: LinearVector<BRepFontPlanarRegionLoop>,
    pub(super) hole_loops: LinearVector<BRepFontLoopId>,
    pub(super) regions: LinearVector<BRepFontPlanarRegionRegion>,
}

impl BRepFontPlanarRegion {
    /// `IsEmpty`: no loop; `region.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.loops.is_empty()
    }
    /// `IsSingleStroke`; `region.is_single_stroke()`.
    pub fn is_single_stroke(&self) -> bool {
        self.is_single_stroke
    }
    /// `UnitsPerEm`; `region.units_per_em()`.
    pub fn units_per_em(&self) -> f64 {
        self.units_per_em
    }
    /// `Tolerance` in design units; `region.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tolerance
    }
    /// `Vertices`; `region.vertices().len()`.
    pub fn vertices(&self) -> &LinearVector<Xy> {
        &self.vertices
    }
    /// `Curves`; `region.curves().len()`.
    pub fn curves(&self) -> &LinearVector<BRepFontPlanarRegionCurve> {
        &self.curves
    }
    /// `Uses`; `region.uses().len()`.
    pub fn uses(&self) -> &LinearVector<BRepFontPlanarRegionUse> {
        &self.uses
    }
    /// `Loops`; `region.loops().len()`.
    pub fn loops(&self) -> &LinearVector<BRepFontPlanarRegionLoop> {
        &self.loops
    }
    /// `HoleLoops`; `region.hole_loops().len()`.
    pub fn hole_loops(&self) -> &LinearVector<BRepFontLoopId> {
        &self.hole_loops
    }
    /// `Regions`; `region.regions().len()`.
    pub fn regions(&self) -> &LinearVector<BRepFontPlanarRegionRegion> {
        &self.regions
    }
}
