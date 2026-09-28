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

//! The probe's MultiLine: a test double with the semantics of AppDef_MultiLine and
//! AppDef_MyLineTool, which the native probe instantiates the generics with (AppDef
//! itself is not ported yet).
use super::super::common::Reader;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_par_curves::LineTool;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

#[derive(Clone)]
struct ProbePoint {
    p3: Vec<Point3>,
    p2: Vec<Point2>,
    tangents: Option<(Vec<Vector3>, Vec<Vector2>)>,
    curvatures: Option<(Vec<Vector3>, Vec<Vector2>)>,
}

/// AppDef_MultiLine of AppDef_MultiPointConstraint with every tangent (and curvature)
/// of a flagged point set.
#[derive(Clone)]
pub struct ProbeLine {
    nb3d: i32,
    nb2d: i32,
    points: Vec<ProbePoint>,
}

fn vectors(r: &mut Reader<'_>, nb3d: i32, nb2d: i32) -> (Vec<Vector3>, Vec<Vector2>) {
    (
        (0..nb3d).map(|_| r.vec()).collect(),
        (0..nb2d).map(|_| r.vec2d()).collect(),
    )
}

impl ProbeLine {
    /// `nb3d nb2d n` then per point `tangent curvature points... [tangents] [curvatures]`.
    pub fn read(r: &mut Reader<'_>) -> Self {
        let (nb3d, nb2d, n) = (r.integer(), r.integer(), r.integer());
        let points = (0..n)
            .map(|_| {
                let (tangent, curvature) = (r.flag(), r.flag());
                let p3 = (0..nb3d).map(|_| r.pnt()).collect();
                let p2 = (0..nb2d).map(|_| r.pnt2d()).collect();
                let tangents = tangent.then(|| vectors(r, nb3d, nb2d));
                let curvatures = curvature.then(|| vectors(r, nb3d, nb2d));
                ProbePoint {
                    p3,
                    p2,
                    tangents,
                    curvatures,
                }
            })
            .collect();
        Self { nb3d, nb2d, points }
    }

    // ML.Value(Index): Standard_OutOfRange outside 1..=NbMultiPoints.
    fn point(&self, index: i32) -> Result<&ProbePoint, Failure> {
        usize::try_from(index - 1)
            .ok()
            .and_then(|slot| self.points.get(slot))
            .ok_or_else(|| Failure::out_of_range("AppDef_MultiLine::Value"))
    }
}

// tab(i + low - 1) = values[i - 1] for every value.
fn fill<T: Copy>(tab: &mut Array1<'_, T>, values: &[T]) -> Result<(), Failure> {
    let low: i32 = tab.lower();
    for (value, i) in values.iter().zip(0..) {
        tab.set_value(low + i, *value)
            .map_err(|error| Failure::out_of_range(&error.to_string()))?;
    }
    Ok(())
}

impl LineTool for ProbeLine {
    fn first_point(&self) -> i32 {
        1
    }
    fn last_point(&self) -> i32 {
        self.points.len() as i32
    }
    fn nb_p2d(&self) -> i32 {
        self.nb2d
    }
    fn nb_p3d(&self) -> i32 {
        self.nb3d
    }
    fn value_3d(&self, index: i32, points: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        fill(points, &self.point(index)?.p3)
    }
    fn value_2d(&self, index: i32, points2d: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        fill(points2d, &self.point(index)?.p2)
    }
    fn value(
        &self,
        index: i32,
        points: &mut Array1<'_, Point3>,
        points2d: &mut Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        let point = self.point(index)?;
        fill(points, &point.p3)?;
        fill(points2d, &point.p2)
    }
    fn tangency_3d(&self, index: i32, tangents: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        let Some((t3, _)) = &self.point(index)?.tangents else {
            return Ok(false);
        };
        fill(tangents, t3).map(|()| true)
    }
    fn tangency_2d(
        &self,
        index: i32,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let Some((_, t2)) = &self.point(index)?.tangents else {
            return Ok(false);
        };
        fill(tangents2d, t2).map(|()| true)
    }
    fn tangency(
        &self,
        index: i32,
        tangents: &mut Array1<'_, Vector3>,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let Some((t3, t2)) = &self.point(index)?.tangents else {
            return Ok(false);
        };
        fill(tangents, t3)?;
        fill(tangents2d, t2).map(|()| true)
    }
    fn curvature_3d(
        &self,
        index: i32,
        curvatures: &mut Array1<'_, Vector3>,
    ) -> Result<bool, Failure> {
        let Some((c3, _)) = &self.point(index)?.curvatures else {
            return Ok(false);
        };
        fill(curvatures, c3).map(|()| true)
    }
    fn curvature_2d(
        &self,
        index: i32,
        curvatures2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let Some((_, c2)) = &self.point(index)?.curvatures else {
            return Ok(false);
        };
        fill(curvatures2d, c2).map(|()| true)
    }
    fn curvature(
        &self,
        index: i32,
        curvatures: &mut Array1<'_, Vector3>,
        curvatures2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let Some((c3, c2)) = &self.point(index)?.curvatures else {
            return Ok(false);
        };
        fill(curvatures, c3)?;
        fill(curvatures2d, c2).map(|()| true)
    }
}
