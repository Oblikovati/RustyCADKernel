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

//! The line of the Approx driver probes: an AppDef_MultiLine read by the AppParCurves
//! probe, with the scripted Approx members of GbAxLineTool
//! (tools/oracle/tkgeombase_probe_approx_line.cxx).
use super::super::app_par_curves::{multi_point, read_line};
use super::super::common::Reader;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_def::{MultiLine, MultiPointConstraint};
use rustycad_tkgeombase::app_par_curves::LineTool;
use rustycad_tkgeombase::approx::Status;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// `gb_ax_p3(k, t)`: 3D source curve k.
pub fn source_p3(k: i32, t: f64) -> Point3 {
    let c: f64 = f64::from(k);
    Point3::new((t * t - 1.0) * c, (t * t * t - 1.1 * t) * c, 0.25 * t + c)
}

/// `gb_ax_d3(k, t)`: its derivative.
pub fn source_d3(k: i32, t: f64) -> Vector3 {
    let c: f64 = f64::from(k);
    Vector3::new(2.0 * t * c, (3.0 * t * t - 1.1) * c, 0.25)
}

/// `gb_ax_p2(k, t)`: 2D source curve k.
pub fn source_p2(k: i32, t: f64) -> Point2 {
    let c: f64 = f64::from(k);
    Point2::new(t * t - 1.0 + (c - 1.0), (t * t * t - 1.1 * t) * c)
}

/// `gb_ax_d2(k, t)`: its derivative.
pub fn source_d2(k: i32, t: f64) -> Vector2 {
    let c: f64 = f64::from(k);
    Vector2::new(2.0 * t, (3.0 * t * t - 1.1) * c)
}

/// The rules of GbAxLineTool.
#[derive(Clone, Copy, Default)]
pub struct Rules {
    mode: i32,
    cap: i32,
    source: bool,
}

/// An AppDef line with the parameter of each source point and the request's rules.
#[derive(Clone)]
pub struct AxLine {
    line: MultiLine,
    params: Vec<Option<f64>>,
    rules: Rules,
}

// gb_ax_source_point: the multipoint at `t`, with the source tangents when flagged.
fn source_point(nb3d: i32, nb2d: i32, t: f64, tangent: bool) -> MultiPointConstraint {
    let p3: Vec<Point3> = (1..=nb3d).map(|k| source_p3(k, t)).collect();
    let p2: Vec<Point2> = (1..=nb2d).map(|k| source_p2(k, t)).collect();
    let mut m = multi_point(&p3, &p2);
    if tangent {
        for k in 1..=nb3d {
            m.set_tang(k, source_d3(k, t)).expect("probe tangent");
        }
        for k in 1..=nb2d {
            m.set_tang2d(nb3d + k, source_d2(k, t))
                .expect("probe tangent");
        }
    }
    m
}

// The line of `points`, as `AppDef_MultiLine(n)` and SetValue.
fn line_of(points: &[MultiPointConstraint]) -> MultiLine {
    MultiLine::from_multi_points(&Array1::from_shared(points, 1))
}

impl AxLine {
    /// `gb_ax_read_line`: kind 0 an AppDef line, kind 1 source samples; then the rules.
    pub fn read(r: &mut Reader<'_>) -> Self {
        let kind: i32 = r.integer();
        let (line, params) = if kind == 0 {
            let line = read_line(r);
            let n = line.nb_multi_points().expect("probe line") as usize;
            (line, vec![None; n])
        } else {
            let (nb3d, nb2d, n) = (r.integer(), r.integer(), r.integer());
            let mut points = Vec::new();
            let mut params = Vec::new();
            for _ in 0..n {
                let t: f64 = r.real();
                points.push(source_point(nb3d, nb2d, t, r.flag()));
                params.push(Some(t));
            }
            (line_of(&points), params)
        };
        let rules = Rules {
            mode: r.integer(),
            cap: r.integer(),
            source: kind == 1,
        };
        Self {
            line,
            params,
            rules,
        }
    }

    // gb_ax_param: the parameter of a source point.
    fn param(&self, index: i32) -> Result<f64, Failure> {
        self.line.value(index)?;
        self.params[(index - 1) as usize]
            .ok_or_else(|| Failure::new("probe: a point without parameter"))
    }

    fn tangent(&self, index: i32) -> Result<bool, Failure> {
        Ok(self.line.value(index)?.is_tangency_point())
    }

    fn with_points(&self, points: Vec<MultiPointConstraint>, params: Vec<Option<f64>>) -> Self {
        Self {
            line: line_of(&points),
            params,
            rules: self.rules,
        }
    }
}

impl LineTool for AxLine {
    fn first_point(&self) -> i32 {
        self.line.first_point()
    }
    fn last_point(&self) -> Result<i32, Failure> {
        self.line.last_point()
    }
    fn nb_p2d(&self) -> Result<i32, Failure> {
        self.line.nb_p2d()
    }
    fn nb_p3d(&self) -> Result<i32, Failure> {
        self.line.nb_p3d()
    }
    fn value_3d(&self, index: i32, points: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        self.line.value_3d(index, points)
    }
    fn value_2d(&self, index: i32, points2d: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        self.line.value_2d(index, points2d)
    }
    fn value(
        &self,
        index: i32,
        points: &mut Array1<'_, Point3>,
        points2d: &mut Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        LineTool::value(&self.line, index, points, points2d)
    }
    fn tangency_3d(&self, index: i32, tangents: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        self.line.tangency_3d(index, tangents)
    }
    fn tangency_2d(
        &self,
        index: i32,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        self.line.tangency_2d(index, tangents2d)
    }
    fn tangency(
        &self,
        index: i32,
        tangents: &mut Array1<'_, Vector3>,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        self.line.tangency(index, tangents, tangents2d)
    }
    fn curvature_3d(&self, index: i32, c: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        self.line.curvature_3d(index, c)
    }
    fn curvature_2d(&self, index: i32, c: &mut Array1<'_, Vector2>) -> Result<bool, Failure> {
        self.line.curvature_2d(index, c)
    }
    fn curvature(
        &self,
        index: i32,
        c: &mut Array1<'_, Vector3>,
        c2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        self.line.curvature(index, c, c2d)
    }
    fn what_status(&self, i1: i32, i2: i32) -> Result<Status, Failure> {
        let n: i32 = self.line.nb_multi_points()?;
        let added = if n < self.rules.cap {
            Status::PointsAdded
        } else {
            Status::NoPointsAdded
        };
        Ok(match self.rules.mode {
            1 => added,
            2 => Status::NoApproximation,
            3 => [added, Status::NoPointsAdded, Status::NoApproximation]
                [((i1 + i2 + n) % 3) as usize],
            _ => Status::NoPointsAdded,
        })
    }
    fn make_ml_between(&self, i1: i32, i2: i32, nb_p_min: i32) -> Result<Self, Failure> {
        if !self.rules.source {
            return Ok(self.clone());
        }
        let n: i32 = (2 * (i2 - i1) + 1).max(nb_p_min);
        let (t1, t2) = (self.param(i1)?, self.param(i2)?);
        let (tan1, tan2) = (self.tangent(i1)?, self.tangent(i2)?);
        let (nb3d, nb2d) = (self.nb_p3d()?, self.nb_p2d()?);
        let mut points = Vec::new();
        let mut params = Vec::new();
        for k in 0..n {
            let t: f64 = if n == 1 {
                t1
            } else {
                t1 + (t2 - t1) * f64::from(k) / f64::from(n - 1)
            };
            let tangent: bool = (k == 0 && tan1) || (k == n - 1 && tan2);
            points.push(source_point(nb3d, nb2d, t, tangent));
            params.push(Some(t));
        }
        Ok(self.with_points(points, params))
    }
    fn make_ml_one_more_point(
        &self,
        i1: i32,
        i2: i32,
        index_bad: i32,
    ) -> Result<Option<Self>, Failure> {
        let n: i32 = self.line.nb_multi_points()?;
        if !self.rules.source || n >= self.rules.cap || index_bad <= i1 || index_bad > i2 {
            return Ok(None);
        }
        let mut points = Vec::new();
        let mut params = Vec::new();
        for i in i1..=i2 {
            if i == index_bad {
                let t: f64 = 0.5 * (self.param(i - 1)? + self.param(i)?);
                points.push(source_point(self.nb_p3d()?, self.nb_p2d()?, t, false));
                params.push(Some(t));
            }
            points.push(self.line.value(i)?);
            params.push(self.params[(i - 1) as usize]);
        }
        Ok(Some(self.with_points(points, params)))
    }
}
