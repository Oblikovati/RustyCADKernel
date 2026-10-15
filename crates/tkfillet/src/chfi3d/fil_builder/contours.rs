// Rust translation of the contour members of ChFi3d_FilBuilder (ChFi3d_FilBuilder.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The contours of the fillet builder and their radii: constant, by edge, by vertex,
//! by `(parameter, radius)` pairs or by laws; and the simulation of a contour.
use super::{ChFi3dFilBuilder, not_fillet_spine};
use crate::chfi_ds::{CircSection, FilSpine, SpineHandle, Stripe, lock, shared};
use crate::chfi3d::builder::ChFi3dBuilderOps;
use rustycad_tkbrep::topods::{Edge, Face, Vertex};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::law::LawFunction;
use rustycad_tkmath::gp::Xy;

impl ChFi3dFilBuilder {
    /// The fillet spine of the contour `ic` when `ic <= NbElements()`, edited by `f`
    /// (Standard_ConstructionError when it is not a fillet spine); None out of range.
    fn with_fil_spine<R>(
        &self,
        ic: i32,
        f: impl FnOnce(&mut FilSpine) -> Result<R, Failure>,
    ) -> Result<Option<R>, Failure> {
        if ic > self.base.nb_elements() {
            return Ok(None);
        }
        let sp: SpineHandle = self
            .base
            .value(ic)?
            .ok_or_else(|| not_fillet_spine("ChFi3d_FilBuilder: contour without spine"))?;
        let mut g = lock(&sp);
        let fsp = g
            .as_fil_spine_mut()
            .ok_or_else(|| not_fillet_spine("ChFi3d_FilBuilder: not a fillet spine"))?;
        Ok(Some(f(fsp)?))
    }

    /// `Add(E)`: a contour of the tangent edges through `e`, when `e` is an edge of the
    /// shape not yet in a contour; `b.add(&edge)?`.
    pub fn add(&mut self, e: &Edge) -> Result<(), Failure> {
        if self.base.contains(e)? != 0 || !self.base.my_ef_map.contains(e.as_ref()) {
            return Ok(());
        }
        let spine: SpineHandle = shared(FilSpine::with_tolerance(self.base.tolesp));
        let mut stripe = Stripe::new();
        *stripe.change_spine() = Some(spine.clone());
        let mut e_wnt = e.clone();
        e_wnt.set_orientation(Orientation::Forward);
        lock(&spine).set_edges(&e_wnt);
        if self.base.perform_element(&spine, -1., &Face::default())? {
            self.base.perform_extremity(&spine)?;
            lock(&spine).load()?;
            self.base.my_list_stripe.push(shared(stripe));
        }
        Ok(())
    }

    /// `Add(Radius, E)`: the contour through `e` with the constant radius `radius` on
    /// `e`; `b.add_with_radius(2.0, &edge)?`.
    pub fn add_with_radius(&mut self, radius: f64, e: &Edge) -> Result<(), Failure> {
        self.add(e)?;
        let ic = self.base.contains(e)?;
        if ic != 0 {
            self.set_radius_on_edge(radius, ic, e)?;
        }
        Ok(())
    }

    /// `SetRadius(C, IC, IinC)`: the law of evolution of the radius of the contour `ic`;
    /// `b.set_radius_law(&law, 1, 1)?`.
    pub fn set_radius_law(
        &mut self,
        c: &Handle<dyn LawFunction>,
        ic: i32,
        iinc: i32,
    ) -> Result<(), Failure> {
        self.with_fil_spine(ic, |f| {
            f.set_radius_law(c, iinc);
            Ok(())
        })?;
        Ok(())
    }

    /// `IsConstant(IC)`: the radius of the contour is constant (false out of range);
    /// `b.is_constant(1)?`.
    pub fn is_constant(&self, ic: i32) -> Result<bool, Failure> {
        Ok(self
            .with_fil_spine(ic, |f| Ok(f.is_constant()))?
            .unwrap_or(false))
    }

    /// `Radius(IC)`: the constant radius of the contour (-1 out of range);
    /// `b.radius(1)?`.
    pub fn radius(&self, ic: i32) -> Result<f64, Failure> {
        Ok(self.with_fil_spine(ic, |f| f.radius())?.unwrap_or(-1.))
    }

    /// `ResetContour(IC)`: the radii of the contour forgotten; `b.reset_contour(1)?`.
    pub fn reset_contour(&mut self, ic: i32) -> Result<(), Failure> {
        self.with_fil_spine(ic, |f| crate::chfi_ds::SpineObject::reset(f, true))?;
        Ok(())
    }

    /// `SetRadius(Radius, IC, E)`: a constant radius on the edge `e` of the contour;
    /// `b.set_radius_on_edge(2.0, 1, &edge)?`.
    pub fn set_radius_on_edge(&mut self, radius: f64, ic: i32, e: &Edge) -> Result<(), Failure> {
        self.with_fil_spine(ic, |f| f.set_radius_on_edge(radius, e))?;
        Ok(())
    }

    /// `UnSet(IC, E)`: the radius of the edge `e` of the contour forgotten;
    /// `b.unset_on_edge(1, &edge)?`.
    pub fn unset_on_edge(&mut self, ic: i32, e: &Edge) -> Result<(), Failure> {
        self.with_fil_spine(ic, |f| f.unset_radius_on_edge(e))?;
        Ok(())
    }

    /// `SetRadius(Radius, IC, V)`: the radius at the vertex `v` of the contour;
    /// `b.set_radius_on_vertex(2.0, 1, &v)?`.
    pub fn set_radius_on_vertex(
        &mut self,
        radius: f64,
        ic: i32,
        v: &Vertex,
    ) -> Result<(), Failure> {
        self.with_fil_spine(ic, |f| f.set_radius_on_vertex(radius, v))?;
        Ok(())
    }

    /// `UnSet(IC, V)`: the radius at the vertex `v` of the contour forgotten;
    /// `b.unset_on_vertex(1, &v)?`.
    pub fn unset_on_vertex(&mut self, ic: i32, v: &Vertex) -> Result<(), Failure> {
        self.with_fil_spine(ic, |f| f.unset_radius_on_vertex(v))?;
        Ok(())
    }

    /// `SetRadius(UandR, IC, IinC)`: the radius at a relative parameter of the edge
    /// `iinc` of the contour; `b.set_radius_at(Xy::new(0.5, 2.0), 1, 1)?`.
    pub fn set_radius_at(&mut self, u_and_r: Xy, ic: i32, iinc: i32) -> Result<(), Failure> {
        self.with_fil_spine(ic, |f| f.set_radius_at(u_and_r, iinc))?;
        Ok(())
    }

    /// `IsConstant(IC, E)`: the radius on the edge `e` of the contour is constant (false
    /// out of range); `b.is_constant_on_edge(1, &edge)?`.
    pub fn is_constant_on_edge(&self, ic: i32, e: &Edge) -> Result<bool, Failure> {
        let r = self.with_fil_spine(ic, |f| f.is_constant_on_edge(f.index_of_edge(e)))?;
        Ok(r.unwrap_or(false))
    }

    /// `Radius(IC, E)`: the constant radius on the edge `e` (-1 out of range);
    /// `b.radius_on_edge(1, &edge)?`.
    pub fn radius_on_edge(&self, ic: i32, e: &Edge) -> Result<f64, Failure> {
        Ok(self
            .with_fil_spine(ic, |f| f.radius_on_edge(e))?
            .unwrap_or(-1.))
    }

    /// `GetBounds(IC, E, First, Last)`: the bounds of the law on the edge `e`, None when
    /// there is none; `b.get_bounds(1, &edge)?`.
    pub fn get_bounds(&self, ic: i32, e: &Edge) -> Result<Option<(f64, f64)>, Failure> {
        let law = self.get_law(ic, e)?;
        match law.get() {
            Some(l) => Ok(Some(l.bounds()?)),
            None => Ok(None),
        }
    }

    /// `GetLaw(IC, E)`: the law of the radius on the edge `e` (null out of range);
    /// `b.get_law(1, &edge)?`.
    pub fn get_law(&self, ic: i32, e: &Edge) -> Result<Handle<dyn LawFunction>, Failure> {
        let r = self.with_fil_spine(ic, |f| f.change_law(e, |l| l.clone()))?;
        Ok(r.unwrap_or_default())
    }

    /// `SetLaw(IC, E, L)`: the law of the radius on the edge `e`;
    /// `b.set_law(1, &edge, &law)?`.
    pub fn set_law(
        &mut self,
        ic: i32,
        e: &Edge,
        l: &Handle<dyn LawFunction>,
    ) -> Result<(), Failure> {
        self.with_fil_spine(ic, |f| f.change_law(e, |law| *law = l.clone()))?;
        Ok(())
    }

    /// `Simulate(IC)`: the sections of the contour `ic` without the surfaces;
    /// `b.simulate(1)?`.
    pub fn simulate(&mut self, ic: i32) -> Result<(), Failure> {
        let Some(st) = usize::try_from(ic - 1)
            .ok()
            .and_then(|k| self.base.my_list_stripe.get(k))
            .cloned()
        else {
            return Ok(());
        };
        (self as &mut dyn ChFi3dBuilderOps).perform_set_of_surf(&st, true)
    }

    /// `NbSurf(IC)`: the number of surfaces of the simulated contour (0 out of range);
    /// `b.nb_surf(1)`.
    pub fn nb_surf(&self, ic: i32) -> i32 {
        let Some(st) = usize::try_from(ic - 1)
            .ok()
            .and_then(|k| self.base.my_list_stripe.get(k))
        else {
            return 0;
        };
        lock(st)
            .set_of_surf_data()
            .map(|s| lock(s).len() as i32)
            .unwrap_or(0)
    }

    /// `Sect(IC, IS)`: the sections of the surface `is` of the simulated contour (null
    /// out of range or without simulation); `b.sect(1, 1)?`.
    pub fn sect(&self, ic: i32, is: i32) -> Result<Handle<HArray1<CircSection>>, Failure> {
        let Some(st) = usize::try_from(ic - 1)
            .ok()
            .and_then(|k| self.base.my_list_stripe.get(k))
        else {
            return Ok(Handle::default());
        };
        let sd = crate::chfi3d::surf_data_at(st, is)?;
        let simul = lock(&sd).simul();
        Ok(simul.cast())
    }
}
