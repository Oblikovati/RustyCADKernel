//! HLRBRep_Data.cxx: Classify, SimplClassify, Compare, HidingStartLevel and the
//! local 2D geometry of the current and face edges.
use super::update::null_wire;
use super::{Body, Data, box_rejected};
use crate::hlr_algo::{
    Interference, MinMaxIndices, encode_min_max, enlarge_min_max, init_min_max, update_min_max,
};
use crate::hlr_brep::{CLProps, Curve, EdgeData};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkgeomalgo::geom_int::adjust_periodic;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Point2};

/// The tangent, normal and curvature of a projected curve at a parameter
/// (`gp_Dir2d& Tg, gp_Dir2d& Nm, double& Cu`).
pub type LocalGeometry2D = (Direction2, Direction2, f64);

fn local_geometry_2d(c: &Curve, param: f64) -> Result<LocalGeometry2D, Failure> {
    let mut props = CLProps::new(c, 2, epsilon(1.))?;
    props.set_parameter(param)?;
    if !props.is_tangent_defined()? {
        return Err(Failure::with_type(
            "Standard_Failure",
            "HLRBRep_Data::LocalGeometry2D",
            "",
        ));
    }
    let tg = props.tangent()?;
    let cu = props.curvature()?;
    let nm = if cu > epsilon(1.) && !precision::is_infinite(cu) {
        props.normal()?
    } else {
        Direction2::new(-tg.y(), tg.x())?
    };
    Ok((tg, nm, cu))
}

impl Data {
    /// LocalLEGeometry2D(Param, Tg, Nm, Cu): the 2D tangent, normal and curvature of
    /// the current edge (as loaded by InitInterference);
    /// `let (tg, nm, cu) = ds.local_le_geometry_2d(0.5)?;`.
    pub fn local_le_geometry_2d(&self, param: f64) -> Result<LocalGeometry2D, Failure> {
        self.body(|b| local_geometry_2d(b.edge(b.w.ll_curve)?.geometry(), param))
    }

    /// LocalFEGeometry2D(FE, Param, Tg, Nm, Cu): the same on the edge `fe`;
    /// `let (tg, nm, cu) = ds.local_fe_geometry_2d(3, 0.5)?;`.
    pub fn local_fe_geometry_2d(&self, fe: i32, param: f64) -> Result<LocalGeometry2D, Failure> {
        self.body(|b| local_geometry_2d(b.edge(fe)?.geometry(), param))
    }

    /// Compare(E, ED): the state of the edge `e` against the hiding face (IN when a
    /// point of it is behind the face). The native `ED` is the record `e` at every
    /// caller; the port reads it; `ds.compare(3)?`.
    pub fn compare(&self, e: i32) -> Result<State, Failure> {
        self.body(|b| Ok(b.classify(e, false, 0.)?.0))
    }

    /// Classify(E, ED, LevelFlag, Level, param): the state of the edge `e` (at
    /// `param` with `level_flag`, else at three of its points) against the hiding
    /// face, and the number of face layers in front of it (counted with
    /// `level_flag`); `let (state, level) = ds.classify(3, true, 0.5)?;`.
    pub fn classify(&self, e: i32, level_flag: bool, param: f64) -> Result<(State, i32), Failure> {
        self.body(|b| b.classify(e, level_flag, param))
    }

    /// SimplClassify(E, ED, Nbp, p1, p2): OUT when one of `nbp` points between `p1`
    /// and `p2` of the edge `e` is outside the face box, else IN;
    /// `ds.simpl_classify(3, 5, 0.0, 1.0)?`.
    pub fn simpl_classify(&self, e: i32, nbp: i32, p1: f64, p2: f64) -> Result<State, Failure> {
        self.body(|b| {
            let ed = b.edge(e)?;
            let dp = (p2 - p1) / f64::from(nbp + 1);
            let mut sta = p1 + dp;
            for _ in 1..=nbp {
                if b.point_rejected(ed, sta)? {
                    return Ok(State::Out);
                }
                sta += dp;
            }
            Ok(State::In)
        })
    }

    /// HidingStartLevel(E, ED, IL): the number of face layers in front of the edge
    /// `e` at the start, from its level in the middle of the first visible span and
    /// the interferences `il` before it; `ds.hiding_start_level(3, &list)?`.
    pub fn hiding_start_level(&self, e: i32, il: &List<Interference>) -> Result<i32, Failure> {
        self.body(|b| {
            let ec = b.edge(e)?.geometry();
            let mut sta = ec.parameter_3d(ec.first_parameter()?)?;
            let mut end = ec.parameter_3d(ec.last_parameter()?)?;
            let tolpar = (end - sta) * 0.01;
            for int in il.iter() {
                let param = int.intersection().parameter();
                if param > end {
                    break;
                }
                if (param - sta).abs() > (param - end).abs() {
                    end = param;
                } else {
                    sta = param;
                }
            }
            let param = 0.5 * (sta + end);
            let (_, mut level) = b.classify(e, true, param)?;
            for int in il.iter() {
                let p = int.intersection().parameter();
                if p < param - tolpar {
                    match int.transition() {
                        Orientation::Forward => level -= int.intersection().level(),
                        Orientation::Reversed => level += int.intersection().level(),
                        _ => {}
                    }
                } else if p > param + tolpar {
                    break;
                }
            }
            Ok(level)
        })
    }
}

impl Body<'_> {
    // Whether the box of the projected point at `sta` (enlarged by the edge
    // tolerance) misses the box of the hiding face.
    fn point_rejected(&self, ed: &EdgeData, sta: f64) -> Result<bool, Failure> {
        let (x, y, z) = self.w.proj.project_xyz(ed.geometry().value_3d(sta)?);
        let (mut tot_min, mut tot_max) = ([0.; 16], [0.; 16]);
        init_min_max(precision::infinite(), &mut tot_min, &mut tot_max);
        update_min_max(x, y, z, &mut tot_min, &mut tot_max);
        enlarge_min_max(f64::from(ed.tolerance()), &mut tot_min, &mut tot_max);
        let (vert_min, vert_max) = self.w.scaled_box(&tot_min, &tot_max);
        let mut min_max_vert = MinMaxIndices::default();
        encode_min_max(&vert_min, &vert_max, &mut min_max_vert);
        Ok(box_rejected(&self.face_min_max()?, &min_max_vert))
    }

    pub(super) fn classify(
        &mut self,
        e: i32,
        level_flag: bool,
        param: f64,
    ) -> Result<(State, i32), Failure> {
        let ed = self
            .e
            .value(e)
            .map_err(super::data_array("HLRBRep_Data::Classify"))?;
        let ec = ed.geometry();
        let sta = if level_flag {
            if self.point_rejected(ed, param)? {
                return Ok((State::Out, 0));
            }
            param
        } else {
            let sta = ec.parameter_3d(ec.first_parameter()?)?;
            if self.point_rejected(ed, sta)? {
                return Ok((State::Out, 0));
            }
            let end = ec.parameter_3d(ec.last_parameter()?)?;
            if self.point_rejected(ed, end)? {
                return Ok((State::Out, 0));
            }
            let sta = 0.4 * sta + 0.6 * end; // dangerous if it is the middle
            if self.point_rejected(ed, sta)? {
                return Ok((State::Out, 0));
            }
            sta
        };
        let psta = ec.value(sta)?;
        let p_lim = ec.value_3d(sta)?;
        let l = self.w.proj.shoot(psta.x(), psta.y())?;
        let w_lim = elclib::line_parameter(l.position(), p_lim);
        let loaded = self.w.loaded_face.ok_or_else(null_wire)?;
        let s = self
            .f
            .value(loaded)
            .map_err(super::data_array("HLRBRep_Data::Classify"))?
            .geometry();
        self.w.intersector.perform_line(s, &l, w_lim)?;
        self.count_layers(level_flag, w_lim)
    }

    // The sight line points before `w_lim` (less a Z tolerance) that the face
    // classifier finds on the face.
    fn count_layers(&self, level_flag: bool, mut w_lim: f64) -> Result<(State, i32), Failure> {
        let (mut state, mut level) = (State::Out, 0);
        let inter = &self.w.intersector;
        if !inter.is_done()? {
            return Ok((state, level));
        }
        let nb_points = inter.nb_points()?;
        if nb_points == 0 {
            return Ok((state, level));
        }
        let big = self.w.big_size;
        let tol_z = match (self.w.i_face_test, self.w.le_out_line || self.w.le_internal) {
            (false, _) => big * 0.000001,
            (true, false) => big * 0.001,
            (true, true) => big * 0.01,
        };
        w_lim -= tol_z;
        let g = self.face_geom()?;
        let u_range = if g.is_u_periodic()? {
            Some((g.u_period()?, g.first_u_parameter()?, g.last_u_parameter()?))
        } else {
            None
        };
        let v_range = if g.is_v_periodic()? {
            Some((g.v_period()?, g.first_v_parameter()?, g.last_v_parameter()?))
        } else {
            None
        };
        let classifier = self.w.classifier.get().ok_or_else(null_classifier)?;
        for i in 1..=nb_points {
            let (_, mut u, mut v, w, _) = inter.cs_point(i)?.values();
            if w >= w_lim {
                continue;
            }
            if let Some((period, lo, hi)) = u_range.filter(|r| r.0 > 0.) {
                u = adjust_periodic(u, lo, hi, period, 0.).1;
            }
            if let Some((period, lo, hi)) = v_range.filter(|r| r.0 > 0.) {
                v = adjust_periodic(v, lo, hi, period, 0.).1;
            }
            if classifier.classify(Point2::new(u, v), precision::p_confusion(), true)? != State::Out
            {
                state = State::In;
                level += 1;
                if !level_flag {
                    return Ok((state, level));
                }
            }
        }
        Ok((state, level))
    }
}

fn null_classifier() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "HLRBRep_Data::Classify: no face classifier; expected InitEdge",
    )
}
