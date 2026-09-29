// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of the implicit copy members of IntCurve_IntCurveCurveGen.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The implicit copy constructor and assignment of `IntCurve_IntCurveCurveGen`, which
//! classes deriving from `Geom2dInt_GInter` (Geom2dHatch_Intersector) copy by value.
use super::{IConicTool, IntCurveCurveGen, PCurveTool};
use crate::int_imp_par_gen::IntersectorTypes;

impl<T: IntersectorTypes<Imp = IConicTool>> Clone for IntCurveCurveGen<T>
where
    T::Par: PCurveTool,
{
    /// The implicit copy constructor: every member copied, the IntRes2d_Intersection
    /// part by its copy constructor (neither done nor reversed).
    fn clone(&self) -> Self {
        Self {
            base: self.base.clone(),
            param1inf: self.param1inf,
            param1sup: self.param1sup,
            param2inf: self.param2inf,
            param2sup: self.param2sup,
            intconiconi: self.intconiconi.clone(),
            intconicurv: self.intconicurv.clone(),
            intcurvcurv: self.intcurvcurv.clone(),
        }
    }
}

impl<T: IntersectorTypes<Imp = IConicTool>> IntCurveCurveGen<T>
where
    T::Par: PCurveTool,
{
    /// The implicit `operator=`: the IntRes2d_Intersection part by its assignment (the
    /// results and both flags), the other members copied. The inner intersectors are
    /// copied with their copy constructor: each Perform sets their flags before use.
    /// `target.assign(&source)`.
    pub fn assign(&mut self, other: &Self) {
        self.base.assign(&other.base);
        self.param1inf = other.param1inf;
        self.param1sup = other.param1sup;
        self.param2inf = other.param2inf;
        self.param2sup = other.param2sup;
        self.intconiconi = other.intconiconi.clone();
        self.intconicurv = other.intconicurv.clone();
        self.intcurvcurv = other.intcurvcurv.clone();
    }
}
