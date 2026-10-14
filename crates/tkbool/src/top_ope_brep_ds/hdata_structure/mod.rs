// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_HDataStructure.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_HDataStructure`: the data structure shared through handles by the
//! fillers and builders of the topological operations, with the queries and the
//! interference storage they use.
mod interferences;

use super::{
    TopOpeBRepDSConfig, TopOpeBRepDSCurve, TopOpeBRepDSCurveIterator, TopOpeBRepDSDataStructure,
    TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSPoint, TopOpeBRepDSPointIterator,
    TopOpeBRepDSSurface, TopOpeBRepDSSurfaceCurveInterference, TopOpeBRepDSSurfaceIterator,
    is_geometry, is_topology, kind_to_shape,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};

pub use interferences::fun_topopebrepds_sort_on_parameter;
pub(crate) use interferences::store_interference_ds;

/// The shared data structure (`TopOpeBRepDS_HDataStructure`); the data structure is
/// read through [`TopOpeBRepDSHDataStructure::ds`] and changed through
/// [`TopOpeBRepDSHDataStructure::change_ds`] (lock guards, so the accessors returning
/// references natively return copies here);
/// `let hds = Handle::new(TopOpeBRepDSHDataStructure::new());`.
#[derive(Default)]
pub struct TopOpeBRepDSHDataStructure {
    parent: StandardTransient,
    my_ds: SharedState<TopOpeBRepDSDataStructure>,
}
impl_transient!(
    TopOpeBRepDSHDataStructure,
    "TopOpeBRepDS_HDataStructure",
    StandardTransient,
    parent
);

/// `FUN_HDS_FACESINTERFER(F1, F2, HDS)`: whether a FACE/EDGE interference of `f1` has
/// `f2` as support; `fun_hds_facesinterfer(&f1, &f2, &hds)?`.
pub fn fun_hds_facesinterfer(
    f1: &Shape,
    f2: &Shape,
    hds: &Handle<TopOpeBRepDSHDataStructure>,
) -> Result<bool, Failure> {
    let h = hds.get().ok_or_else(|| {
        Failure::with_type(
            "Standard_NullObject",
            "FUN_HDS_FACESINTERFER: null data structure; expected a TopOpeBRepDS_HDataStructure",
            "TopOpeBRepDS_HDataStructure",
        )
    })?;
    let ds = h.ds();
    if !ds.has_shape(f1, true) {
        return Ok(false);
    }
    let i_f2 = ds.shape_index(f2, true);
    for i in ds.shape_interferences(f1, true).iter() {
        let Some(i) = i.get() else {
            continue;
        };
        if i.geometry_type() == TopOpeBRepDSKind::Edge
            && i.support_type() == TopOpeBRepDSKind::Face
            && i.support() == i_f2
        {
            return Ok(true);
        }
    }
    Ok(false)
}

impl TopOpeBRepDSHDataStructure {
    /// `TopOpeBRepDS_HDataStructure()`; `TopOpeBRepDSHDataStructure::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `ChkIntg()`: the integrity of the structure checked by a `TopOpeBRepDS_Check`
    /// (natively a member creating the checker on `this`; here on the handle);
    /// `TopOpeBRepDSHDataStructure::chk_intg(&hds)?`.
    pub fn chk_intg(hds: &Handle<Self>) -> Result<(), Failure> {
        super::TopOpeBRepDSCheck::with_hds(hds).chk_intg()?;
        Ok(())
    }

    /// `DS()`: the data structure, read; `hds.ds().nb_curves()`.
    pub fn ds(&self) -> RwLockReadGuard<'_, TopOpeBRepDSDataStructure> {
        self.my_ds.read()
    }

    /// `ChangeDS()`: the data structure, changed; `hds.change_ds().add_shape(&face)`.
    pub fn change_ds(&self) -> RwLockWriteGuard<'_, TopOpeBRepDSDataStructure> {
        self.my_ds.write()
    }

    /// `AddAncestors(S)`: the wires, faces, shells and solids of `s` over a known
    /// edge, wire, face or shell added with the rank of `s`; `hds.add_ancestors(&solid)?`.
    pub fn add_ancestors(&self, s: &Shape) -> Result<(), Failure> {
        self.add_ancestors_of_types(s, ShapeEnum::Wire, ShapeEnum::Edge)?;
        self.add_ancestors_of_types(s, ShapeEnum::Face, ShapeEnum::Wire)?;
        self.add_ancestors_of_types(s, ShapeEnum::Shell, ShapeEnum::Face)?;
        self.add_ancestors_of_types(s, ShapeEnum::Solid, ShapeEnum::Shell)
    }

    /// `AddAncestors(S, T1, T2)`: each sub-shape of type `t1` of `s` with a known
    /// sub-shape of type `t2` added with the rank of `s`;
    /// `hds.add_ancestors_of_types(&solid, ShapeEnum::Face, ShapeEnum::Wire)?`.
    pub fn add_ancestors_of_types(
        &self,
        s: &Shape,
        t1: ShapeEnum,
        t2: ShapeEnum,
    ) -> Result<(), Failure> {
        let mut bds = self.change_ds();
        let rank_s = bds.ancestor_rank(s);
        for st1 in Explorer::new(s, t1, ShapeEnum::Shape)? {
            for st2 in Explorer::new(&st1, t2, ShapeEnum::Shape)? {
                if bds.has_shape(&st2, true) {
                    bds.add_shape_with_rank(&st1, rank_s);
                    break;
                }
            }
        }
        Ok(())
    }

    /// `NbSurfaces()`; `hds.nb_surfaces()`.
    pub fn nb_surfaces(&self) -> i32 {
        self.ds().nb_surfaces()
    }

    /// `NbCurves()`; `hds.nb_curves()`.
    pub fn nb_curves(&self) -> i32 {
        self.ds().nb_curves()
    }

    /// `NbPoints()`; `hds.nb_points()`.
    pub fn nb_points(&self) -> i32 {
        self.ds().nb_points()
    }

    /// `Surface(I)`: a copy; `hds.surface(1)`.
    pub fn surface(&self, i: i32) -> TopOpeBRepDSSurface {
        self.ds().surface(i).clone()
    }

    /// `SurfaceCurves(I)`: the curves of the interferences of the surface;
    /// `hds.surface_curves(1)?`.
    pub fn surface_curves(&self, i: i32) -> Result<TopOpeBRepDSCurveIterator, Failure> {
        TopOpeBRepDSCurveIterator::new(self.ds().surface_interferences(i))
    }

    /// `Curve(I)`: a copy; `hds.curve(1)`.
    pub fn curve(&self, i: i32) -> TopOpeBRepDSCurve {
        self.ds().curve(i).clone()
    }

    /// `ChangeCurve(I)`: the curve changed by `f`; `hds.change_curve(1, |c| c.change_keep(false))`.
    pub fn change_curve<R>(&self, i: i32, f: impl FnOnce(&mut TopOpeBRepDSCurve) -> R) -> R {
        f(self.change_ds().change_curve(i))
    }

    /// `CurvePoints(I)`: the points of the interferences of the curve;
    /// `hds.curve_points(1)?`.
    pub fn curve_points(&self, i: i32) -> Result<TopOpeBRepDSPointIterator, Failure> {
        TopOpeBRepDSPointIterator::new(self.ds().curve_interferences(i))
    }

    /// `Point(I)`: a copy; Standard_ProgramError out of range; `hds.point(1)?`.
    pub fn point(&self, i: i32) -> Result<TopOpeBRepDSPoint, Failure> {
        Ok(*self.ds().point(i)?)
    }

    /// `NbShapes()`; `hds.nb_shapes()`.
    pub fn nb_shapes(&self) -> i32 {
        self.ds().nb_shapes()
    }

    /// `Shape(I, FindKeep)` (FindKeep natively defaulted to true): a copy;
    /// `hds.shape(1, true)`.
    pub fn shape(&self, i: i32, find_keep: bool) -> Shape {
        self.ds().shape(i, find_keep).clone()
    }

    /// `Shape(S, FindKeep)`: the index; `hds.shape_index(&face, true)`.
    pub fn shape_index(&self, s: &Shape, find_keep: bool) -> i32 {
        self.ds().shape_index(s, find_keep)
    }

    /// `HasGeometry(S)`; `hds.has_geometry(&face)`.
    pub fn has_geometry(&self, s: &Shape) -> bool {
        self.ds().has_geometry(s)
    }

    /// `HasShape(S, FindKeep)`; `hds.has_shape(&face, true)`.
    pub fn has_shape(&self, s: &Shape, find_keep: bool) -> bool {
        self.ds().has_shape(s, find_keep)
    }

    /// `HasSameDomain(S, FindKeep)`: a known shape with same domain shapes;
    /// `hds.has_same_domain(&face, true)`.
    pub fn has_same_domain(&self, s: &Shape, find_keep: bool) -> bool {
        self.has_shape(s, find_keep) && !self.ds().shape_same_domain(s).is_empty()
    }

    /// `SameDomain(S)`: a copy of the same domain shapes (an iterator natively);
    /// `hds.same_domain(&face)`.
    pub fn same_domain(&self, s: &Shape) -> List<Shape> {
        self.ds().shape_same_domain(s).clone()
    }

    /// `SameDomainOrientation(S)`: UNSHGEOMETRY for an unknown shape;
    /// `hds.same_domain_orientation(&face)`.
    pub fn same_domain_orientation(&self, s: &Shape) -> TopOpeBRepDSConfig {
        if !self.has_shape(s, true) {
            return TopOpeBRepDSConfig::UnshGeometry;
        }
        self.ds().same_domain_ori(s)
    }

    /// `SameDomainReference(S)`: 0 for an unknown shape; `hds.same_domain_reference(&face)`.
    pub fn same_domain_reference(&self, s: &Shape) -> i32 {
        if !self.has_shape(s, true) {
            return 0;
        }
        self.ds().same_domain_ref(s)
    }

    /// `SolidSurfaces(S)`; `hds.solid_surfaces(&solid)?`.
    pub fn solid_surfaces(&self, s: &Shape) -> Result<TopOpeBRepDSSurfaceIterator, Failure> {
        TopOpeBRepDSSurfaceIterator::new(self.ds().shape_interferences(s, true))
    }

    /// `SolidSurfaces(I)`; `hds.solid_surfaces_index(1)?`.
    pub fn solid_surfaces_index(&self, i: i32) -> Result<TopOpeBRepDSSurfaceIterator, Failure> {
        TopOpeBRepDSSurfaceIterator::new(self.ds().shape_interferences_index(i, true)?)
    }

    /// `FaceCurves(F)`; `hds.face_curves(&face)?`.
    pub fn face_curves(&self, f: &Shape) -> Result<TopOpeBRepDSCurveIterator, Failure> {
        TopOpeBRepDSCurveIterator::new(self.ds().shape_interferences(f, true))
    }

    /// `FaceCurves(I)`; `hds.face_curves_index(1)?`.
    pub fn face_curves_index(&self, i: i32) -> Result<TopOpeBRepDSCurveIterator, Failure> {
        TopOpeBRepDSCurveIterator::new(self.ds().shape_interferences_index(i, true)?)
    }

    /// `EdgePoints(E)`; `hds.edge_points(&edge)?`.
    pub fn edge_points(&self, e: &Shape) -> Result<TopOpeBRepDSPointIterator, Failure> {
        TopOpeBRepDSPointIterator::new(self.ds().shape_interferences(e, true))
    }

    /// `MakeCurve(C1, C2)`: `c2` added as a copy of `c1` (its mother): new surface/curve
    /// interferences copied from those of `c1` stored on its shapes, with the index of
    /// the new curve as geometry; `let i = hds.make_curve(&c1, &mut c2)?;`.
    pub fn make_curve(
        &self,
        cur_c: &TopOpeBRepDSCurve,
        new_c: &mut TopOpeBRepDSCurve,
    ) -> Result<i32, Failure> {
        let copy = |i: &Handle<TopOpeBRepDSInterference>| -> Result<Handle<TopOpeBRepDSInterference>, Failure> {
            if i.is_null() {
                return Ok(Handle::default());
            }
            Ok(Handle::new(TopOpeBRepDSSurfaceCurveInterference::from_interference(i)?).cast())
        };
        let (sci1, sci2) = (copy(cur_c.get_sci1())?, copy(cur_c.get_sci2())?);
        let (s1, s2) = (cur_c.shape1().clone(), cur_c.shape2().clone());
        let mut ds = self.change_ds();
        if !sci1.is_null() {
            ds.add_shape_interference(&s1, &sci1);
        }
        if !sci2.is_null() {
            ds.add_shape_interference(&s2, &sci2);
        }
        new_c.set_shapes(&s1, &s2);
        new_c.set_sci(&sci1, &sci2);
        new_c.change_mother(cur_c.ds_index());
        let inew_c = ds.add_curve(new_c);
        for sci in [&sci1, &sci2] {
            if let Some(i) = sci.get() {
                i.set_geometry(inew_c);
            }
        }
        Ok(inew_c)
    }

    /// `RemoveCurve(iC)`; `hds.remove_curve(1)`.
    pub fn remove_curve(&self, i_c: i32) {
        self.change_ds().remove_curve(i_c);
    }

    /// `NbGeometry(K)`: the count of points, curves or surfaces, 0 for a topology kind;
    /// `hds.nb_geometry(TopOpeBRepDSKind::Curve)`.
    pub fn nb_geometry(&self, k: TopOpeBRepDSKind) -> i32 {
        if !is_geometry(k) {
            return 0;
        }
        match k {
            TopOpeBRepDSKind::Point => self.nb_points(),
            TopOpeBRepDSKind::Curve => self.nb_curves(),
            TopOpeBRepDSKind::Surface => self.nb_surfaces(),
            _ => 0,
        }
    }

    /// `NbTopology()`: the shape count; `hds.nb_topology()`.
    pub fn nb_topology(&self) -> i32 {
        self.ds().nb_shapes()
    }

    /// `NbTopology(K)`: the count of shapes of the type of the kind (kept or not);
    /// `hds.nb_topology_of(TopOpeBRepDSKind::Face)?`.
    pub fn nb_topology_of(&self, k: TopOpeBRepDSKind) -> Result<i32, Failure> {
        if !is_topology(k) {
            return Ok(0);
        }
        let s = kind_to_shape(k);
        let ds = self.ds();
        let mut res = 0;
        for i in 1..=ds.nb_shapes() {
            if ds.keep_shape_index(i, false) && ds.shape(i, false).shape_type()? == s {
                res += 1;
            }
        }
        Ok(res)
    }

    /// `EdgesSameParameter()`: whether every kept edge of the structure is same
    /// parameter (Standard_NullObject natively for a removed shape, which the empty
    /// shape type check raises here as well); `hds.edges_same_parameter()?`.
    pub fn edges_same_parameter(&self) -> Result<bool, Failure> {
        let ds = self.ds();
        for i in 1..=ds.nb_shapes() {
            let s = ds.shape(i, true);
            if s.shape_type()? == ShapeEnum::Edge && !BRepTool::same_parameter(&topods::edge(s)?)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
