// Rust translation of the constructor and the queries of ChFi3d_Builder_1.cxx and of
// ChFi3d_Builder::Reset and ChFi3d_Builder::Generated (ChFi3d_Builder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The construction of the builder, its settings and the queries on its contours and
//! results.
use super::ChFi3dBuilder;
use crate::chfi_ds::{ErrorStatus, Map, SpineHandle, StripeHandle, StripeMap, lock};
use rustycad_tkbool::top_ope_brep_build::TopOpeBRepBuildHBuilder;
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSBuildTool, TopOpeBRepDSDataStructure, TopOpeBRepDSHDataStructure,
};
use rustycad_tkbool::top_ope_brep_tool::{TopOpeBRepToolGeomTool, TopOpeBRepToolOutCurveType};
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::ncollection::{DataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;

/// `mkbuildtool()`: the build tool of the data structure, B-splines of degree 1 for the
/// 3d curves, without overwriting nor translating the curves.
fn mkbuildtool() -> TopOpeBRepDSBuildTool {
    let gt2 = TopOpeBRepToolGeomTool::new(TopOpeBRepToolOutCurveType::BSpline1, true, false, false);
    let mut bt = TopOpeBRepDSBuildTool::with_geom_tool(&gt2);
    bt.set_over_write(false);
    bt.set_translate(false);
    bt
}

/// The ancestor map of `kind` sub-shapes of `s` to their `ancestor` shapes.
fn filled(s: &Shape, kind: ShapeEnum, ancestor: ShapeEnum) -> Result<Map, Failure> {
    let mut map = Map::new();
    map.fill(s, kind, ancestor)?;
    Ok(map)
}

impl ChFi3dBuilder {
    /// `ChFi3d_Builder(S, Ta)`: the builder of the fillets of `s` with the angular
    /// tolerance `ta`; `ChFi3dBuilder::new(&solid, 1e-2)?`.
    pub fn new(s: &Shape, ta: f64) -> Result<Self, Failure> {
        let mut b = Self {
            tolappangle: 0.,
            tolesp: 0.,
            tol2d: 0.,
            tolapp3d: 0.,
            tolapp2d: 0.,
            fleche: 0.,
            my_conti: Continuity::C1,
            my_ef_map: filled(s, ShapeEnum::Edge, ShapeEnum::Face)?,
            my_eso_map: filled(s, ShapeEnum::Edge, ShapeEnum::Solid)?,
            my_esh_map: filled(s, ShapeEnum::Edge, ShapeEnum::Shell)?,
            my_vf_map: filled(s, ShapeEnum::Vertex, ShapeEnum::Face)?,
            my_ve_map: filled(s, ShapeEnum::Vertex, ShapeEnum::Edge)?,
            ds: TopOpeBRepDSDataStructure::new(),
            my_ds: Handle::new(TopOpeBRepDSHDataStructure::new()),
            ds_moved: false,
            my_coup: Handle::new(TopOpeBRepBuildHBuilder::new(&mkbuildtool())),
            my_list_stripe: Vec::new(),
            my_vdata_map: StripeMap::new(),
            my_regul: Vec::new(),
            badstripes: Vec::new(),
            badvertices: Vec::new(),
            my_evi_map: DataMap::default(),
            my_edge_first_face: DataMap::default(),
            done: false,
            hasresult: false,
            my_shape: s.clone(),
            angular: 0.,
            my_generated: List::default(),
            my_shape_result: Shape::default(),
            bad_shape: Shape::default(),
        };
        b.set_params(ta, 1.0e-4, 1.0e-5, 1.0e-4, 1.0e-5, 1.0e-3);
        b.set_continuity(Continuity::C1, ta);
        Ok(b)
    }

    /// `SetParams(Tang, Tesp, T2d, TApp3d, TolApp2d, Fleche)`: the angular, 3d, 2d,
    /// approximation tolerances and the deflection;
    /// `b.set_params(1e-2, 1e-4, 1e-5, 1e-4, 1e-5, 1e-3)`.
    pub fn set_params(
        &mut self,
        tang: f64,
        tesp: f64,
        t2d: f64,
        tapp3d: f64,
        tolapp2d: f64,
        fleche: f64,
    ) {
        self.angular = tang;
        self.tolesp = tesp;
        self.tol2d = t2d;
        self.tolapp3d = tapp3d;
        self.tolapp2d = tolapp2d;
        self.fleche = fleche;
    }

    /// `SetContinuity(InternalContinuity, AngularTolerance)`;
    /// `b.set_continuity(Continuity::C1, 1e-2)`.
    pub fn set_continuity(&mut self, internal_continuity: Continuity, angular_tolerance: f64) {
        self.my_conti = internal_continuity;
        self.tolappangle = angular_tolerance;
    }

    /// `IsDone()`; `b.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `Shape()`: the result (Standard_NoSuchObject when not done); `b.shape()?`.
    pub fn shape(&self) -> Result<Shape, Failure> {
        if !self.done {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "ChFi3d_Builder::Shape() - no result",
            ));
        }
        Ok(self.my_shape_result.clone())
    }

    /// `NbFaultyContours()`; `b.nb_faulty_contours()`.
    pub fn nb_faulty_contours(&self) -> i32 {
        self.badstripes.len() as i32
    }

    /// `FaultyContour(I)`: the index of the `i`-th faulty contour among the contours (0
    /// when out of range); `b.faulty_contour(1)`.
    pub fn faulty_contour(&self, i: i32) -> i32 {
        let Some(st) = usize::try_from(i - 1)
            .ok()
            .and_then(|k| self.badstripes.get(k))
        else {
            return 0;
        };
        self.my_list_stripe
            .iter()
            .position(|s| std::sync::Arc::ptr_eq(s, st))
            .map_or(0, |k| k as i32 + 1)
    }

    /// The `ic`-th contour (1-based), if any.
    pub(crate) fn stripe_at(&self, ic: i32) -> Option<&StripeHandle> {
        usize::try_from(ic - 1)
            .ok()
            .and_then(|k| self.my_list_stripe.get(k))
    }

    /// `NbComputedSurfaces(IC)`: the number of surface data of the contour `ic`;
    /// `b.nb_computed_surfaces(1)`.
    pub fn nb_computed_surfaces(&self, ic: i32) -> i32 {
        let Some(st) = self.stripe_at(ic) else {
            return 0;
        };
        let st = lock(st);
        if st.spine().is_none() {
            return 0;
        }
        st.set_of_surf_data().map_or(0, |hd| lock(hd).len() as i32)
    }

    /// `ComputedSurface(IC, IS)`: the surface of the `is`-th surface data of the contour
    /// `ic`; `b.computed_surface(1, 1)?`.
    pub fn computed_surface(&self, ic: i32, is: i32) -> Result<Handle<dyn GeomSurface>, Failure> {
        let st = self
            .stripe_at(ic)
            .ok_or_else(|| null_stripe("ComputedSurface"))?;
        let hd = lock(st)
            .set_of_surf_data()
            .cloned()
            .ok_or_else(|| null_stripe("ComputedSurface"))?;
        let sd = usize::try_from(is - 1)
            .ok()
            .and_then(|k| lock(&hd).get(k).cloned())
            .ok_or_else(|| Failure::out_of_range("NCollection_Sequence::Value"))?;
        let isurf = lock(&sd).surf();
        Ok(self.my_ds.surface(isurf).surface().clone())
    }

    /// `NbFaultyVertices()`; `b.nb_faulty_vertices()`.
    pub fn nb_faulty_vertices(&self) -> i32 {
        self.badvertices.len() as i32
    }

    /// `FaultyVertex(IV)`: the `iv`-th faulty vertex (null when out of range);
    /// `b.faulty_vertex(1)?`.
    pub fn faulty_vertex(&self, iv: i32) -> Result<Vertex, Failure> {
        match usize::try_from(iv - 1)
            .ok()
            .and_then(|k| self.badvertices.get(k))
        {
            Some(v) => topods::vertex(v),
            None => Ok(Vertex::default()),
        }
    }

    /// `HasResult()`; `b.has_result()`.
    pub fn has_result(&self) -> bool {
        self.hasresult
    }

    /// `BadShape()`: the partial result (Standard_NoSuchObject without one);
    /// `b.bad_shape()?`.
    pub fn bad_shape(&self) -> Result<Shape, Failure> {
        if !self.hasresult {
            return Err(Failure::with_message(
                "Standard_NoSuchObject",
                "ChFi3d_Builder::BadShape() - no result",
            ));
        }
        Ok(self.bad_shape.clone())
    }

    /// `StripeStatus(IC)`: the error status of the contour `ic`; `b.stripe_status(1)?`.
    pub fn stripe_status(&self, ic: i32) -> Result<ErrorStatus, Failure> {
        let st = self
            .stripe_at(ic)
            .ok_or_else(|| null_stripe("StripeStatus"))?;
        let sp = lock(st)
            .spine()
            .cloned()
            .ok_or_else(|| null_stripe("StripeStatus"))?;
        Ok(lock(&sp).error_status())
    }

    /// `Builder()`: the builder of the result; `b.builder()`.
    pub fn builder(&self) -> Handle<TopOpeBRepBuildHBuilder> {
        self.my_coup.clone()
    }

    /// `Remove(E)`: removes the contour containing `e`; `b.remove(&edge)?`.
    pub fn remove(&mut self, e: &Edge) -> Result<(), Failure> {
        for k in 0..self.my_list_stripe.len() {
            let sp = lock(&self.my_list_stripe[k])
                .spine()
                .cloned()
                .ok_or_else(|| null_stripe("Remove"))?;
            let sp = lock(&sp);
            for j in 1..=sp.nb_edges() {
                if e.is_same(sp.edges(j)?) {
                    drop(sp);
                    self.my_list_stripe.remove(k);
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// `Value(I)`: the spine of the `i`-th contour (Standard_NoSuchObject out of range,
    /// `None` for a contour without spine); `b.value(1)?`.
    pub fn value(&self, i: i32) -> Result<Option<SpineHandle>, Failure> {
        let st = self.stripe_at(i.max(1)).ok_or_else(|| {
            Failure::with_message("Standard_NoSuchObject", "NCollection_List::Iterator::Value")
        })?;
        Ok(lock(st).spine().cloned())
    }

    /// `NbElements()`: the number of contours up to the first one without spine;
    /// `b.nb_elements()`.
    pub fn nb_elements(&self) -> i32 {
        self.my_list_stripe
            .iter()
            .take_while(|s| lock(s).spine().is_some())
            .count() as i32
    }

    /// `Contains(E, IndexInSpine)`: the contour containing `e` with the index of `e` in its
    /// spine ((0, 0) when none); `b.contains_with_index(&edge)?`.
    pub fn contains_with_index(&self, e: &Edge) -> Result<(i32, i32), Failure> {
        for (i, st) in (1..).zip(&self.my_list_stripe) {
            let Some(sp) = lock(st).spine().cloned() else {
                break;
            };
            let sp = lock(&sp);
            for j in 1..=sp.nb_edges() {
                if e.is_same(sp.edges(j)?) {
                    return Ok((i, j));
                }
            }
        }
        Ok((0, 0))
    }

    /// `Contains(E)`: the index of the contour containing `e` (0 when none);
    /// `b.contains(&edge)?`.
    pub fn contains(&self, e: &Edge) -> Result<i32, Failure> {
        Ok(self.contains_with_index(e)?.0)
    }

    /// The spine of the contour `ic` when `ic <= NbElements()`.
    fn element(&self, ic: i32) -> Result<Option<SpineHandle>, Failure> {
        if ic <= self.nb_elements() {
            return self.value(ic);
        }
        Ok(None)
    }

    /// `Length(IC)`: the length of the contour `ic` (-1 when out of range);
    /// `b.length(1)?`.
    pub fn length(&self, ic: i32) -> Result<f64, Failure> {
        match self.element(ic)? {
            Some(sp) => {
                let sp = lock(&sp);
                sp.last_parameter_of(sp.nb_edges())
            }
            None => Ok(-1.),
        }
    }

    /// `FirstVertex(IC)`: the first vertex of the contour `ic` (null when out of range);
    /// `b.first_vertex(1)?`.
    pub fn first_vertex(&self, ic: i32) -> Result<Vertex, Failure> {
        match self.element(ic)? {
            Some(sp) => lock(&sp).first_vertex(),
            None => Ok(Vertex::default()),
        }
    }

    /// `LastVertex(IC)`: the last vertex of the contour `ic` (null when out of range);
    /// `b.last_vertex(1)?`.
    pub fn last_vertex(&self, ic: i32) -> Result<Vertex, Failure> {
        match self.element(ic)? {
            Some(sp) => lock(&sp).last_vertex(),
            None => Ok(Vertex::default()),
        }
    }

    /// `Abscissa(IC, V)`: the abscissa of `v` on the contour `ic` (-1 when out of range);
    /// `b.abscissa(1, &v)?`.
    pub fn abscissa(&self, ic: i32, v: &Vertex) -> Result<f64, Failure> {
        match self.element(ic)? {
            Some(sp) => lock(&sp).absc_of_vertex(v),
            None => Ok(-1.),
        }
    }

    /// `RelativeAbscissa(IC, V)`: the abscissa of `v` over the length of the contour
    /// (-1 when out of range); `b.relative_abscissa(1, &v)?`.
    pub fn relative_abscissa(&self, ic: i32, v: &Vertex) -> Result<f64, Failure> {
        if ic <= self.nb_elements() {
            return Ok(self.abscissa(ic, v)? / self.length(ic)?);
        }
        Ok(-1.)
    }

    /// `Closed(IC)`; `b.closed(1)?`.
    pub fn closed(&self, ic: i32) -> Result<bool, Failure> {
        match self.element(ic)? {
            Some(sp) => lock(&sp).is_closed(),
            None => Ok(false),
        }
    }

    /// `ClosedAndTangent(IC)`; `b.closed_and_tangent(1)?`.
    pub fn closed_and_tangent(&self, ic: i32) -> Result<bool, Failure> {
        Ok(self.element(ic)?.is_some_and(|sp| lock(&sp).is_periodic()))
    }

    /// `Reset()`: forgets the results and the corners, and the contours without spine;
    /// `b.reset()?`.
    pub fn reset(&mut self) -> Result<(), Failure> {
        self.done = false;
        self.my_vdata_map.clear();
        self.my_regul.clear();
        self.my_evi_map.clear(true);
        self.badstripes.clear();
        self.badvertices.clear();
        let mut kept = Vec::with_capacity(self.my_list_stripe.len());
        for st in self.my_list_stripe.drain(..) {
            if lock(&st).spine().is_some() {
                lock(&st).reset()?;
                kept.push(st);
            }
        }
        self.my_list_stripe = kept;
        Ok(())
    }

    /// `Generated(EouV)`: the faces generated from an edge or a vertex; `b.generated(&e)?`.
    pub fn generated(&mut self, eou_v: &Shape) -> Result<&List<Shape>, Failure> {
        self.my_generated.clear();
        if eou_v.is_null() || !matches!(eou_v.shape_type()?, ShapeEnum::Edge | ShapeEnum::Vertex) {
            return Ok(&self.my_generated);
        }
        if let Some(l) = self.my_evi_map.seek(eou_v) {
            for &i in l {
                for f in self.my_coup.new_faces(i)?.iter() {
                    self.my_generated.append(f.clone());
                }
            }
        }
        Ok(&self.my_generated)
    }
}

/// The failure of a contour without spine or surface data.
fn null_stripe(what: &str) -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        &format!("ChFi3d_Builder::{what}: null handle"),
    )
}
