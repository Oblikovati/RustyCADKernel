// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_Curve.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_Curve`: a section curve of the data structure, with its range,
//! tolerance, the faces it lies on and the surface/curve interferences carrying its
//! p-curves.
use super::{TopOpeBRepDSInterference, TopOpeBRepDSSurfaceCurveInterference};
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::GeomCurve;

/// A curve of the data structure (`TopOpeBRepDS_Curve`); the default tolerance is
/// Precision::Confusion() and the curve is kept;
/// `let c = TopOpeBRepDSCurve::new(&curve, 1e-7, false);`.
#[derive(Clone)]
pub struct TopOpeBRepDSCurve {
    my_curve: Handle<dyn GeomCurve>,
    my_first: f64,
    my_last: f64,
    my_range_defined: bool,
    my_tolerance: f64,
    my_is_walk: bool,
    my_s1: Shape,
    my_s2: Shape,
    my_sci1: Handle<TopOpeBRepDSInterference>,
    my_sci2: Handle<TopOpeBRepDSInterference>,
    my_keep: bool,
    my_mother: i32,
    my_ds_index: i32,
    my_existing_edge: Edge,
    my_is_existing_edge_reversed: bool,
    my_equivalent_curve: i32,
    my_is_equivalent_curve_reversed: bool,
}

impl Default for TopOpeBRepDSCurve {
    /// `TopOpeBRepDS_Curve()`.
    fn default() -> Self {
        Self {
            my_curve: Handle::default(),
            my_first: 0.,
            my_last: 0.,
            my_range_defined: false,
            my_tolerance: precision::confusion(),
            my_is_walk: false,
            my_s1: Shape::default(),
            my_s2: Shape::default(),
            my_sci1: Handle::default(),
            my_sci2: Handle::default(),
            my_keep: true,
            my_mother: 0,
            my_ds_index: 0,
            my_existing_edge: Edge::default(),
            my_is_existing_edge_reversed: false,
            my_equivalent_curve: 0,
            my_is_equivalent_curve_reversed: false,
        }
    }
}

/// The p-curve of a surface/curve interference; Standard_NullObject when it is not
/// one (a null dereference natively).
fn sci_pcurve(sci: &Handle<TopOpeBRepDSInterference>) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    if sci.is_null() {
        return Ok(Handle::default());
    }
    let sc: Handle<TopOpeBRepDSSurfaceCurveInterference> = sci.clone().cast();
    Ok(sc.get().ok_or_else(not_surface_curve)?.pcurve())
}

/// Standard_NullObject of an interference that is not a surface/curve one.
fn not_surface_curve() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "TopOpeBRepDS_Curve: the interference is not a TopOpeBRepDS_SurfaceCurveInterference; expected a surface/curve interference",
        "TopOpeBRepDS_Curve",
    )
}

impl TopOpeBRepDSCurve {
    /// `TopOpeBRepDS_Curve(P, T, IsWalk)` (IsWalk natively defaulted to false);
    /// `TopOpeBRepDSCurve::new(&c, 1e-7, false)`.
    pub fn new(p: &Handle<dyn GeomCurve>, t: f64, is_walk: bool) -> Self {
        let mut c = Self::default();
        c.define_curve(p, t, is_walk);
        c
    }

    /// `DefineCurve(P, T, IsWalk)`; `c.define_curve(&curve, 1e-7, true)`.
    pub fn define_curve(&mut self, p: &Handle<dyn GeomCurve>, t: f64, is_walk: bool) {
        (self.my_curve, self.my_tolerance, self.my_is_walk) = (p.clone(), t, is_walk);
    }

    /// `Tolerance(tol)`; `c.set_tolerance(1e-6)`.
    pub fn set_tolerance(&mut self, tol: f64) {
        self.my_tolerance = tol;
    }

    /// `SetSCI(I1, I2)`; `c.set_sci(&i1, &i2)`.
    pub fn set_sci(
        &mut self,
        i1: &Handle<TopOpeBRepDSInterference>,
        i2: &Handle<TopOpeBRepDSInterference>,
    ) {
        (self.my_sci1, self.my_sci2) = (i1.clone(), i2.clone());
    }

    /// `GetSCI1()`; `c.get_sci1()`.
    pub fn get_sci1(&self) -> &Handle<TopOpeBRepDSInterference> {
        &self.my_sci1
    }

    /// `GetSCI2()`; `c.get_sci2()`.
    pub fn get_sci2(&self) -> &Handle<TopOpeBRepDSInterference> {
        &self.my_sci2
    }

    /// `GetSCI(theI1, theI2)` (deprecated natively); `let (i1, i2) = c.get_sci();`.
    pub fn get_sci(
        &self,
    ) -> (
        Handle<TopOpeBRepDSInterference>,
        Handle<TopOpeBRepDSInterference>,
    ) {
        (self.my_sci1.clone(), self.my_sci2.clone())
    }

    /// `SetShapes(S1, S2)`; `c.set_shapes(&f1, &f2)`.
    pub fn set_shapes(&mut self, s1: &Shape, s2: &Shape) {
        (self.my_s1, self.my_s2) = (s1.clone(), s2.clone());
    }

    /// `GetShapes(S1, S2)`; `let (s1, s2) = c.get_shapes();`.
    pub fn get_shapes(&self) -> (Shape, Shape) {
        (self.my_s1.clone(), self.my_s2.clone())
    }

    /// `Shape1()`; `c.shape1()`.
    pub fn shape1(&self) -> &Shape {
        &self.my_s1
    }

    /// `ChangeShape1()`; `*c.change_shape1() = f`.
    pub fn change_shape1(&mut self) -> &mut Shape {
        &mut self.my_s1
    }

    /// `Shape2()`; `c.shape2()`.
    pub fn shape2(&self) -> &Shape {
        &self.my_s2
    }

    /// `ChangeShape2()`; `*c.change_shape2() = f`.
    pub fn change_shape2(&mut self) -> &mut Shape {
        &mut self.my_s2
    }

    /// `Curve()`; `c.curve()`.
    pub fn curve(&self) -> &Handle<dyn GeomCurve> {
        &self.my_curve
    }

    /// `SetRange(First, Last)`: the range, defined; `c.set_range(0., 1.)`.
    pub fn set_range(&mut self, first: f64, last: f64) {
        (self.my_first, self.my_last, self.my_range_defined) = (first, last, true);
    }

    /// `Range(First, Last)`: the range when defined; `c.range()`.
    pub fn range(&self) -> Option<(f64, f64)> {
        self.my_range_defined
            .then_some((self.my_first, self.my_last))
    }

    /// `Tolerance()`; `c.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.my_tolerance
    }

    /// `ChangeCurve()`; `*c.change_curve() = curve`.
    pub fn change_curve(&mut self) -> &mut Handle<dyn GeomCurve> {
        &mut self.my_curve
    }

    /// `Curve(C3D, Tol)`: the curve and its tolerance; `c.set_curve(&curve, 1e-6)`.
    pub fn set_curve(&mut self, c3d: &Handle<dyn GeomCurve>, tol: f64) {
        (self.my_curve, self.my_tolerance) = (c3d.clone(), tol);
    }

    /// `Curve1()`: the p-curve of the first interference (null without one);
    /// Standard_NullObject when it is not a surface/curve interference; `c.curve1()?`.
    pub fn curve1(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        sci_pcurve(&self.my_sci1)
    }

    /// `Curve1(PC1)`: the p-curve of the first interference, when there is one;
    /// `c.set_curve1(&pc)?`.
    pub fn set_curve1(&self, pc1: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        set_sci_pcurve(&self.my_sci1, pc1)
    }

    /// `Curve2()`; `c.curve2()?`.
    pub fn curve2(&self) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        sci_pcurve(&self.my_sci2)
    }

    /// `Curve2(PC2)`; `c.set_curve2(&pc)?`.
    pub fn set_curve2(&self, pc2: &Handle<dyn Geom2dCurve>) -> Result<(), Failure> {
        set_sci_pcurve(&self.my_sci2, pc2)
    }

    /// `IsWalk()`; `c.is_walk()`.
    pub fn is_walk(&self) -> bool {
        self.my_is_walk
    }

    /// `ChangeIsWalk(B)`; `c.change_is_walk(true)`.
    pub fn change_is_walk(&mut self, b: bool) {
        self.my_is_walk = b;
    }

    /// `Keep()`; `c.keep()`.
    pub fn keep(&self) -> bool {
        self.my_keep
    }

    /// `ChangeKeep(B)`; `c.change_keep(false)`.
    pub fn change_keep(&mut self, b: bool) {
        self.my_keep = b;
    }

    /// `Mother()`; `c.mother()`.
    pub fn mother(&self) -> i32 {
        self.my_mother
    }

    /// `ChangeMother(I)`; `c.change_mother(2)`.
    pub fn change_mother(&mut self, i: i32) {
        self.my_mother = i;
    }

    /// `DSIndex()`; `c.ds_index()`.
    pub fn ds_index(&self) -> i32 {
        self.my_ds_index
    }

    /// `ChangeDSIndex(I)`; `c.change_ds_index(2)`.
    pub fn change_ds_index(&mut self, i: i32) {
        self.my_ds_index = i;
    }

    /// `SetExistingEdge(theEdge, theIsReversed)` (natively defaulted to false);
    /// `c.set_existing_edge(&edge, false)`.
    pub fn set_existing_edge(&mut self, the_edge: &Edge, the_is_reversed: bool) {
        (self.my_existing_edge, self.my_is_existing_edge_reversed) =
            (the_edge.clone(), the_is_reversed);
    }

    /// `ExistingEdge()`; `c.existing_edge()`.
    pub fn existing_edge(&self) -> &Edge {
        &self.my_existing_edge
    }

    /// `IsExistingEdgeReversed()`; `c.is_existing_edge_reversed()`.
    pub fn is_existing_edge_reversed(&self) -> bool {
        self.my_is_existing_edge_reversed
    }

    /// `SetEquivalentCurve(theCurveIndex, theIsReversed)` (natively defaulted to
    /// false); `c.set_equivalent_curve(2, true)`.
    pub fn set_equivalent_curve(&mut self, the_curve_index: i32, the_is_reversed: bool) {
        (
            self.my_equivalent_curve,
            self.my_is_equivalent_curve_reversed,
        ) = (the_curve_index, the_is_reversed);
    }

    /// `EquivalentCurve()`; `c.equivalent_curve()`.
    pub fn equivalent_curve(&self) -> i32 {
        self.my_equivalent_curve
    }

    /// `IsEquivalentCurveReversed()`; `c.is_equivalent_curve_reversed()`.
    pub fn is_equivalent_curve_reversed(&self) -> bool {
        self.my_is_equivalent_curve_reversed
    }
}

/// The p-curve of a surface/curve interference set in place (nothing for a null one).
fn set_sci_pcurve(
    sci: &Handle<TopOpeBRepDSInterference>,
    pc: &Handle<dyn Geom2dCurve>,
) -> Result<(), Failure> {
    if sci.is_null() {
        return Ok(());
    }
    let sc: Handle<TopOpeBRepDSSurfaceCurveInterference> = sci.clone().cast();
    sc.get().ok_or_else(not_surface_curve)?.set_pcurve(pc);
    Ok(())
}
