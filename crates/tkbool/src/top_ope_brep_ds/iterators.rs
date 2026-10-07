// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_InterferenceIterator,
// TopOpeBRepDS_CurveIterator, TopOpeBRepDS_PointIterator and TopOpeBRepDS_SurfaceIterator
// (.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Iterators over interference lists: all the interferences matching the geometry
//! and support filters, or those on curves, points (and vertices) or surfaces.
use super::{
    ListOfInterference, TopOpeBRepDSConfig, TopOpeBRepDSCurvePointInterference,
    TopOpeBRepDSEdgeVertexInterference, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    TopOpeBRepDSSurfaceCurveInterference,
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::{Orientation, State};
use std::ops::{Deref, DerefMut};

/// The `MatchInterference` override of an iterator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum MatchKind {
    /// `TopOpeBRepDS_InterferenceIterator`: the filters.
    #[default]
    Filters,
    /// `TopOpeBRepDS_CurveIterator`: the curve interferences.
    Curve,
    /// `TopOpeBRepDS_PointIterator`: the point and vertex interferences.
    Point,
}

/// An iterator over the interferences of a list matching the filters
/// (`TopOpeBRepDS_InterferenceIterator`); it walks a copy of the list of handles;
/// `let mut it = TopOpeBRepDSInterferenceIterator::with_list(&l)?;`.
#[derive(Clone, Default)]
pub struct TopOpeBRepDSInterferenceIterator {
    my_items: Vec<Handle<TopOpeBRepDSInterference>>,
    my_position: usize,
    my_gk: Option<TopOpeBRepDSKind>,
    my_g: Option<i32>,
    my_sk: Option<TopOpeBRepDSKind>,
    my_s: Option<i32>,
    kind: MatchKind,
}

/// Standard_NullObject of a null interference in a list.
fn null_item() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "TopOpeBRepDS_InterferenceIterator: null interference; expected an interference handle",
        "TopOpeBRepDS_InterferenceIterator",
    )
}

impl TopOpeBRepDSInterferenceIterator {
    /// `TopOpeBRepDS_InterferenceIterator()`: no list, no filter;
    /// `TopOpeBRepDSInterferenceIterator::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `TopOpeBRepDS_InterferenceIterator(L)`: `Init(L)`;
    /// `TopOpeBRepDSInterferenceIterator::with_list(&l)?`.
    pub fn with_list(l: &ListOfInterference) -> Result<Self, Failure> {
        let mut it = Self::new();
        it.init(l)?;
        Ok(it)
    }

    /// An iterator of a derived kind: constructed as the base (whose constructor
    /// matches with the base filters, as a virtual call in a C++ constructor does).
    fn derived(l: &ListOfInterference, kind: MatchKind) -> Result<Self, Failure> {
        let mut it = Self::with_list(l)?;
        it.kind = kind;
        Ok(it)
    }

    /// `Init(L)`: at the first matching interference; `it.init(&l)?`.
    pub fn init(&mut self, l: &ListOfInterference) -> Result<(), Failure> {
        self.my_items = l.iter().cloned().collect();
        self.my_position = 0;
        self.match_()
    }

    /// `GeometryKind(GK)`; `it.geometry_kind(TopOpeBRepDSKind::Curve)`.
    pub fn geometry_kind(&mut self, gk: TopOpeBRepDSKind) {
        self.my_gk = Some(gk);
    }

    /// `Geometry(G)`; `it.geometry(2)`.
    pub fn geometry(&mut self, g: i32) {
        self.my_g = Some(g);
    }

    /// `SupportKind(ST)`; `it.support_kind(TopOpeBRepDSKind::Face)`.
    pub fn support_kind(&mut self, st: TopOpeBRepDSKind) {
        self.my_sk = Some(st);
    }

    /// `Support(S)`; `it.support(1)`.
    pub fn support(&mut self, s: i32) {
        self.my_s = Some(s);
    }

    /// `Match()`: moves to the next matching interference (the current included);
    /// `it.match_()?`.
    pub fn match_(&mut self) -> Result<(), Failure> {
        while let Some(i) = self.my_items.get(self.my_position) {
            if self.match_interference(i)? {
                break;
            }
            self.my_position += 1;
        }
        Ok(())
    }

    /// `MatchInterference(I)` (virtual natively): the filters, or the curve or point
    /// kinds of the derived iterators; `it.match_interference(&i)?`.
    pub fn match_interference(
        &self,
        i: &Handle<TopOpeBRepDSInterference>,
    ) -> Result<bool, Failure> {
        let i = i.get().ok_or_else(null_item)?;
        let gt = i.geometry_type();
        Ok(match self.kind {
            MatchKind::Curve => gt == TopOpeBRepDSKind::Curve,
            MatchKind::Point => gt == TopOpeBRepDSKind::Point || gt == TopOpeBRepDSKind::Vertex,
            MatchKind::Filters => {
                self.my_gk.is_none_or(|k| gt == k)
                    && self.my_sk.is_none_or(|k| i.support_type() == k)
                    && self.my_g.is_none_or(|g| i.geometry() == g)
                    && self.my_s.is_none_or(|s| i.support() == s)
            }
        })
    }

    /// `More()`; `it.more()`.
    pub fn more(&self) -> bool {
        self.my_position < self.my_items.len()
    }

    /// `Next()`: the next matching interference; `it.next()?`.
    #[allow(clippy::should_implement_trait)] // OCCT's More/Next protocol, fallible.
    pub fn next(&mut self) -> Result<(), Failure> {
        if self.more() {
            self.my_position += 1;
            self.match_()?;
        }
        Ok(())
    }

    /// `Value()`: the current interference; Standard_NoSuchObject past the end;
    /// `it.value()?`.
    pub fn value(&self) -> Result<&Handle<TopOpeBRepDSInterference>, Failure> {
        self.my_items.get(self.my_position).ok_or_else(|| Failure::with_type("Standard_NoSuchObject", "TopOpeBRepDS_InterferenceIterator::Value: no current interference; expected More()", "TopOpeBRepDS_InterferenceIterator"))
    }

    /// `ChangeIterator()`: the position of the list iterator (0-based in the copy of
    /// the list); `*it.change_iterator() += 1`.
    pub fn change_iterator(&mut self) -> &mut usize {
        &mut self.my_position
    }

    /// The current interference, dereferenced.
    fn current(&self) -> Result<&TopOpeBRepDSInterference, Failure> {
        self.value()?.get().ok_or_else(null_item)
    }
}

/// Implements the base iterator of a derived iterator.
macro_rules! derived_iterator {
    ($it:ty) => {
        impl Deref for $it {
            type Target = TopOpeBRepDSInterferenceIterator;
            fn deref(&self) -> &TopOpeBRepDSInterferenceIterator {
                &self.base
            }
        }
        impl DerefMut for $it {
            fn deref_mut(&mut self) -> &mut TopOpeBRepDSInterferenceIterator {
                &mut self.base
            }
        }
        impl $it {
            /// `Current()`: the geometry index of the current interference; `it.current()?`.
            pub fn current(&self) -> Result<i32, Failure> {
                Ok(self.base.current()?.geometry())
            }

            /// `Orientation(S)`: the orientation of the current transition seen from `s`;
            /// `it.orientation(State::In)?`.
            pub fn orientation(&self, s: State) -> Result<Orientation, Failure> {
                Ok(self
                    .base
                    .current()?
                    .transition()
                    .orientation(s, rustycad_tkg3d::top_abs::ShapeEnum::Face))
            }
        }
    };
}

/// An iterator over the curve interferences of a list (`TopOpeBRepDS_CurveIterator`);
/// `let it = TopOpeBRepDSCurveIterator::new(&l)?;`.
#[derive(Clone)]
pub struct TopOpeBRepDSCurveIterator {
    base: TopOpeBRepDSInterferenceIterator,
}
derived_iterator!(TopOpeBRepDSCurveIterator);

impl TopOpeBRepDSCurveIterator {
    /// `TopOpeBRepDS_CurveIterator(L)`: at the first curve interference;
    /// `TopOpeBRepDSCurveIterator::new(&l)?`.
    pub fn new(l: &ListOfInterference) -> Result<Self, Failure> {
        let mut base = TopOpeBRepDSInterferenceIterator::derived(l, MatchKind::Curve)?;
        base.match_()?;
        Ok(Self { base })
    }

    /// `PCurve()`: the p-curve of the current surface/curve interference (natively an
    /// unchecked cast; Standard_NullObject for another interference here);
    /// `it.pcurve()?`.
    pub fn pcurve(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let sci: Handle<TopOpeBRepDSSurfaceCurveInterference> = self.base.value()?.clone().cast();
        let sc = sci.get().ok_or_else(|| Failure::with_type("Standard_NullObject", "TopOpeBRepDS_CurveIterator::PCurve: not a surface/curve interference; expected a TopOpeBRepDS_SurfaceCurveInterference", "TopOpeBRepDS_CurveIterator"))?;
        Ok(sc.pcurve())
    }
}

/// An iterator over the point and vertex interferences of a list
/// (`TopOpeBRepDS_PointIterator`); `let it = TopOpeBRepDSPointIterator::new(&l)?;`.
#[derive(Clone)]
pub struct TopOpeBRepDSPointIterator {
    base: TopOpeBRepDSInterferenceIterator,
}
derived_iterator!(TopOpeBRepDSPointIterator);

/// Standard_ProgramError of a point iterator member.
fn point_error(member: &str) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!(
            "TopOpeBRepDS_PointIterator::{member}(): the interference has no such data; expected a curve/point or edge/vertex interference"
        ),
        "TopOpeBRepDS_PointIterator",
    )
}

impl TopOpeBRepDSPointIterator {
    /// `TopOpeBRepDS_PointIterator(L)`: at the first point or vertex interference;
    /// `TopOpeBRepDSPointIterator::new(&l)?`.
    pub fn new(l: &ListOfInterference) -> Result<Self, Failure> {
        let mut base = TopOpeBRepDSInterferenceIterator::derived(l, MatchKind::Point)?;
        base.match_()?;
        Ok(Self { base })
    }

    /// `Parameter()`: the parameter of a curve/point or edge/vertex interference;
    /// Standard_ProgramError for another one; `it.parameter()?`.
    pub fn parameter(&self) -> Result<f64, Failure> {
        let i = self.base.value()?;
        let cp: Handle<TopOpeBRepDSCurvePointInterference> = i.clone().cast();
        if let Some(cp) = cp.get() {
            return Ok(cp.parameter());
        }
        let ev: Handle<TopOpeBRepDSEdgeVertexInterference> = i.clone().cast();
        ev.get()
            .map(|ev| ev.parameter())
            .ok_or_else(|| point_error("Parameter"))
    }

    /// `IsVertex()`; `it.is_vertex()?`.
    pub fn is_vertex(&self) -> Result<bool, Failure> {
        Ok(self.base.current()?.geometry_type() == TopOpeBRepDSKind::Vertex)
    }

    /// `IsPoint()`; `it.is_point()?`.
    pub fn is_point(&self) -> Result<bool, Failure> {
        Ok(self.base.current()?.geometry_type() == TopOpeBRepDSKind::Point)
    }

    /// The configuration of the current edge/vertex interference.
    fn config(&self, member: &str) -> Result<TopOpeBRepDSConfig, Failure> {
        let ev: Handle<TopOpeBRepDSEdgeVertexInterference> = self.base.value()?.clone().cast();
        ev.get()
            .map(|ev| ev.config())
            .ok_or_else(|| point_error(member))
    }

    /// `DiffOriented()`: an edge/vertex interference of DIFFORIENTED configuration;
    /// Standard_ProgramError for another interference; `it.diff_oriented()?`.
    pub fn diff_oriented(&self) -> Result<bool, Failure> {
        Ok(self.config("DiffOriented")? == TopOpeBRepDSConfig::DiffOriented)
    }

    /// `SameOriented()`; `it.same_oriented()?`.
    pub fn same_oriented(&self) -> Result<bool, Failure> {
        Ok(self.config("SameOriented")? == TopOpeBRepDSConfig::SameOriented)
    }

    /// `Support()`: the support index of the current interference; `it.support_index()?`.
    pub fn support_index(&self) -> Result<i32, Failure> {
        Ok(self.base.current()?.support())
    }
}

/// An iterator over the surface interferences of a list
/// (`TopOpeBRepDS_SurfaceIterator`); natively the surface filter is set after the
/// first match, so the first interference is current whatever its geometry;
/// `let it = TopOpeBRepDSSurfaceIterator::new(&l)?;`.
#[derive(Clone)]
pub struct TopOpeBRepDSSurfaceIterator {
    base: TopOpeBRepDSInterferenceIterator,
}
derived_iterator!(TopOpeBRepDSSurfaceIterator);

impl TopOpeBRepDSSurfaceIterator {
    /// `TopOpeBRepDS_SurfaceIterator(L)`; `TopOpeBRepDSSurfaceIterator::new(&l)?`.
    pub fn new(l: &ListOfInterference) -> Result<Self, Failure> {
        let mut base = TopOpeBRepDSInterferenceIterator::with_list(l)?;
        base.geometry_kind(TopOpeBRepDSKind::Surface);
        Ok(Self { base })
    }
}
