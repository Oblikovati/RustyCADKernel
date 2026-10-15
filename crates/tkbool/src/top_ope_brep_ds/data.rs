// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_GeometryData,
// TopOpeBRepDS_PointData, TopOpeBRepDS_CurveData, TopOpeBRepDS_SurfaceData,
// TopOpeBRepDS_ShapeData, TopOpeBRepDS_ShapeWithState and TopOpeBRepDS_ListOfShapeOn1State
// (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The records of the data structure: the interferences of each geometry and shape,
//! the same domain data of the shapes, and the split states of the shapes.
use super::{
    TopOpeBRepDSConfig, TopOpeBRepDSCurve, TopOpeBRepDSInterference, TopOpeBRepDSPoint,
    TopOpeBRepDSSurface,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::{Orientation, State};
use std::ops::{Deref, DerefMut};

/// The interferences of an entity of the data structure.
pub type ListOfInterference = List<Handle<TopOpeBRepDSInterference>>;

/// The interferences of a geometry (`TopOpeBRepDS_GeometryData`);
/// `let mut g = TopOpeBRepDSGeometryData::default(); g.add_interference(&i);`.
#[derive(Clone, Default)]
pub struct TopOpeBRepDSGeometryData {
    my_interferences: ListOfInterference,
}

impl TopOpeBRepDSGeometryData {
    /// `Assign(Other)` (and the copy constructor): a copy of the interference handles;
    /// `g.assign(&other)`.
    pub fn assign(&mut self, other: &TopOpeBRepDSGeometryData) {
        self.my_interferences = other.my_interferences.clone();
    }

    /// `Interferences()`; `g.interferences()`.
    pub fn interferences(&self) -> &ListOfInterference {
        &self.my_interferences
    }

    /// `ChangeInterferences()`; `g.change_interferences().append(i)`.
    pub fn change_interferences(&mut self) -> &mut ListOfInterference {
        &mut self.my_interferences
    }

    /// `AddInterference(I)`: appended; `g.add_interference(&i)`.
    pub fn add_interference(&mut self, i: &Handle<TopOpeBRepDSInterference>) {
        self.my_interferences.append(i.clone());
    }
}

/// Implements the `TopOpeBRepDS_GeometryData` base of a data record.
macro_rules! geometry_data_base {
    ($data:ty) => {
        impl Deref for $data {
            type Target = TopOpeBRepDSGeometryData;
            fn deref(&self) -> &TopOpeBRepDSGeometryData {
                &self.base
            }
        }
        impl DerefMut for $data {
            fn deref_mut(&mut self) -> &mut TopOpeBRepDSGeometryData {
                &mut self.base
            }
        }
    };
}

/// A point with its interferences and the indices of the shapes it came from
/// (`TopOpeBRepDS_PointData`); `TopOpeBRepDSPointData::with_shapes(&p, 1, 2)`.
#[derive(Clone, Default)]
pub struct TopOpeBRepDSPointData {
    base: TopOpeBRepDSGeometryData,
    pub(crate) my_point: TopOpeBRepDSPoint,
    my_s1: i32,
    my_s2: i32,
}
geometry_data_base!(TopOpeBRepDSPointData);

impl TopOpeBRepDSPointData {
    /// `TopOpeBRepDS_PointData(P)`: shapes 0; `TopOpeBRepDSPointData::new(&p)`.
    pub fn new(p: &TopOpeBRepDSPoint) -> Self {
        Self {
            my_point: *p,
            ..Self::default()
        }
    }

    /// `TopOpeBRepDS_PointData(P, I1, I2)`; `TopOpeBRepDSPointData::with_shapes(&p, 1, 2)`.
    pub fn with_shapes(p: &TopOpeBRepDSPoint, i1: i32, i2: i32) -> Self {
        Self {
            my_point: *p,
            my_s1: i1,
            my_s2: i2,
            ..Self::default()
        }
    }

    /// `SetShapes(I1, I2)`; `d.set_shapes(1, 2)`.
    pub fn set_shapes(&mut self, i1: i32, i2: i32) {
        (self.my_s1, self.my_s2) = (i1, i2);
    }

    /// `GetShapes(I1, I2)`; `let (i1, i2) = d.get_shapes();`.
    pub fn get_shapes(&self) -> (i32, i32) {
        (self.my_s1, self.my_s2)
    }
}

/// A curve with its interferences (`TopOpeBRepDS_CurveData`);
/// `TopOpeBRepDSCurveData::new(&c)`.
#[derive(Clone, Default)]
pub struct TopOpeBRepDSCurveData {
    base: TopOpeBRepDSGeometryData,
    pub(crate) my_curve: TopOpeBRepDSCurve,
}
geometry_data_base!(TopOpeBRepDSCurveData);

impl TopOpeBRepDSCurveData {
    /// `TopOpeBRepDS_CurveData(C)`; `TopOpeBRepDSCurveData::new(&c)`.
    pub fn new(c: &TopOpeBRepDSCurve) -> Self {
        Self {
            base: TopOpeBRepDSGeometryData::default(),
            my_curve: c.clone(),
        }
    }
}

/// A surface with its interferences (`TopOpeBRepDS_SurfaceData`);
/// `TopOpeBRepDSSurfaceData::new(&s)`.
#[derive(Clone, Default)]
pub struct TopOpeBRepDSSurfaceData {
    base: TopOpeBRepDSGeometryData,
    pub(crate) my_surface: TopOpeBRepDSSurface,
}
geometry_data_base!(TopOpeBRepDSSurfaceData);

impl TopOpeBRepDSSurfaceData {
    /// `TopOpeBRepDS_SurfaceData(S)`; `TopOpeBRepDSSurfaceData::new(&s)`.
    pub fn new(s: &TopOpeBRepDSSurface) -> Self {
        Self {
            base: TopOpeBRepDSGeometryData::default(),
            my_surface: s.clone(),
        }
    }
}

/// The data of a shape (`TopOpeBRepDS_ShapeData`): its interferences, same domain
/// shapes, reference, orientation configuration, index and ancestor rank; kept;
/// `let d = TopOpeBRepDSShapeData::default();`.
#[derive(Clone)]
pub struct TopOpeBRepDSShapeData {
    pub(crate) my_interferences: ListOfInterference,
    pub(crate) my_same_domain: List<Shape>,
    pub(crate) my_same_domain_ref: i32,
    pub(crate) my_same_domain_ori: TopOpeBRepDSConfig,
    pub(crate) my_same_domain_ind: i32,
    pub(crate) my_orientation: Orientation,
    pub(crate) my_orientation_def: bool,
    pub(crate) my_ancestor_rank: i32,
    pub(crate) my_keep: bool,
}

impl Default for TopOpeBRepDSShapeData {
    /// `TopOpeBRepDS_ShapeData()`.
    fn default() -> Self {
        Self {
            my_interferences: List::default(),
            my_same_domain: List::default(),
            my_same_domain_ref: 0,
            my_same_domain_ori: TopOpeBRepDSConfig::UnshGeometry,
            my_same_domain_ind: 0,
            my_orientation: Orientation::Forward,
            my_orientation_def: false,
            my_ancestor_rank: 0,
            my_keep: true,
        }
    }
}

impl TopOpeBRepDSShapeData {
    /// `Interferences()`; `d.interferences()`.
    pub fn interferences(&self) -> &ListOfInterference {
        &self.my_interferences
    }

    /// `ChangeInterferences()`; `d.change_interferences().append(i)`.
    pub fn change_interferences(&mut self) -> &mut ListOfInterference {
        &mut self.my_interferences
    }

    /// `Keep()`; `d.keep()`.
    pub fn keep(&self) -> bool {
        self.my_keep
    }

    /// `ChangeKeep(B)`; `d.change_keep(false)`.
    pub fn change_keep(&mut self, b: bool) {
        self.my_keep = b;
    }
}

/// The split parts of a shape by state (`TopOpeBRepDS_ShapeWithState`): UNKNOWN and
/// not split by default; `let mut s = TopOpeBRepDSShapeWithState::default();`.
#[derive(Clone)]
pub struct TopOpeBRepDSShapeWithState {
    my_part_in: List<Shape>,
    my_part_out: List<Shape>,
    my_part_on: List<Shape>,
    my_state: State,
    my_is_splitted: bool,
}

impl Default for TopOpeBRepDSShapeWithState {
    /// `TopOpeBRepDS_ShapeWithState()`.
    fn default() -> Self {
        Self {
            my_part_in: List::default(),
            my_part_out: List::default(),
            my_part_on: List::default(),
            my_state: State::Unknown,
            my_is_splitted: false,
        }
    }
}

impl TopOpeBRepDSShapeWithState {
    /// The part of a state (none for UNKNOWN), changed in place by Builder1.
    pub(crate) fn part_mut(&mut self, a_state: State) -> Option<&mut List<Shape>> {
        match a_state {
            State::In => Some(&mut self.my_part_in),
            State::Out => Some(&mut self.my_part_out),
            State::On => Some(&mut self.my_part_on),
            State::Unknown => None,
        }
    }

    /// `Part(aState)`: the parts IN, OUT or ON, an empty list for UNKNOWN;
    /// `s.part(State::In)`.
    pub fn part(&self, a_state: State) -> List<Shape> {
        match a_state {
            State::In => self.my_part_in.clone(),
            State::Out => self.my_part_out.clone(),
            State::On => self.my_part_on.clone(),
            State::Unknown => List::default(),
        }
    }

    /// `AddPart(aShape, aState)`: appended (nothing for UNKNOWN); `s.add_part(&f, State::On)`.
    pub fn add_part(&mut self, a_shape: &Shape, a_state: State) {
        if let Some(part) = self.part_mut(a_state) {
            part.append(a_shape.clone());
        }
    }

    /// `AddParts(aListOfShape, aState)`: appended in order; `s.add_parts(&list, State::In)`.
    pub fn add_parts(&mut self, a_list_of_shape: &List<Shape>, a_state: State) {
        if let Some(part) = self.part_mut(a_state) {
            for s in a_list_of_shape.iter() {
                part.append(s.clone());
            }
        }
    }

    /// `SetState(aState)`; `s.set_state(State::In)`.
    pub fn set_state(&mut self, a_state: State) {
        self.my_state = a_state;
    }

    /// `State()`; `s.state()`.
    pub fn state(&self) -> State {
        self.my_state
    }

    /// `SetIsSplitted(anIsSplitted)`; `s.set_is_splitted(true)`.
    pub fn set_is_splitted(&mut self, an_is_splitted: bool) {
        self.my_is_splitted = an_is_splitted;
    }

    /// `IsSplitted()`; `s.is_splitted()`.
    pub fn is_splitted(&self) -> bool {
        self.my_is_splitted
    }
}

/// The shapes on one state of a shape and its split flag
/// (`TopOpeBRepDS_ListOfShapeOn1State`); `let mut l = TopOpeBRepDSListOfShapeOn1State::default();`.
#[derive(Clone, Default)]
pub struct TopOpeBRepDSListOfShapeOn1State {
    my_list: List<Shape>,
    my_splits: i32,
}

impl TopOpeBRepDSListOfShapeOn1State {
    /// `ListOnState()`; `l.list_on_state()`.
    pub fn list_on_state(&self) -> &List<Shape> {
        &self.my_list
    }

    /// `ChangeListOnState()`; `l.change_list_on_state().append(s)`.
    pub fn change_list_on_state(&mut self) -> &mut List<Shape> {
        &mut self.my_list
    }

    /// `IsSplit()`: the first bit of the split flags; `l.is_split()`.
    pub fn is_split(&self) -> bool {
        self.my_splits & 1 != 0
    }

    /// `Split(B)` (natively defaulted to true): sets or clears the first bit;
    /// `l.split(true)`.
    pub fn split(&mut self, b: bool) {
        if b {
            self.my_splits |= 1;
        } else {
            self.my_splits &= !1;
        }
    }

    /// `Clear()`: no shape, not split; `l.clear()`.
    pub fn clear(&mut self) {
        self.my_list.clear();
        self.my_splits = 0;
    }
}
