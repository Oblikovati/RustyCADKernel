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

//! The probe's AppDef_MultiLine reader: the twin of `gb_apc_read_line`
//! (tools/oracle/tkgeombase_probe_appparcurves_fit.cxx).
use super::super::common::Reader;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkgeombase::app_def::{MultiLine, MultiPointConstraint};
use rustycad_tkmath::gp::{Point2, Point3};

/// The multipoint of `p3` and `p2` through the constructor the native probe picks.
pub fn multi_point(p3: &[Point3], p2: &[Point2]) -> MultiPointConstraint {
    let (a3, a2) = (Array1::from_shared(p3, 1), Array1::from_shared(p2, 1));
    if p2.is_empty() {
        return MultiPointConstraint::from_points(&a3);
    }
    if p3.is_empty() {
        return MultiPointConstraint::from_points2d(&a2);
    }
    MultiPointConstraint::from_points_and_2d(&a3, &a2)
}

// The tangents (pass 0) or curvatures (pass 1) of the flagged multipoint.
fn read_vectors(r: &mut Reader<'_>, m: &mut MultiPointConstraint, pass: i32, nb3d: i32, nb2d: i32) {
    for k in 1..=nb3d {
        let v = r.vec();
        let set = if pass == 0 {
            m.set_tang(k, v)
        } else {
            m.set_curv(k, v)
        };
        set.expect("probe line vector");
    }
    for k in 1..=nb2d {
        let v = r.vec2d();
        let set = if pass == 0 {
            m.set_tang2d(nb3d + k, v)
        } else {
            m.set_curv2d(nb3d + k, v)
        };
        set.expect("probe line vector");
    }
}

/// `nb3d nb2d n` then per multipoint `tangent curvature points... [tangents] [curvatures]`.
pub fn read_line(r: &mut Reader<'_>) -> MultiLine {
    let (nb3d, nb2d, n) = (r.integer(), r.integer(), r.integer());
    let line = MultiLine::with_nb_multi_points(n).expect("probe line size");
    for i in 1..=n {
        let flags = [r.flag(), r.flag()];
        let p3: Vec<Point3> = (0..nb3d).map(|_| r.pnt()).collect();
        let p2: Vec<Point2> = (0..nb2d).map(|_| r.pnt2d()).collect();
        let mut m = multi_point(&p3, &p2);
        for pass in 0..2 {
            if flags[pass as usize] {
                read_vectors(r, &mut m, pass, nb3d, nb2d);
            }
        }
        line.set_value(i, &m).expect("probe line value");
    }
    line
}
