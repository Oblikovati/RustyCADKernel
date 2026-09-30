//! `ApproxInt_MultiLineTool.lxx`: the static tool of the AppParCurves generics over the
//! multi-line, as its `LineTool` implementation (docs/geomalgo-design.md section 20).
use super::ApproxIntMultiLine;
use crate::approx_int::line::ApproxIntLine;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::app_par_curves::LineTool;
use rustycad_tkgeombase::approx::Status;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

impl<L: ApproxIntLine> LineTool for ApproxIntMultiLine<L> {
    fn first_point(&self) -> i32 {
        ApproxIntMultiLine::first_point(self)
    }
    fn last_point(&self) -> Result<i32, Failure> {
        Ok(ApproxIntMultiLine::last_point(self))
    }
    fn nb_p2d(&self) -> Result<i32, Failure> {
        Ok(ApproxIntMultiLine::nb_p2d(self))
    }
    fn nb_p3d(&self) -> Result<i32, Failure> {
        Ok(ApproxIntMultiLine::nb_p3d(self))
    }
    fn value_3d(&self, index: i32, points: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        ApproxIntMultiLine::value_3d(self, index, points)
    }
    fn value_2d(&self, index: i32, points2d: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        ApproxIntMultiLine::value_2d(self, index, points2d)
    }
    fn value(
        &self,
        index: i32,
        points: &mut Array1<'_, Point3>,
        points2d: &mut Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        ApproxIntMultiLine::value(self, index, points, points2d)
    }
    fn tangency_3d(&self, index: i32, tangents: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        ApproxIntMultiLine::tangency_3d(self, index, tangents)
    }
    fn tangency_2d(
        &self,
        index: i32,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        ApproxIntMultiLine::tangency_2d(self, index, tangents2d)
    }
    fn tangency(
        &self,
        index: i32,
        tangents: &mut Array1<'_, Vector3>,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        ApproxIntMultiLine::tangency(self, index, tangents, tangents2d)
    }
    // The three `Curvature` members of the tool answer false: the line has no
    // curvature.
    fn curvature_3d(&self, _: i32, _: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        Ok(false)
    }
    fn curvature_2d(&self, _: i32, _: &mut Array1<'_, Vector2>) -> Result<bool, Failure> {
        Ok(false)
    }
    fn curvature(
        &self,
        _: i32,
        _: &mut Array1<'_, Vector3>,
        _: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        Ok(false)
    }
    // `WhatStatus(ML, I1, I2)` ignores the indices.
    fn what_status(&self, _: i32, _: i32) -> Result<Status, Failure> {
        Ok(ApproxIntMultiLine::what_status(self))
    }
    fn make_ml_between(&self, i1: i32, i2: i32, nb_p_min: i32) -> Result<Self, Failure> {
        ApproxIntMultiLine::make_ml_between(self, i1, i2, nb_p_min)
    }
    fn make_ml_one_more_point(
        &self,
        i1: i32,
        i2: i32,
        index_bad: i32,
    ) -> Result<Option<Self>, Failure> {
        ApproxIntMultiLine::make_ml_one_more_point(self, i1, i2, index_bad)
    }
}
