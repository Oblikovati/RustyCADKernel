// Rust translation of ShapeAnalysis_CanonicalRecognition conic recognition at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::params::{compare_conic_params, set_conic_parameters};
use super::{ShapeAnalysisCanonicalRecognition, surfaces};
use rustycad_tkbrep::topods::{self, Edge, ShapeIterator};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_convert::ConvType;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Axis2, Circle3, Ellipse3, Line3};
impl ShapeAnalysisCanonicalRecognition {
    /// The private `IsConic(theTarget, theTol, thePos, theParams)`: the conic of an edge,
    /// or of every edge of a wire (each compared with the first one).
    fn is_conic(
        &mut self,
        target: CurveType,
        tol: f64,
        pos: &mut Axis2,
        params: &mut [f64],
    ) -> Result<bool, Failure> {
        if self.status != 0 {
            return Ok(false);
        }
        match self.s_type {
            ShapeEnum::Edge => {
                self.edge_conic_parameters(&topods::edge(&self.shape)?, target, tol, pos, params)
            }
            ShapeEnum::Wire => self.is_wire_conic(target, tol, pos, params),
            _ => {
                self.status = 1;
                Ok(false)
            }
        }
    }

    /// A failed conversion keeps its status; a parameter type mismatch sets status 1.
    fn edge_conic_parameters(
        &mut self,
        edge: &Edge,
        target: CurveType,
        tol: f64,
        pos: &mut Axis2,
        params: &mut [f64],
    ) -> Result<bool, Failure> {
        let conic = surfaces::edge_curve(
            edge,
            tol,
            ConvType::Target,
            target,
            &mut self.gap,
            &mut self.status,
        )?;
        if conic.is_null() {
            return Ok(false);
        }
        let found = set_conic_parameters(target, &conic, pos, params)?;
        if !found {
            self.status = 1;
        }
        Ok(found)
    }

    /// The wire branch of `IsConic`.
    fn is_wire_conic(
        &mut self,
        target: CurveType,
        tol: f64,
        pos: &mut Axis2,
        params: &mut [f64],
    ) -> Result<bool, Failure> {
        let mut it = ShapeIterator::new(&self.shape, true, true);
        if !it.more() {
            self.status = 1;
            return Ok(false);
        }
        let first = topods::edge(it.value()?)?;
        if !self.edge_conic_parameters(&first, target, tol, pos, params)? {
            return Ok(false);
        }
        it.next();
        let mut other_pos = Axis2::default();
        let mut other_params = vec![0.0; params.len()];
        while it.more() {
            let edge = topods::edge(it.value()?)?;
            let conic = surfaces::edge_curve(
                &edge,
                tol,
                ConvType::Target,
                target,
                &mut self.gap,
                &mut self.status,
            )?;
            if conic.is_null() {
                return Ok(false);
            }
            // The native result of SetConicParameters is overwritten by the comparison.
            set_conic_parameters(target, &conic, &mut other_pos, &mut other_params)?;
            if !compare_conic_params(target, tol, pos, params, &other_pos, &other_params)? {
                return Ok(false);
            }
            it.next();
        }
        Ok(true)
    }

    /// `IsLine(theTol, theLin)`: the line of the edge or of every edge of the wire within
    /// `tol`; `recognizer.is_line(1e-7, &mut line)?`.
    pub fn is_line(&mut self, tol: f64, lin: &mut Line3) -> Result<bool, Failure> {
        let mut pos = Axis2::default();
        let mut params = [0.0; 1];
        let found = self.is_conic(CurveType::Line, tol, &mut pos, &mut params)?;
        if found {
            lin.set_position(pos.axis());
        }
        Ok(found)
    }

    /// `IsCircle(theTol, theCirc)`: the circle of the edge or of every edge of the wire
    /// within `tol`; `recognizer.is_circle(1e-7, &mut circle)?`.
    pub fn is_circle(&mut self, tol: f64, circ: &mut Circle3) -> Result<bool, Failure> {
        let mut pos = Axis2::default();
        let mut params = [0.0; 1];
        let found = self.is_conic(CurveType::Circle, tol, &mut pos, &mut params)?;
        if found {
            circ.set_position(pos);
            circ.set_radius(params[0])?;
        }
        Ok(found)
    }

    /// `IsEllipse(theTol, theElips)`: the ellipse of the edge or of every edge of the wire
    /// within `tol`; `recognizer.is_ellipse(1e-7, &mut ellipse)?`.
    pub fn is_ellipse(&mut self, tol: f64, elips: &mut Ellipse3) -> Result<bool, Failure> {
        let mut pos = Axis2::default();
        let mut params = [0.0; 2];
        let found = self.is_conic(CurveType::Ellipse, tol, &mut pos, &mut params)?;
        if found {
            elips.set_position(pos);
            elips.set_major_radius(params[0])?;
            elips.set_minor_radius(params[1])?;
        }
        Ok(found)
    }
}
