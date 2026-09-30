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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Line.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_Line`: the root of the intersection lines (docs/geomalgo-design.md section 16).
use super::IntPatchIType;
use super::failures::domain_error;
use crate::int_surf::{IntSurfSituation, IntSurfTypeTrans};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient, Transient};

/// The fields of `IntPatch_Line` (`typ` and the private ones).
#[derive(Clone, Copy, Debug)]
struct LineFields {
    typ: IntPatchIType,
    tg: bool,
    t_s1: IntSurfTypeTrans,
    t_s2: IntSurfTypeTrans,
    sit1: IntSurfSituation,
    sit2: IntSurfSituation,
    iso: [bool; 4],
}

/// How the transitions of a new line are given: the three protected constructors of
/// `IntPatch_Line` (IN/OUT types, TOUCH situations, or undecided).
///
/// `IntPatchLineTransitions::Types(IntSurfTypeTrans::In, IntSurfTypeTrans::Out)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntPatchLineTransitions {
    /// `IntPatch_Line(Tang, Trans1, Trans2)`: IN or OUT transitions.
    Types(IntSurfTypeTrans, IntSurfTypeTrans),
    /// `IntPatch_Line(Tang, Situ1, Situ2)`: TOUCH transitions of these situations.
    Touch(IntSurfSituation, IntSurfSituation),
    /// `IntPatch_Line(Tang)`: undecided transitions.
    Undecided,
}

/// The data of `IntPatch_Line`, embedded by every line class. It is shared through the
/// line handle, so it sits behind a lock (docs/geom-design.md section 3).
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::{IntPatchIType, IntPatchLineBase, IntPatchLineTransitions};
/// let base = IntPatchLineBase::new(true, IntPatchLineTransitions::Undecided, IntPatchIType::Walking);
/// assert!(base.situation_s1().is_err());
/// ```
pub struct IntPatchLineBase {
    fields: SharedState<LineFields>,
}

impl IntPatchLineBase {
    /// The protected constructors of `IntPatch_Line`, with the `typ` the derived
    /// constructor sets. `IntPatchLineBase::new(false, transitions, IntPatchIType::Lin)`.
    pub fn new(tang: bool, transitions: IntPatchLineTransitions, typ: IntPatchIType) -> Self {
        let unknown = IntSurfSituation::Unknown;
        let (t_s1, t_s2, sit1, sit2) = match transitions {
            IntPatchLineTransitions::Types(t1, t2) => (t1, t2, unknown, unknown),
            IntPatchLineTransitions::Touch(s1, s2) => {
                (IntSurfTypeTrans::Touch, IntSurfTypeTrans::Touch, s1, s2)
            }
            IntPatchLineTransitions::Undecided => {
                let undecided = IntSurfTypeTrans::Undecided;
                (undecided, undecided, unknown, unknown)
            }
        };
        let fields = LineFields {
            typ,
            tg: tang,
            t_s1,
            t_s2,
            sit1,
            sit2,
            iso: [false; 4],
        };
        Self {
            fields: SharedState::new(fields),
        }
    }

    /// `SetValue(Uiso1, Viso1, Uiso2, Viso2)`. `base.set_value(true, false, false, false)`.
    pub fn set_value(&self, uiso1: bool, viso1: bool, uiso2: bool, viso2: bool) {
        self.fields.write().iso = [uiso1, viso1, uiso2, viso2];
    }

    /// `ArcType()`. `base.arc_type()`.
    pub fn arc_type(&self) -> IntPatchIType {
        self.fields.read().typ
    }

    /// `IsTangent()`. `base.is_tangent()`.
    pub fn is_tangent(&self) -> bool {
        self.fields.read().tg
    }

    /// `TransitionOnS1()`. `base.transition_on_s1()`.
    pub fn transition_on_s1(&self) -> IntSurfTypeTrans {
        self.fields.read().t_s1
    }

    /// `TransitionOnS2()`. `base.transition_on_s2()`.
    pub fn transition_on_s2(&self) -> IntSurfTypeTrans {
        self.fields.read().t_s2
    }

    /// `SituationS1()`: Standard_DomainError unless the transition on S1 is TOUCH.
    /// `base.situation_s1()?`.
    pub fn situation_s1(&self) -> Result<IntSurfSituation, Failure> {
        let f: LineFields = *self.fields.read();
        situation(f.t_s1, f.sit1, "SituationS1")
    }

    /// `SituationS2()`: Standard_DomainError unless the transition on S2 is TOUCH.
    /// `base.situation_s2()?`.
    pub fn situation_s2(&self) -> Result<IntSurfSituation, Failure> {
        let f: LineFields = *self.fields.read();
        situation(f.t_s2, f.sit2, "SituationS2")
    }

    /// `IsUIsoOnS1()`, `IsVIsoOnS1()`, `IsUIsoOnS2()` and `IsVIsoOnS2()` in that order.
    /// `base.isos()[0]`.
    pub fn isos(&self) -> [bool; 4] {
        self.fields.read().iso
    }
}

fn situation(
    t: IntSurfTypeTrans,
    sit: IntSurfSituation,
    member: &str,
) -> Result<IntSurfSituation, Failure> {
    if t != IntSurfTypeTrans::Touch {
        let reason = format!("the transition is {t:?}; expected Touch");
        return Err(domain_error(&format!("IntPatch_Line::{member}"), &reason));
    }
    Ok(sit)
}

/// `IntPatch_Line`: an intersection line between two surfaces (a conic, an analytic
/// curve, a walking polyline or a restriction), reached through
/// `Handle<dyn IntPatchLine>`. Every member is non virtual natively: the provided
/// methods read the embedded [`IntPatchLineBase`].
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::{IntPatchIType, IntPatchLine, IntPatchRLine};
/// let line = IntPatchRLine::new_undecided(false);
/// assert_eq!(line.arc_type(), IntPatchIType::Restriction);
/// ```
pub trait IntPatchLine: Transient {
    /// The embedded `IntPatch_Line` data.
    fn line_base(&self) -> &IntPatchLineBase;

    /// `SetValue(Uiso1, Viso1, Uiso2, Viso2)`: the isoparametric flags.
    fn set_value(&self, uiso1: bool, viso1: bool, uiso2: bool, viso2: bool) {
        self.line_base().set_value(uiso1, viso1, uiso2, viso2);
    }
    /// `ArcType()`.
    fn arc_type(&self) -> IntPatchIType {
        self.line_base().arc_type()
    }
    /// `IsTangent()`.
    fn is_tangent(&self) -> bool {
        self.line_base().is_tangent()
    }
    /// `TransitionOnS1()`.
    fn transition_on_s1(&self) -> IntSurfTypeTrans {
        self.line_base().transition_on_s1()
    }
    /// `TransitionOnS2()`.
    fn transition_on_s2(&self) -> IntSurfTypeTrans {
        self.line_base().transition_on_s2()
    }
    /// `SituationS1()`: Standard_DomainError unless the transition on S1 is TOUCH.
    fn situation_s1(&self) -> Result<IntSurfSituation, Failure> {
        self.line_base().situation_s1()
    }
    /// `SituationS2()`: Standard_DomainError unless the transition on S2 is TOUCH.
    fn situation_s2(&self) -> Result<IntSurfSituation, Failure> {
        self.line_base().situation_s2()
    }
    /// `IsUIsoOnS1()`.
    fn is_u_iso_on_s1(&self) -> bool {
        self.line_base().isos()[0]
    }
    /// `IsVIsoOnS1()`.
    fn is_v_iso_on_s1(&self) -> bool {
        self.line_base().isos()[1]
    }
    /// `IsUIsoOnS2()`.
    fn is_u_iso_on_s2(&self) -> bool {
        self.line_base().isos()[2]
    }
    /// `IsVIsoOnS2()`.
    fn is_v_iso_on_s2(&self) -> bool {
        self.line_base().isos()[3]
    }
}
impl_transient_interface!(dyn IntPatchLine, "IntPatch_Line", StandardTransient);
