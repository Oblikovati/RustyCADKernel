//! HLRBRep_Data.cxx: Update, the boxes and flags of the edges and faces seen through
//! a projector.
use super::{Body, Data, Walk, data_array};
use crate::hlr_algo::{
    MinMaxIndices, Projector, add_min_max, copy_min_max, encode_min_max, enlarge_min_max,
    failures::c_int_truncation, init_min_max, size_box,
};
use crate::hlr_brep::{EdgeData, EdgeFaceTool, FaceData, SLProps, Surface};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
use std::sync::Arc;

const MEMBER: &str = "HLRBRep_Data::Update";

/// `myToler * 10`: a float product, widened where it meets a double.
pub(super) fn toler10(toler: f32) -> f64 {
    f64::from(toler * 10.)
}

/// The view criterion of a normal `nm` at `pt` (both in the view frame): its Z, or
/// in perspective the side of the eye at the focus.
pub(super) fn view_side(proj: &Projector, nm: Vector3, pt: Point3) -> Result<f64, Failure> {
    if proj.perspective() {
        Ok(nm.z() * proj.focus()? - (nm.x() * pt.x() + nm.y() * pt.y() + nm.z() * pt.z()))
    } else {
        Ok(nm.z())
    }
}

/// The surface properties of order 2 at the native resolution `Epsilon(1.)`
/// (`mySLProps`).
pub(super) fn sl_props(s: &Surface) -> Result<SLProps<'_>, Failure> {
    SLProps::new(s, 2, epsilon(1.))
}

impl Walk {
    // REJECT1: the integer box of a real box, scaled by `myDeca` and `mySurD`.
    pub(super) fn scaled_box(
        &self,
        tot_min: &[f64; 16],
        tot_max: &[f64; 16],
    ) -> (MinMaxIndices, MinMaxIndices) {
        let (mut lo, mut hi) = (MinMaxIndices::default(), MinMaxIndices::default());
        let scale =
            |k: usize, t: &[f64; 16]| c_int_truncation((self.deca[k] + t[k]) * self.sur_d[k]);
        for k in 0..8 {
            lo.min[k] = scale(k, tot_min);
            hi.min[k] = scale(k, tot_max);
            lo.max[k] = scale(k + 8, tot_min);
            hi.max[k] = scale(k + 8, tot_max);
        }
        (lo, hi)
    }
}

impl Data {
    /// Update(P): shares `p` with every curve and surface, computes the global box
    /// and its integer scaling, then the box, vertical flags and simple flags of
    /// every edge, and the flags, orientation of the outlines, back/hiding state and
    /// wire boxes of every face; `ds.update(&projector)?`.
    pub fn update(&self, p: &Projector) -> Result<(), Failure> {
        self.body(|b| {
            b.w.proj = Arc::new(*p);
            b.global_box()?;
            for edge in 1..=b.e.upper() {
                b.update_edge(edge)?;
            }
            for face in 1..=b.f.upper() {
                b.update_face(face)?;
            }
            Ok(())
        })
    }
}

impl Body<'_> {
    pub(super) fn edge(&self, i: i32) -> Result<&EdgeData, Failure> {
        self.e.value(i).map_err(data_array("HLRBRep_Data"))
    }

    pub(super) fn edge_mut(&mut self, i: i32) -> Result<&mut EdgeData, Failure> {
        self.e.change_value(i).map_err(data_array("HLRBRep_Data"))
    }

    pub(super) fn face(&self, i: i32) -> Result<&FaceData, Failure> {
        self.f.value(i).map_err(data_array("HLRBRep_Data"))
    }

    pub(super) fn face_mut(&mut self, i: i32) -> Result<&mut FaceData, Failure> {
        self.f.change_value(i).map_err(data_array("HLRBRep_Data"))
    }

    // `iFaceGeom`.
    pub(super) fn face_geom(&self) -> Result<&Surface, Failure> {
        let i = self.w.i_face_geom.ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_Data: no face surface; expected Update or InitEdge",
            )
        })?;
        Ok(self.face(i)?.geometry())
    }

    // The global box of the projected edges and the integer scaling of the boxes.
    fn global_box(&mut self) -> Result<(), Failure> {
        let (mut tot_min, mut tot_max) = ([0.; 16], [0.; 16]);
        init_min_max(precision::infinite(), &mut tot_min, &mut tot_max);
        let mut tol_min_max = 0.;
        for edge in 1..=self.e.upper() {
            let proj = self.w.proj.clone();
            let ec = self.edge_mut(edge)?.change_geometry();
            ec.set_projector(Some(proj));
            let enl = ec.update(&mut tot_min, &mut tot_max)?;
            if enl > tol_min_max {
                tol_min_max = enl;
            }
        }
        enlarge_min_max(tol_min_max, &mut tot_min, &mut tot_max);
        let mut d = [0.; 16];
        let mut precad = -precision::infinite();
        for i in 0..16 {
            d[i] = tot_max[i] - tot_min[i];
            if precad < d[i] {
                precad = d[i];
            }
        }
        self.w.big_size = precad;
        precad *= 0.0005;
        for (s, di) in self.w.sur_d.iter_mut().zip(d) {
            *s = f64::from(0x0000_7fff) / (di + precad);
        }
        precad *= 0.5;
        for (deca, t) in self.w.deca.iter_mut().zip(tot_min) {
            *deca = -t + precad;
        }
        Ok(())
    }

    // The box, vertical flags and simple flags of one edge.
    fn update_edge(&mut self, edge: i32) -> Result<(), Failure> {
        let (mut tot_min, mut tot_max) = ([0.; 16], [0.; 16]);
        init_min_max(precision::infinite(), &mut tot_min, &mut tot_max);
        let ed = self.edge(edge)?;
        let tol_min_max = ed.geometry().update_min_max(&mut tot_min, &mut tot_max)?;
        let tol = f64::from(ed.tolerance());
        let vertical = (0..7).all(|k| tot_max[k] - tot_min[k] < tol);
        self.edge_mut(edge)?.set_vertical(vertical);
        enlarge_min_max(tol_min_max, &mut tot_min, &mut tot_max);
        let (edge_min, edge_max) = self.w.scaled_box(&tot_min, &tot_max);
        let mut min_max_edge = MinMaxIndices::default();
        encode_min_max(&edge_min, &edge_max, &mut min_max_edge);
        self.edge_mut(edge)?.update_min_max(&min_max_edge);
        let (ver1, ver2) = if vertical {
            self.merge_vertical_ends(edge)?;
            (true, true)
        } else {
            self.vertical_ends(edge)?
        };
        let ed = self.edge_mut(edge)?;
        ed.set_ver_at_sta(ed.vertical() || ver1);
        ed.set_ver_at_end(ed.vertical() || ver2);
        ed.set_auto_intersection_done(true);
        ed.set_simple(true);
        Ok(())
    }

    // A vertical edge (seen as a point): the edges at its start vertex take its end
    // vertex, with its outline and cut flags.
    fn merge_vertical_ends(&mut self, edge: i32) -> Result<(), Failure> {
        let ed = self.edge(edge)?;
        let (vsta, vend) = (ed.v_sta(), ed.v_end());
        let vout = ed.out_lv_sta() || ed.out_lv_end();
        let vcut = ed.cut_at_sta() || ed.cut_at_end();
        for ebis in 1..=self.e.upper() {
            let eb = self.edge_mut(ebis)?;
            if vsta == eb.v_sta() {
                eb.set_v_sta(vend);
                eb.set_out_lv_sta(vout);
                eb.set_cut_at_sta(vcut);
            } else if vsta == eb.v_end() {
                eb.set_v_end(vend);
                eb.set_out_lv_end(vout);
                eb.set_cut_at_end(vcut);
            }
        }
        Ok(())
    }

    // Whether the tangents at the ends are along the sight.
    fn vertical_ends(&self, edge: i32) -> Result<(bool, bool), Failure> {
        let ec = self.edge(edge)?.geometry();
        let t = self.w.proj.transformation();
        let lim = toler10(self.w.toler);
        let (_, mut tg1) = ec.d1_3d(ec.parameter_3d(ec.first_parameter()?)?)?;
        let (_, mut tg2) = ec.d1_3d(ec.parameter_3d(ec.last_parameter()?)?)?;
        tg1.transform(t);
        tg2.transform(t);
        let along = |tg: Vector3| -> Result<bool, Failure> {
            if tg.x().abs() + tg.y().abs() < lim {
                return Ok(true);
            }
            let dir = Direction3::from_vector(tg)?;
            Ok(dir.x().abs() + dir.y().abs() < lim)
        };
        Ok((along(tg1)?, along(tg2)?))
    }

    // The flags, orientation, back/hiding state and wire boxes of one face.
    fn update_face(&mut self, face: i32) -> Result<(), Failure> {
        self.w.i_face_geom = Some(face);
        let proj = self.w.proj.clone();
        self.face_mut(face)?
            .geometry_mut()
            .set_projector(Some(proj));
        self.w.i_face_type = self.face(face)?.geometry().get_type();
        self.cut_flags(face)?;
        let toler = self.w.toler;
        let fd = self.face(face)?;
        let side = fd
            .geometry()
            .is_side(f64::from(fd.tolerance()), toler10(toler))?;
        self.face_mut(face)?.set_side(side);
        let mut inverted = false;
        let fd = self.face(face)?;
        if fd.with_out_l() && !fd.side() {
            inverted = self.orient_out_line(face)?;
            self.orient_oth_edge(face)?;
        }
        let fd = self.face(face)?;
        if fd.side() {
            let fd = self.face_mut(face)?;
            fd.set_hiding(false);
            fd.set_back(false);
        } else if !fd.with_out_l() {
            self.back_and_hiding(face)?;
        } else {
            let fd = self.face_mut(face)?;
            fd.set_hiding(!inverted);
            fd.set_back(inverted);
        }
        self.wire_boxes(face)
    }

    // Whether the face is cut by an outline, simple, and of an elementary type.
    fn cut_flags(&mut self, face: i32) -> Result<(), Failure> {
        let (mut cut, mut with_out_l) = (false, false);
        let itr = &mut self.w.face_itr1;
        itr.init_edge(self.f.value(face).map_err(data_array(MEMBER))?)?;
        while itr.more_edge() {
            if itr.internal()? {
                with_out_l = true;
                cut = true;
            } else if itr.out_line()? {
                with_out_l = true;
                if itr.double()? {
                    cut = true;
                }
            }
            itr.next_edge()?;
        }
        let ty = self.w.i_face_type;
        let fd = self.face_mut(face)?;
        fd.set_cut(cut);
        fd.set_with_out_l(with_out_l);
        let elementary = matches!(
            ty,
            SurfaceType::Plane
                | SurfaceType::Cylinder
                | SurfaceType::Cone
                | SurfaceType::Sphere
                | SurfaceType::Torus
        );
        fd.set_simple(!with_out_l && elementary);
        fd.set_plane(ty == SurfaceType::Plane);
        fd.set_cylinder(ty == SurfaceType::Cylinder);
        fd.set_cone(ty == SurfaceType::Cone);
        fd.set_sphere(ty == SurfaceType::Sphere);
        fd.set_torus(ty == SurfaceType::Torus);
        Ok(())
    }

    // A face without outline: back when its normal at the middle of a boundary edge
    // looks away; a side face when no edge decides; hiding from the orientation.
    fn back_and_hiding(&mut self, face: i32) -> Result<(), Failure> {
        self.face_mut(face)?.set_back(false);
        let found = self.find_back(face)?;
        let fd = self.face_mut(face)?;
        if !found {
            fd.set_side(true);
            fd.set_hiding(false);
            fd.set_back(false);
        } else if fd.closed() {
            let hiding = match fd.orientation() {
                Orientation::Reversed => fd.back(),
                Orientation::Forward => !fd.back(),
                Orientation::External => true,
                Orientation::Internal => false,
            };
            fd.set_hiding(hiding);
        } else {
            fd.set_hiding(true);
        }
        Ok(())
    }

    fn find_back(&mut self, face: i32) -> Result<bool, Failure> {
        let fd = self.f.value(face).map_err(data_array(MEMBER))?;
        self.w.face_itr1.init_edge(fd)?;
        while self.w.face_itr1.more_edge() {
            let itr = &self.w.face_itr1;
            self.w.fe = itr.edge()?;
            self.w.fe_ori = itr.orientation()?;
            self.w.fe_out_line = itr.out_line()?;
            self.w.fe_internal = itr.internal()?;
            self.w.fe_double = itr.double()?;
            let boundary = matches!(self.w.fe_ori, Orientation::Forward | Orientation::Reversed);
            if !self.w.fe_double && boundary {
                self.w.fe_geom = self.w.fe;
                if let Some(back) = self.back_at_edge(face, self.w.fe)? {
                    self.face_mut(face)?.set_back(back);
                    return Ok(true);
                }
            }
            self.w.face_itr1.next_edge()?;
        }
        Ok(false)
    }

    // The back flag the normal at the middle of the edge decides, if it does.
    fn back_at_edge(&self, face: i32, fe: i32) -> Result<Option<bool>, Failure> {
        let ec = self.edge(fe)?.geometry();
        let s = self.face(face)?.geometry();
        let p = ec.parameter_3d((ec.last_parameter()? + ec.first_parameter()?) / 2.)?;
        let Some((pu, pv)) = EdgeFaceTool::uv_point(p, ec, s)? else {
            return Ok(None);
        };
        let mut props = sl_props(s)?;
        props.set_parameters(pu, pv)?;
        let mut pt = ec.value_3d(p)?;
        if !props.is_normal_defined()? {
            return Ok(None);
        }
        let mut nm = Vector3::from(props.normal()?);
        let t = self.w.proj.transformation();
        pt.transform(t);
        nm.transform(t);
        let r = view_side(&self.w.proj, nm, pt)?;
        Ok((r.abs() > toler10(self.w.toler)).then_some(r < 0.))
    }

    // The boxes of the wires and of the face, from the boxes of their edges.
    fn wire_boxes(&mut self, face: i32) -> Result<(), Failure> {
        let mut first_time = true;
        let (mut wire_min, mut wire_max) = (MinMaxIndices::default(), MinMaxIndices::default());
        let (mut face_min, mut face_max) = (MinMaxIndices::default(), MinMaxIndices::default());
        let simple = self.face(face)?.simple();
        let fd = self.f.value(face).map_err(data_array(MEMBER))?;
        self.w.face_itr1.init_edge(fd)?;
        while self.w.face_itr1.more_edge() {
            self.w.fe = self.w.face_itr1.edge()?;
            let ed = self.e.change_value(self.w.fe).map_err(data_array(MEMBER))?;
            if !simple {
                ed.set_auto_intersection_done(false);
            }
            let (mut edge_min, mut edge_max) = (MinMaxIndices::default(), MinMaxIndices::default());
            crate::hlr_algo::decode_min_max(ed.min_max(), &mut edge_min, &mut edge_max);
            let itr = &self.w.face_itr1;
            if itr.beginning_of_wire() {
                copy_min_max(&edge_min, &edge_max, &mut wire_min, &mut wire_max);
            } else {
                add_min_max(&edge_min, &edge_max, &mut wire_min, &mut wire_max);
            }
            if itr.end_of_wire() {
                let mut min_max_wire = MinMaxIndices::default();
                encode_min_max(&wire_min, &wire_max, &mut min_max_wire);
                itr.wire()?
                    .get()
                    .ok_or_else(null_wire)?
                    .update_min_max(&min_max_wire);
                if first_time {
                    first_time = false;
                    copy_min_max(&wire_min, &wire_max, &mut face_min, &mut face_max);
                } else {
                    add_min_max(&wire_min, &wire_max, &mut face_min, &mut face_max);
                }
            }
            self.w.face_itr1.next_edge()?;
        }
        let mut min_max_face = MinMaxIndices::default();
        encode_min_max(&face_min, &face_max, &mut min_max_face);
        let fd = self.face_mut(face)?;
        fd.wires()
            .get()
            .ok_or_else(null_wire)?
            .update_min_max(&min_max_face);
        fd.set_size(size_box(&face_min, &face_max));
        Ok(())
    }
}

pub(super) fn null_wire() -> Failure {
    Failure::with_message(
        "Standard_NullObject",
        "HLRBRep_Data: a face without wire blocks; expected FaceData::Set",
    )
}

mod orient;
