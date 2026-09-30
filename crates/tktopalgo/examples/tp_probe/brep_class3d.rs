//! BRepClass3d probe operations over shapes read by `tp_shape`, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepclass3d.cxx. States and transitions are printed as
//! integers, faces and shells as their record index.
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real, xyz};
use super::shape::{TpShape, tp_shape};
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::ncollection::TreeSelector;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction3, Line3, Point3, Vector3};
use rustycad_tktopalgo::brep_class3d::{
    BRepClass3d, BRepClass3dBndBoxTreeSelectorLine, BRepClass3dBndBoxTreeSelectorPoint,
    BRepClass3dIntersector3d, BRepClass3dSClassifier, BRepClass3dSolidClassifier,
    BRepClass3dSolidExplorer, FacePointSample,
};

/// `tp_index_of`: the record index of `x` (IsSame), -1 for a null shape, -2 for another.
fn index_of<S: AsRef<Shape>>(shapes: &[S], x: &Shape) -> i32 {
    if x.is_null() {
        return -1;
    }
    shapes
        .iter()
        .position(|s| s.as_ref().is_same(x))
        .map_or(-2, |k| k as i32)
}

/// `tp_state`: state, rejected, on-a-face flag and face index.
fn state(s: &TpShape, c: &BRepClass3dSClassifier) {
    int(c.state() as i32);
    flag(c.rejected());
    flag(c.is_on_a_face());
    int(index_of(&s.faces, &c.face()));
}

/// `shape mode tol n (x y z)...`: a solid classifier on the points.
pub fn classify_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let mode = r.integer();
        let tol = r.real();
        let n = r.integer();
        if mode == 1 {
            for _ in 0..n {
                let p = r.pnt();
                attempt(|| {
                    let c = BRepClass3dSolidClassifier::with_point(&s.shape, &p, tol)?;
                    state(&s, &c);
                    Ok(())
                });
            }
            return Ok(());
        }
        let mut c = BRepClass3dSolidClassifier::new();
        if mode == 0 || mode == 4 {
            c.load(&s.shape)?;
        } else if mode == 2 {
            c.load(&s.shape)?;
            c.load(&s.shape)?;
        }
        if mode == 4 {
            attempt(|| {
                c.perform_infinite_point(tol)?;
                state(&s, &c);
                Ok(())
            });
        }
        for _ in 0..n {
            let p = r.pnt();
            attempt(|| {
                c.perform(&p, tol)?;
                state(&s, &c);
                Ok(())
            });
        }
        c.destroy();
        attempt(|| {
            c.perform_infinite_point(tol)?;
            state(&s, &c);
            Ok(())
        });
        Ok(())
    });
    end();
}

/// `shape tol mode`: PerformInfinitePoint, then OuterShell.
pub fn infinite_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let tol = r.real();
        if r.integer() == 0 {
            let mut c = BRepClass3dSolidClassifier::with_shape(&s.shape)?;
            attempt(|| {
                c.perform_infinite_point(tol)?;
                state(&s, &c);
                Ok(())
            });
        } else {
            let mut e = BRepClass3dSolidExplorer::with_shape(&s.shape)?;
            let mut c = BRepClass3dSClassifier::new();
            state(&s, &c);
            for _ in 0..2 {
                attempt(|| {
                    c.perform_infinite_point(&mut e, tol)?;
                    state(&s, &c);
                    Ok(())
                });
            }
        }
        if s.shape.shape_type()? == ShapeEnum::Solid {
            attempt(|| {
                let outer = BRepClass3d::outer_shell(&topods::solid(&s.shape)?)?;
                int(index_of(&s.shells, &outer));
                Ok(())
            });
        }
        Ok(())
    });
    end();
}

/// `shape tol n (x y z)...`: the SClassifier(S, P, Tol) constructor on one explorer.
pub fn sclassifier_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let tol = r.real();
        let mut e = BRepClass3dSolidExplorer::with_shape(&s.shape)?;
        for _ in 0..r.integer() {
            let p = r.pnt();
            attempt(|| {
                let c = BRepClass3dSClassifier::with_point(&mut e, &p, tol)?;
                state(&s, &c);
                int(e.get_face_segment_index());
                Ok(())
            });
        }
        Ok(())
    });
    end();
}

/// `shape x y z n`: Segment, then n OtherSegment calls.
pub fn segment_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let p = r.pnt();
        let n = r.integer();
        let mut e = BRepClass3dSolidExplorer::with_shape(&s.shape)?;
        for i in 0..=n {
            let mut l = Line3::new(Point3::new(7.0, 8.0, 9.0), Direction3::new(0.0, 1.0, 0.0)?);
            let mut par = -5.0;
            attempt(|| {
                let flag_value = if i == 0 {
                    e.segment(&p, &mut l, &mut par)?
                } else {
                    e.other_segment(&p, &mut l, &mut par)?
                };
                int(flag_value);
                pnt(l.location());
                dir(l.direction());
                real(par);
                int(e.get_face_segment_index());
                flag(e.reject(&p));
                Ok(())
            });
        }
        Ok(())
    });
    end();
}

/// The shells and faces of the explorer with their intersectors.
fn explore(s: &TpShape, e: &mut BRepClass3dSolidExplorer) -> Result<(), Failure> {
    e.init_shell()?;
    while e.more_shell() {
        int(index_of(&s.shells, &e.current_shell()?.into_shape()));
        flag(e.reject_shell(&Line3::default()));
        e.init_face()?;
        while e.more_face() {
            let f = e.current_face()?;
            int(index_of(&s.faces, &f));
            int(f.orientation() as i32);
            flag(e.reject_face(&Line3::default()));
            attempt(|| {
                let x = e.change_intersector(&f)?;
                int(x.face().orientation() as i32);
                let b = x.bounding();
                flag(b.is_void());
                if !b.is_void() {
                    let limits = b.get()?;
                    real(limits[0]);
                    real(limits[5]);
                }
                Ok(())
            });
            e.next_face()?;
        }
        attempt(|| {
            int(index_of(&s.faces, &e.current_face()?.into_shape()));
            Ok(())
        });
        e.next_shell()?;
    }
    Ok(())
}

/// The explorer members of `tp_bc3_explorer_case`.
pub fn explorer_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let mut e = BRepClass3dSolidExplorer::with_shape(&s.shape)?;
        flag(e.reject(&Point3::default()));
        int(e.get_map_ev().extent() as i32);
        for i in 1..=e.get_map_ev().extent() {
            int(e.get_map_ev().find_key(i)?.shape_type()? as i32);
        }
        flag(e.get_shape().is_same(&s.shape));
        flag(e.get_tree().is_empty());
        explore(&s, &mut e)?;
        attempt(|| {
            int(index_of(&s.shells, &e.current_shell()?.into_shape()));
            Ok(())
        });
        attempt(|| {
            e.init_face()?;
            flag(e.more_face());
            Ok(())
        });
        let mut foreign = Face::default();
        BRepBuilder::default().make_face(&mut foreign);
        attempt(|| e.intersector(&foreign).map(|_| int(0)));
        e.destroy();
        for f in &s.faces {
            attempt(|| e.intersector(f).map(|_| int(1)));
        }
        e.init_shape(&s.shape)?;
        int(e.get_face_segment_index());
        for f in &s.faces {
            attempt(|| e.intersector(f).map(|_| int(2)));
        }
        e.dump_segment(&Point3::default(), &Line3::default(), 1.0, State::In);
        default_explorer()
    });
    end();
}

/// The default explorer of `tp_bc3_explorer_case`: no shape, rejecting after Segment.
fn default_explorer() -> Result<(), Failure> {
    let mut d = BRepClass3dSolidExplorer::new();
    flag(d.reject(&Point3::default()));
    int(d.get_face_segment_index());
    flag(d.get_shape().is_null());
    let mut l = Line3::new(Point3::new(1.0, 2.0, 3.0), Direction3::new(0.0, 0.0, 1.0)?);
    let mut par = -1.0;
    int(d.segment(&Point3::default(), &mut l, &mut par)?);
    real(par);
    pnt(l.location());
    flag(d.reject(&Point3::default()));
    d.init_shell()?;
    flag(d.more_shell());
    Ok(())
}

/// Prints a found flag, a point and its parameters.
fn found_point(found: bool, p: Point3, uv: Option<(f64, f64)>) {
    flag(found);
    pnt(p);
    if let Some((u, v)) = uv {
        real(u);
        real(v);
    }
}

/// `shape face param`: the static FindAPointInTheFace overloads.
pub fn findpoint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let f = s.faces[r.integer() as usize].clone();
        let mut param = r.real();
        let mut sample = FacePointSample {
            point: Point3::new(1.0, 2.0, 3.0),
            u: 4.0,
            v: 5.0,
            d1u: Vector3::new(6.0, 7.0, 8.0),
            d1v: Vector3::new(9.0, 10.0, 11.0),
        };
        attempt(|| {
            let found =
                BRepClass3dSolidExplorer::find_a_point_in_the_face_d1(&f, &mut sample, &mut param)?;
            found_point(found, sample.point, Some((sample.u, sample.v)));
            xyz(*sample.d1u.xyz());
            xyz(*sample.d1v.xyz());
            Ok(())
        });
        let mut q = Point3::new(1.0, 2.0, 3.0);
        attempt(|| {
            let found = BRepClass3dSolidExplorer::find_a_point_in_the_face(&f, &mut q, &mut param)?;
            found_point(found, q, None);
            Ok(())
        });
        let (mut q2, mut u2, mut v2) = (Point3::default(), -1.0, -1.0);
        attempt(|| {
            let found = BRepClass3dSolidExplorer::find_a_point_in_the_face_uv_param(
                &f, &mut q2, &mut u2, &mut v2, &mut param,
            )?;
            found_point(found, q2, Some((u2, v2)));
            Ok(())
        });
        let (mut q3, mut u3, mut v3) = (Point3::default(), -1.0, -1.0);
        attempt(|| {
            let found = BRepClass3dSolidExplorer::find_a_point_in_the_face_uv(
                &f, &mut q3, &mut u3, &mut v3,
            )?;
            found_point(found, q3, Some((u3, v3)));
            Ok(())
        });
        let mut q4 = Point3::default();
        attempt(|| {
            let found = BRepClass3dSolidExplorer::find_a_point_in_the_face_point(&f, &mut q4)?;
            found_point(found, q4, None);
            Ok(())
        });
        let (mut u5, mut v5) = (-1.0, -1.0);
        attempt(|| {
            flag(
                BRepClass3dSolidExplorer::find_a_point_in_the_face_parameters(
                    &f, &mut u5, &mut v5,
                )?,
            );
            real(u5);
            real(v5);
            Ok(())
        });
        Ok(())
    });
    end();
}

/// `shape face u v index n lift`: PointInTheFace n times, then the surface overloads.
pub fn pointinface_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let f = s.faces[r.integer() as usize].clone();
        let (mut u, mut v) = (r.real(), r.real());
        let mut index = r.integer();
        let n = r.integer();
        let lift = r.real();
        let e = BRepClass3dSolidExplorer::with_shape(&s.shape)?;
        let mut param = 0.5;
        let a = BRepAdaptorSurface::with_face(&f, true)?;
        let mut p = Point3::default();
        attempt(|| {
            p = a.value(u, v)?.translated(Vector3::new(0.0, 0.0, lift));
            Ok(())
        });
        for _ in 0..n {
            attempt(|| {
                flag(e.point_in_the_face(&f, &mut p, &mut u, &mut v, &mut param, &mut index)?);
                pnt(p);
                real(u);
                real(v);
                real(param);
                int(index);
                Ok(())
            });
        }
        surface_overloads(&e, &f, (p, u, v), &mut param)
    });
    end();
}

/// The two surface overloads of PointInTheFace of `tp_bc3_pointinface_case`.
fn surface_overloads(
    e: &BRepClass3dSolidExplorer,
    f: &Face,
    (p, u, v): (Point3, f64, f64),
    param: &mut f64,
) -> Result<(), Failure> {
    let h = BRepAdaptorSurface::with_face(f, false)?;
    let mut sample = FacePointSample {
        point: p,
        u,
        v,
        d1u: Vector3::new(1.0, 0.0, 0.0),
        d1v: Vector3::new(0.0, 1.0, 0.0),
    };
    let mut k = 1;
    attempt(|| {
        let bounds = [
            h.first_u_parameter()?,
            h.first_v_parameter()?,
            h.last_u_parameter()?,
            h.last_v_parameter()?,
        ];
        flag(e.point_in_the_face_on_surface_d1(f, &mut sample, param, &mut k, &h, bounds)?);
        found_point_values(&sample, k);
        xyz(*sample.d1u.xyz());
        xyz(*sample.d1v.xyz());
        Ok(())
    });
    let mut second = FacePointSample {
        d1u: Vector3::default(),
        d1v: Vector3::default(),
        ..sample
    };
    let mut k2 = 3;
    attempt(|| {
        flag(e.point_in_the_face_on_surface(
            f,
            &mut second,
            param,
            &mut k2,
            &h,
            [0.0, 0.0, 1.0, 1.0],
        )?);
        found_point_values(&second, k2);
        Ok(())
    });
    Ok(())
}

fn found_point_values(sample: &FacePointSample, index: i32) {
    pnt(sample.point);
    real(sample.u);
    real(sample.v);
    int(index);
}

/// `shape face n (u v)...`: ClassifyUVPoint.
pub fn uvpoint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let f = s.faces[r.integer() as usize].clone();
        let e = BRepClass3dSolidExplorer::with_shape(&s.shape)?;
        let h = BRepAdaptorSurface::with_face(&f, true)?;
        for _ in 0..r.integer() {
            let q = r.pnt2d();
            attempt(|| {
                int(e.classify_uv_point(e.intersector(&f)?, &h, q)? as i32);
                Ok(())
            });
        }
        Ok(())
    });
    end();
}

/// The unit box of the selectors' `Reject` check.
fn unit_box() -> BoundingBox3 {
    BoundingBox3::new(Point3::new(-1.0, -1.0, -1.0), Point3::new(1.0, 1.0, 1.0))
}

/// Mode 0 of `tp_bc3_tree_case`: the point selector.
fn point_tree(r: &mut Reader<'_>, e: &BRepClass3dSolidExplorer) {
    let m = e.get_map_ev();
    let mut sel = BRepClass3dBndBoxTreeSelectorPoint::new(m);
    sel.set_current_point(r.pnt());
    let selected = e.get_tree().select(&mut sel);
    match sel.take_failure() {
        Some(failure) => print!("{} ", failure.exception_type()),
        None => {
            int(selected);
            flag(sel.stop());
        }
    }
    for i in (1..=m.extent() as i32 + 1).chain([0]) {
        let accepted = sel.accept(&i);
        match sel.take_failure() {
            Some(failure) => print!("{} ", failure.exception_type()),
            None => flag(accepted),
        }
    }
    flag(sel.reject(&unit_box()));
}

/// Mode 1 of `tp_bc3_tree_case`: the line selector and its results.
fn line_tree(r: &mut Reader<'_>, e: &BRepClass3dSolidExplorer) -> Result<(), Failure> {
    let m = e.get_map_ev();
    let (p, d, par) = (r.pnt(), r.dir(), r.real());
    let mut sel = BRepClass3dBndBoxTreeSelectorLine::new(m);
    sel.set_current_line(Line3::new(p, d), par)?;
    let selected = e.get_tree().select(&mut sel);
    if let Some(failure) = sel.take_failure() {
        return Err(failure);
    }
    int(selected);
    flag(sel.is_correct());
    int(sel.get_nb_edge_param());
    int(sel.get_nb_vert_param());
    for i in 0..=sel.get_nb_edge_param() + 1 {
        attempt(|| {
            let (x, a, b) = sel.get_edge_param(i)?;
            int(m.find_index(&x) as i32);
            real(a);
            real(b);
            Ok(())
        });
    }
    for i in 0..=sel.get_nb_vert_param() + 1 {
        attempt(|| {
            let (x, a) = sel.get_vert_param(i)?;
            int(m.find_index(&x) as i32);
            real(a);
            Ok(())
        });
    }
    sel.clear_results();
    flag(sel.is_correct());
    int(sel.get_nb_edge_param());
    flag(sel.reject(&unit_box()));
    Ok(())
}

/// `shape mode fields`: the tree selectors on the explorer's tree.
pub fn tree_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let mode = r.integer();
        let e = BRepClass3dSolidExplorer::with_shape(&s.shape)?;
        if mode == 0 {
            point_tree(&mut r, &e);
        } else {
            attempt(|| line_tree(&mut r, &e));
        }
        Ok(())
    });
    end();
}

/// `shape face n (P D prm tol)...`: one Intersector3d on the face for several lines.
pub fn inter3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let f = s.faces[r.integer() as usize].clone();
        let n = r.integer();
        let mut x = BRepClass3dIntersector3d::new();
        flag(x.is_done());
        flag(x.has_a_point());
        int(x.state() as i32);
        for _ in 0..n {
            let (p, d) = (r.pnt(), r.dir());
            let (prm, tol) = (r.real(), r.real());
            attempt(|| {
                x.perform(&Line3::new(p, d), prm, tol, &f)?;
                flag(x.is_done());
                flag(x.has_a_point());
                real(x.u_parameter());
                real(x.v_parameter());
                real(x.w_parameter());
                pnt(*x.pnt());
                if x.has_a_point() {
                    int(x.transition() as i32);
                }
                int(x.state() as i32);
                int(index_of(&s.faces, x.face()));
                int(if x.face().is_null() {
                    -1
                } else {
                    x.face().orientation() as i32
                });
                Ok(())
            });
        }
        Ok(())
    });
    end();
}
