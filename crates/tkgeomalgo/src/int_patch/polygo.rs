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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_Polygo.hxx/.lxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_Polygo`: the 2d polygons of IntPatch (a walking or restriction line, an
//! arc of restriction), as `Intf_Polygon2d` polylines with a deflection.
use super::dump::ostream_real;
use crate::intf::Polygon2d;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::gp::Point2;
use std::fmt::Write;
use std::sync::atomic::{AtomicI32, Ordering};

/// The protected data of `IntPatch_Polygo` (`myError`) with the `myBox` of
/// `Intf_Polygon2d`, embedded by the polygons.
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchPolygoBase;
/// assert_eq!(IntPatchPolygoBase::new(0.5).error, 0.5);
/// ```
#[derive(Clone, Debug, Default)]
pub struct IntPatchPolygoBase {
    /// `myError`: the deflection of the polygon.
    pub error: f64,
    /// `myBox`: the box of the points, enlarged by the deflection.
    pub bounding: BoundingBox2,
}

impl IntPatchPolygoBase {
    /// The protected `IntPatch_Polygo(theError)`. `IntPatchPolygoBase::new(0.0)`.
    pub fn new(error: f64) -> Self {
        Self {
            error,
            bounding: BoundingBox2::default(),
        }
    }
}

/// The number of the dumps (`static int num` of `IntPatch_Polygo::Dump`).
static DUMP_NUMBER: AtomicI32 = AtomicI32::new(0);

/// `IntPatch_Polygo`: a polyline of points; its segments join consecutive points.
/// The implementors give their points; `Error`, `NbSegments`, `Segment` and `Dump` are
/// the native bodies.
///
/// ```ignore
/// let n = polygon.nb_segments();
/// let (begin, end) = polygon.segment(1)?;
/// ```
pub trait IntPatchPolygo: Polygon2d {
    /// The embedded `IntPatch_Polygo` data.
    fn polygo_base(&self) -> &IntPatchPolygoBase;
    /// `NbPoints()`.
    fn nb_points(&self) -> Result<i32, Failure>;
    /// `Point(Index)`.
    fn point(&self, index: i32) -> Result<Point2, Failure>;

    /// `Error()`: the deflection.
    fn error(&self) -> f64 {
        self.polygo_base().error
    }

    /// `Dump()`: the box, the deflection and the points, numbered by a counter shared
    /// by every dump of the process.
    fn dump(&self, out: &mut dyn Write) -> Result<(), Failure> {
        let num: i32 = DUMP_NUMBER.fetch_add(1, Ordering::Relaxed) + 1;
        let mut box_text: Vec<u8> = Vec::new();
        self.bounding().dump(&mut box_text).map_err(|e| {
            Failure::with_message(
                "Standard_Failure",
                &format!("IntPatch_Polygo::Dump() - {e}"),
            )
        })?;
        let nbs: i32 = self.nb_segments();
        let mut text = format!("\n#------------- D u m p     B o x 2 d   ({num})\n");
        text.push_str(&String::from_utf8_lossy(&box_text));
        text.push_str("\n#-----------------------------------------------\n");
        text.push_str(&format!(
            "\npol2d {num} {nbs} {}\n",
            ostream_real(self.deflection_over_estimation())
        ));
        let mut pf: Point2 = Point2::default();
        for i in 1..=nbs {
            let (p, end) = self.segment(i)?;
            pf = end;
            text.push_str(&format!(
                "pnt2d {num}  {} {}\n",
                ostream_real(p.x()),
                ostream_real(p.y())
            ));
        }
        text.push_str(&format!(
            "pnt2d {num}  {} {}\n",
            ostream_real(pf.x()),
            ostream_real(pf.y())
        ));
        out.write_str(&text).map_err(super::dump::sink_failure)
    }
}

/// `NbSegments()` of `IntPatch_Polygo`: `NbPoints() - 1` (-1 when the points cannot be
/// read, which the native code does not guard).
pub(crate) fn polygo_nb_segments<P: IntPatchPolygo + ?Sized>(polygon: &P) -> i32 {
    polygon.nb_points().map_or(-1, |n| n - 1)
}

/// `Segment(theIndex, theBegin, theEnd)` of `IntPatch_Polygo`: the points `index` and
/// `index + 1`.
pub(crate) fn polygo_segment<P: IntPatchPolygo + ?Sized>(
    polygon: &P,
    index: i32,
) -> Result<(Point2, Point2), Failure> {
    Ok((polygon.point(index)?, polygon.point(index + 1)?))
}

/// Implements `Intf_Polygon2d` for an `IntPatch_Polygo` with the inline bodies of
/// IntPatch_Polygo.lxx (`DeflectionOverEstimation` is the error).
macro_rules! polygo_polygon2d {
    ($polygon:ty) => {
        impl crate::intf::Polygon2d for $polygon {
            fn bounding(&self) -> &rustycad_tkmath::bnd::BoundingBox2 {
                &self.base.bounding
            }
            fn closed(&self) -> bool {
                self.is_closed()
            }
            fn deflection_over_estimation(&self) -> f64 {
                self.base.error
            }
            fn nb_segments(&self) -> i32 {
                crate::int_patch::polygo::polygo_nb_segments(self)
            }
            fn segment(
                &self,
                index: i32,
            ) -> Result<
                (rustycad_tkmath::gp::Point2, rustycad_tkmath::gp::Point2),
                rustycad_tkernel::standard::Failure,
            > {
                crate::int_patch::polygo::polygo_segment(self, index)
            }
        }
    };
}
pub(crate) use polygo_polygon2d;
