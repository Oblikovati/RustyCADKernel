//! `PrsDim_IdenticRelation` between two edges: the dispatch on lines, circles and ellipses,
//! and the presentation of two collinear lines.
use super::identic_relation::PrsDimIdenticRelation;
use super::relation_support::null_object;
use super::*;
use crate::dsg_prs::DsgPrsIdenticPresentation;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkbrep::topods;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkernel::tcollection::ExtendedString;
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomEllipse, GeomLine};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dStructureMembers as _, Graphic3dStructureSource};

fn is_type(
    curve: &Handle<dyn GeomCurve>,
    kind: &'static rustycad_tkernel::standard::RuntimeType,
) -> bool {
    curve.get().is_some_and(|c| c.is_instance(kind))
}

/// The file-local `PrsDim_Sort`: a bubble sort of four parameters with their points and
/// curve numbers.
fn sort4(parameters: &mut [f64; 4], points: &mut [Point3; 4], curves: &mut [i32; 4]) {
    let mut found = true;
    while found {
        found = false;
        for i in 0..3 {
            if parameters[i + 1] < parameters[i] {
                found = true;
                parameters.swap(i, i + 1);
                points.swap(i, i + 1);
                curves.swap(i, i + 1);
            }
        }
    }
}

impl PrsDimIdenticRelation {
    /// `ComputeTwoEdgesPresentation`: two lines, circles or ellipses on the plane, with the
    /// projection of the edge off it.
    /// Example: `relation.compute_two_edges_presentation(&prs)?`.
    pub fn compute_two_edges_presentation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
    ) -> Result<(), Failure> {
        let first = topods::edge(&self.first_shape())?;
        let second = topods::edge(&self.second_shape())?;
        let Some((done, edges)) = PrsDim::compute_two_edges_ext(&first, &second, &self.plane())?
        else {
            return Ok(());
        };
        self.set_ext_shape(edges.ext_index);
        if !done {
            return Ok(());
        }
        let (inf1, inf2) = (edges.first_infinite, edges.second_infinite);
        prs.get()
            .ok_or_else(null_object)?
            .set_infinite_state((inf1 || inf2) && self.ext_shape() != 0);
        let (c1, c2) = (&edges.first_curve, &edges.second_curve);
        let (mut ends1, mut ends2) = (edges.first_ends, edges.second_ends);
        if is_type(c1, GeomLine::static_type()) && is_type(c2, GeomLine::static_type()) {
            let line = if inf1 && !inf2 { c2 } else { c1 };
            let lin = line.cast::<GeomLine>().get().ok_or_else(null_object)?.lin();
            self.compute_two_lines_presentation(prs, &lin, &mut ends1, &mut ends2, inf1, inf2)?;
        } else if is_type(c1, GeomCircle::static_type()) && is_type(c2, GeomCircle::static_type()) {
            self.fields.write().is_circle = true;
            let circle = c1
                .cast::<GeomCircle>()
                .get()
                .ok_or_else(null_object)?
                .circ()?;
            self.add_circle_label(prs, &circle, (ends1, ends2))?;
        } else if is_type(c1, GeomEllipse::static_type()) && is_type(c2, GeomEllipse::static_type())
        {
            let ellipse = c1
                .cast::<GeomEllipse>()
                .get()
                .ok_or_else(null_object)?
                .elips()?;
            self.add_ellipse_label(prs, &ellipse, (ends1, ends2))?;
        } else {
            return Ok(());
        }
        let ext = self.ext_shape();
        if ext == 0 || edges.ext_curve.is_null() {
            return Ok(());
        }
        let style = PrsDimProjectionStyle::default();
        if ext == 1 {
            self.compute_proj_edge_presentation(prs, &first, c1, ends1.0, ends1.1, &style)
        } else {
            self.compute_proj_edge_presentation(prs, &second, c2, ends2.0, ends2.1, &style)
        }
    }

    /// `ComputeTwoLinesPresentation`: the identity mark of two collinear lines (infinite ones
    /// at a point, else on their common part, their junction or the included segment).
    /// Example: `relation.compute_two_lines_presentation(&prs, &line, &mut e1, &mut e2, false, false)?`.
    pub fn compute_two_lines_presentation(
        &self,
        prs: &Handle<dyn Graphic3dStructureSource>,
        line: &Line3,
        ends1: &mut (Point3, Point3),
        ends2: &mut (Point3, Point3),
        infinite1: bool,
        infinite2: bool,
    ) -> Result<(), Failure> {
        let text = ExtendedString::from_utf8(" ==");
        let drawer = self.attributes();
        if infinite1 && infinite2 {
            let attach = if self.automatic_position() {
                let attach = line.location();
                let pln = self.plane().get().ok_or_else(null_object)?.pln();
                let shift = Vector3::from(pln.x_axis().direction()) * self.arrow_size();
                let mut r = self.relation_base().fields.write();
                r.position = attach.translated(shift);
                r.automatic_position = true;
                attach
            } else {
                elclib::value(elclib::parameter(line, self.position())?, line)
            };
            {
                let mut f = self.fields.write();
                f.f_attach = attach;
                f.s_attach = attach;
            }
            return DsgPrsIdenticPresentation::add(prs, &drawer, &text, &attach, &self.position());
        }
        let mut pf1 = elclib::parameter(line, ends1.0)?;
        let mut pl1 = elclib::parameter(line, ends1.1)?;
        let mut pf2 = elclib::parameter(line, ends2.0)?;
        let mut pl2 = elclib::parameter(line, ends2.1)?;
        if infinite1 {
            (pf1, pl1) = (pf2, pl2);
            *ends1 = *ends2;
        } else if infinite2 {
            (pf2, pl2) = (pf1, pl1);
            *ends2 = *ends1;
        }
        let mut parameters = [pf1, pf2, pl1, pl2];
        let mut points = [ends1.0, ends2.0, ends1.1, ends2.1];
        let mut curves = [1, 2, 1, 2];
        sort4(&mut parameters, &mut points, &mut curves);
        let half = |a: Point3, b: Point3| Point3::from((a.xyz() + b.xyz()) / 2.0);
        let (f_attach, s_attach, middle);
        if parameters[0] == parameters[1] && parameters[2] == parameters[3] {
            middle = half(points[1], points[2]);
            let mid_par = (parameters[1] + parameters[2]) / 2.0;
            let delta = (parameters[3] - parameters[0]) / 5.0;
            f_attach = elclib::value(mid_par - delta, line);
            s_attach = elclib::value(mid_par + delta, line);
        } else if parameters[1] == parameters[2] {
            middle = points[1];
            let mut delta = parameters[1] - parameters[0];
            let delta2 = parameters[3] - parameters[2];
            if delta > delta2 {
                delta = delta2;
            }
            f_attach = elclib::value(parameters[1] - delta / 2.0, line);
            s_attach = elclib::value(parameters[1] + delta / 2.0, line);
        } else if (curves[0] == curves[1] && parameters[1] != parameters[2])
            || (curves[0] != curves[1] && curves[1] != curves[2] && parameters[1] != parameters[2])
        {
            middle = half(points[1], points[2]);
            f_attach = points[1];
            s_attach = points[2];
        } else {
            f_attach = half(points[0], points[1]);
            s_attach = half(points[1], points[2]);
            middle = half(f_attach, s_attach);
        }
        {
            let mut f = self.fields.write();
            f.f_attach = f_attach;
            f.s_attach = s_attach;
        }
        let position = if self.automatic_position() {
            let normal = self
                .plane()
                .get()
                .ok_or_else(null_object)?
                .pln()
                .axis()
                .direction();
            let across = Vector3::between(f_attach, s_attach)
                .normalized()?
                .crossed(Vector3::from(normal))
                * self.compute_seg_size();
            let position = middle.translated(across);
            let mut r = self.relation_base().fields.write();
            r.position = position;
            r.automatic_position = true;
            position
        } else {
            let position = self.dragged_line_position(line, f_attach, s_attach)?;
            self.set_position(&position);
            position
        };
        DsgPrsIdenticPresentation::add_segment(prs, &drawer, &text, &f_attach, &s_attach, &position)
    }

    /// The dragged position kept between the attach points along the line.
    fn dragged_line_position(
        &self,
        line: &Line3,
        f_attach: Point3,
        s_attach: Point3,
    ) -> Result<Point3, Failure> {
        let mut current = self.position();
        let mut parameter = elclib::parameter(line, current)?;
        let distance = line.distance(current);
        let projected = elclib::value(parameter, line);
        let confusion = precision::confusion();
        let mut trans = Vector3::default();
        if distance >= confusion {
            trans = Vector3::between(projected, current).normalized()?;
        }
        let first = elclib::parameter(line, f_attach)?;
        let last = elclib::parameter(line, s_attach)?;
        let clamped = if parameter <= first {
            Some(first + 1e-5)
        } else if parameter >= last {
            Some(last - 1e-5)
        } else {
            None
        };
        if let Some(value) = clamped {
            parameter = value;
            current = elclib::value(parameter, line);
            if distance >= confusion {
                current = current.translated(trans * distance);
            }
        }
        Ok(current)
    }
}
