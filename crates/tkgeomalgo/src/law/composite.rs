// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of ModelingAlgorithms/TKGeomAlgo/Law/Law_Composite.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Law_Composite`: a law made of consecutive elementary laws.
use super::LawFunction;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::{Array1, List};
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::{at, one_based, put};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::Shape;

/// A handle on an elementary law.
type LawHandle = Handle<dyn LawFunction>;

/// The fields of a composite law. `curfunc` is the elementary law of the last
/// evaluation; `first` and `last` are the bounds of the whole list, known after the
/// first evaluation.
struct CompositeFields {
    first: f64,
    last: f64,
    curfunc: LawHandle,
    funclist: List<LawHandle>,
    periodic: bool,
    t_first: f64,
    t_last: f64,
    p_tol: f64,
}

/// A law made of a list of elementary laws (`Law_Composite`); evaluation picks the
/// law whose bounds hold the parameter, shifted by the tolerance towards the nearer
/// end of the trimmed range so that a shared knot goes to the inner law.
///
/// `let law = LawComposite::new(); law.change_laws(|laws| { laws.append(piece); });`
pub struct LawComposite {
    parent: StandardTransient,
    fields: SharedState<CompositeFields>,
}
impl_transient!(LawComposite, "Law_Composite", dyn LawFunction, parent, interfaces: [dyn LawFunction]);

impl Default for LawComposite {
    fn default() -> Self {
        Self::new()
    }
}

/// The `Bounds` of an elementary law, Standard_NullObject for a null one.
fn law_bounds(law: &LawHandle) -> Result<(f64, f64), Failure> {
    live(law)?.bounds()
}

fn live(law: &LawHandle) -> Result<&dyn LawFunction, Failure> {
    law.get()
        .ok_or_else(|| errors::null_object("Law_Composite: null elementary law"))
}

/// The first or last law of the list, Standard_NoSuchObject for an empty list.
fn end_law(list: &List<LawHandle>, last: bool) -> Result<LawHandle, Failure> {
    let law: Option<&LawHandle> = if last { list.last() } else { list.first() };
    law.cloned().ok_or_else(|| {
        errors::no_such_object("Law_Composite: empty list of laws; expected at least one law")
    })
}

impl LawComposite {
    /// An empty, untrimmed composite (`Law_Composite()`); `LawComposite::new()`.
    pub fn new() -> Self {
        Self::with_range(-1.0e100, 1.0e100, 0.0)
    }

    /// An empty composite trimmed to `[first, last]` with tolerance `tol`
    /// (`Law_Composite(First, Last, Tol)`); `LawComposite::with_range(0.0, 1.0, 1e-9)`.
    pub fn with_range(first: f64, last: f64, tol: f64) -> Self {
        Self {
            parent: StandardTransient::default(),
            fields: SharedState::new(CompositeFields {
                first: -1.0e100,
                last: 1.0e100,
                curfunc: Handle::default(),
                funclist: List::default(),
                periodic: false,
                t_first: first,
                t_last: last,
                p_tol: tol,
            }),
        }
    }

    /// `Prepare`: selects the elementary law for `w` and returns it with `w`, which a
    /// periodic composite brings into its period. The lock is never held while an
    /// elementary law is queried.
    fn prepare(&self, w: f64) -> Result<(LawHandle, f64), Failure> {
        let (t_first, t_last, p_tol, periodic) = {
            let fields = self.fields.read();
            (fields.t_first, fields.t_last, fields.p_tol, fields.periodic)
        };
        let eps: f64 = if w - t_first < t_last - w {
            p_tol
        } else {
            -p_tol
        };
        let (first, last, current) = self.range_and_current()?;
        let mut w_param: f64 = w;
        let mut w_test: f64 = w + eps; // shift to discriminate the knots
        if periodic {
            w_test = elclib::in_period(w_test, first, last);
            w_param = w_test - eps;
        }
        let (f, l) = law_bounds(&current)?;
        if f <= w_test && w_test <= l {
            return Ok((current, w_param));
        }
        let chosen: LawHandle = self.search(w_param, w_test, first, last)?;
        self.fields.write().curfunc = chosen.clone();
        Ok((chosen, w_param))
    }

    /// The list bounds and the current law; the first call takes the bounds from the
    /// last and first laws and makes the first law current.
    fn range_and_current(&self) -> Result<(f64, f64, LawHandle), Failure> {
        let (curfunc, list) = {
            let fields = self.fields.read();
            (fields.curfunc.clone(), fields.funclist.clone())
        };
        if curfunc.is_some() {
            let fields = self.fields.read();
            return Ok((fields.first, fields.last, curfunc));
        }
        let (_, last) = law_bounds(&end_law(&list, true)?)?;
        let first_law: LawHandle = end_law(&list, false)?;
        let (first, _) = law_bounds(&first_law)?;
        let mut fields = self.fields.write();
        (fields.first, fields.last, fields.curfunc) = (first, last, first_law.clone());
        Ok((first, last, first_law))
    }

    /// The law of `w`: the first or last one outside the range, else the first whose
    /// bounds hold `w_test` (the last one when none does, as the native loop ends).
    fn search(&self, w: f64, w_test: f64, first: f64, last: f64) -> Result<LawHandle, Failure> {
        let list: List<LawHandle> = self.fields.read().funclist.clone();
        if w <= first {
            return end_law(&list, false);
        }
        if w >= last {
            return end_law(&list, true);
        }
        let mut chosen: LawHandle = self.fields.read().curfunc.clone();
        for law in list.iter() {
            chosen = law.clone();
            let (f, l) = law_bounds(law)?;
            if f <= w_test && w_test <= l {
                break;
            }
        }
        Ok(chosen)
    }

    /// Lends the elementary law used at `w` to `edit`, which may replace it
    /// (`ChangeElementaryLaw` returns a reference to it natively); `edit` must not call
    /// this composite. `law.change_elementary_law(0.5, |piece| piece.clone())?`.
    pub fn change_elementary_law<R>(
        &self,
        w: f64,
        edit: impl FnOnce(&mut LawHandle) -> R,
    ) -> Result<R, Failure> {
        self.prepare(w)?;
        Ok(edit(&mut self.fields.write().curfunc))
    }

    /// Lends the list of elementary laws to `edit` (`ChangeLaws`); `edit` must not call
    /// this composite. `law.change_laws(|laws| laws.append(piece.clone()));`
    pub fn change_laws<R>(&self, edit: impl FnOnce(&mut List<LawHandle>) -> R) -> R {
        edit(&mut self.fields.write().funclist)
    }

    /// Whether the composite is periodic (`IsPeriodic`); `law.is_periodic()`.
    pub fn is_periodic(&self) -> bool {
        self.fields.read().periodic
    }

    /// Makes the composite periodic (`SetPeriodic`); `law.set_periodic()`.
    pub fn set_periodic(&self) {
        self.fields.write().periodic = true;
    }
}

impl LawFunction for LawComposite {
    /// Standard_NotImplemented, as natively.
    fn continuity(&self) -> Result<Shape, Failure> {
        Err(errors::not_implemented("Law_Composite::Continuity()"))
    }
    /// The sum of the interval counts of the elementary laws.
    fn nb_intervals(&self, s: Shape) -> Result<i32, Failure> {
        let list: List<LawHandle> = self.fields.read().funclist.clone();
        let mut count: i32 = 0;
        for law in list.iter() {
            count += live(law)?.nb_intervals(s)?;
        }
        Ok(count)
    }
    /// The bounds of the first law at `T(1)`, `T(2)`, then the interval ends of every
    /// law from `T(2)` on (absolute indices, as natively).
    fn intervals(&self, t: &mut Array1<'_, f64>, s: Shape) -> Result<(), Failure> {
        let list: List<LawHandle> = self.fields.read().funclist.clone();
        let (first, last) = law_bounds(&end_law(&list, false)?)?;
        put(t, 1, first)?;
        put(t, 2, last)?;
        let mut i_glob: i32 = 2;
        for law in list.iter() {
            let law: &dyn LawFunction = live(law)?;
            let nb_index: i32 = law.nb_intervals(s)? + 1;
            let mut local: Array1<'static, f64> = one_based(nb_index)?;
            law.intervals(&mut local, s)?;
            for i_loc in 2..=nb_index {
                put(t, i_glob, at(&local, i_loc)?)?;
                i_glob += 1;
            }
        }
        Ok(())
    }
    fn value(&self, x: f64) -> Result<f64, Failure> {
        let (law, w) = self.prepare(x)?;
        live(&law)?.value(w)
    }
    fn d1(&self, x: f64) -> Result<(f64, f64), Failure> {
        let (law, w) = self.prepare(x)?;
        live(&law)?.d1(w)
    }
    fn d2(&self, x: f64) -> Result<(f64, f64, f64), Failure> {
        let (law, w) = self.prepare(x)?;
        live(&law)?.d2(w)
    }
    /// A composite trimmed to `[p_first, p_last]` sharing the elementary laws.
    fn trim(&self, p_first: f64, p_last: f64, tol: f64) -> Result<LawHandle, Failure> {
        let trimmed = LawComposite::with_range(p_first, p_last, tol);
        let list: List<LawHandle> = self.fields.read().funclist.clone();
        trimmed.change_laws(|laws| *laws = list);
        Ok(Handle::new(trimmed).cast())
    }
    /// The bounds of the whole list, `(-1e100, 1e100)` before the first evaluation.
    fn bounds(&self) -> Result<(f64, f64), Failure> {
        let fields = self.fields.read();
        Ok((fields.first, fields.last))
    }
}
