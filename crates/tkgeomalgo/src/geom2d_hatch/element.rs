// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of Geom2dHatch_Element.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dHatch_Element`: an oriented boundary curve of the hatcher.
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::top_abs::Orientation;

/// `Geom2dHatch_Element`: a curve (owned by value, as the native member) and its
/// orientation. The native default constructor leaves the orientation
/// uninitialized; the port starts it at FORWARD.
///
/// ```
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkg3d::top_abs::Orientation;
/// use rustycad_tkgeomalgo::geom2d_hatch::Element;
/// let element = Element::new(Geom2dAdaptorCurve::default(), Orientation::Reversed);
/// assert_eq!(element.orientation(), Orientation::Reversed);
/// ```
pub struct Element {
    my_curve: Geom2dAdaptorCurve,
    my_orientation: Orientation,
}

impl Default for Element {
    /// `Geom2dHatch_Element()`: no curve, FORWARD.
    fn default() -> Self {
        Self::new(Geom2dAdaptorCurve::default(), Orientation::Forward)
    }
}

impl Element {
    /// `Geom2dHatch_Element(Curve, Orientation = TopAbs_FORWARD)`: the element keeps
    /// `curve`; `Element::new(adaptor, Orientation::Forward)`.
    pub fn new(curve: Geom2dAdaptorCurve, orientation: Orientation) -> Self {
        Self {
            my_curve: curve,
            my_orientation: orientation,
        }
    }

    /// Curve(): the curve of the element; `element.curve()`.
    pub fn curve(&self) -> &Geom2dAdaptorCurve {
        &self.my_curve
    }

    /// ChangeCurve(): the curve of the element, to modify; `element.change_curve()`.
    pub fn change_curve(&mut self) -> &mut Geom2dAdaptorCurve {
        &mut self.my_curve
    }

    /// Orientation(Orientation): sets the orientation;
    /// `element.set_orientation(Orientation::Reversed)`.
    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.my_orientation = orientation;
    }

    /// Orientation(): the orientation of the element; `element.orientation()`.
    pub fn orientation(&self) -> Orientation {
        self.my_orientation
    }
}
