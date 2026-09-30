//! BRepExtrema probe operations of the Ext wrappers, the Rust twin of
//! tools/oracle/tktopalgo_probe_brepextrema_ext.cxx: each wrapper built, initialized and
//! performed in several ways, then every accessor for the indices 0 to NbExt+1.
use super::brep_extrema::{ext_algo, ext_flag, sub, vertex};
use super::common::{Reader, attempt, end, flag, int, pnt, real};
use super::shape::tp_shape;
use rustycad_tkbrep::topods::{self, Vertex};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tktopalgo::brep_extrema::{
    BRepExtremaExtCC, BRepExtremaExtCF, BRepExtremaExtFF, BRepExtremaExtPC, BRepExtremaExtPF,
};

/// Prints the two reals of a parameter pair.
fn pair((u, v): (f64, f64)) {
    real(u);
    real(v);
}

/// `n` vertices `x y z tol`.
fn vertices(r: &mut Reader) -> Result<Vec<Vertex>, Failure> {
    let n = r.integer();
    (0..n).map(|_| vertex(r)).collect()
}

/// The number of extrema to walk: NbExt when done (0 otherwise or when it raises).
fn walked(done: bool, nb: Result<i32, Failure>) -> i32 {
    if done { nb.unwrap_or(0) } else { 0 }
}

/// `tp_bxx_pc_state`.
fn pc_state(e: &BRepExtremaExtPC) {
    flag(e.is_done());
    attempt(|| e.nb_ext().map(int));
    for i in 0..=walked(e.is_done(), e.nb_ext()) + 1 {
        attempt(|| e.square_distance(i).map(real));
        attempt(|| e.is_min(i).map(flag));
        attempt(|| e.parameter(i).map(real));
        attempt(|| e.point(i).map(pnt));
    }
    let (d1, d2, p1, p2) = e.trimmed_square_distances();
    real(d1);
    real(d2);
    pnt(p1);
    pnt(p2);
}

/// `bxx_pc`: `shape edge mode n (x y z tol)...`.
pub fn pc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let e = sub(&mut r, &s)?;
        let mode = r.integer();
        let vs = vertices(&mut r)?;
        let edge = topods::edge(&e)?;
        let mut one = BRepExtremaExtPC::new();
        if mode == 1 {
            attempt(|| one.initialize(&edge));
        }
        for v in &vs {
            match mode {
                0 => attempt(|| {
                    pc_state(&BRepExtremaExtPC::with_vertex_edge(v, &edge)?);
                    Ok(())
                }),
                3 => {
                    for k in &s.edges {
                        attempt(|| one.initialize(k));
                        attempt(|| one.perform(v));
                        pc_state(&one);
                    }
                }
                _ => {
                    attempt(|| one.perform(v));
                    pc_state(&one);
                }
            }
        }
        Ok(())
    });
    end();
}

/// `tp_bxx_pf_state`.
fn pf_state(e: &BRepExtremaExtPF) {
    flag(e.is_done());
    int(e.nb_ext());
    for i in 0..=e.nb_ext() + 1 {
        attempt(|| e.square_distance(i).map(real));
        attempt(|| e.parameter(i).map(pair));
        attempt(|| e.point(i).map(pnt));
    }
}

/// The flag and algorithm of the `k`-th Perform of mode 3.
fn changed(k: i32, (fl, al): (ExtFlag, ExtAlgo)) -> (ExtFlag, ExtAlgo) {
    let f = if k % 3 == 1 { ExtFlag::Min } else { fl };
    let a = if k % 3 == 2 { ExtAlgo::Tree } else { al };
    (f, a)
}

/// `bxx_pf`: `shape face flag algo mode n (x y z tol)...`.
pub fn pf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let s = tp_shape(&mut r)?;
        let f = topods::face(&sub(&mut r, &s)?)?;
        let (fl, al) = (ext_flag(r.integer()), ext_algo(r.integer()));
        let mode = r.integer();
        let vs = vertices(&mut r)?;
        let mut one = BRepExtremaExtPF::new();
        if mode == 1 || mode == 3 {
            attempt(|| one.initialize(&f, fl, al));
        }
        let mut k = 0;
        for v in &vs {
            if mode == 0 {
                attempt(|| {
                    pf_state(&BRepExtremaExtPF::with_vertex_face(v, &f, fl, al)?);
                    Ok(())
                });
                continue;
            }
            if mode == 3 {
                let (nf, na) = changed(k, (fl, al));
                one.set_flag(nf);
                one.set_algo(na);
            }
            k += 1;
            attempt(|| one.perform(v, &f));
            pf_state(&one);
        }
        Ok(())
    });
    end();
}

/// `tp_bxx_cc_state`.
fn cc_state(e: &BRepExtremaExtCC) {
    flag(e.is_done());
    attempt(|| e.is_parallel().map(flag));
    attempt(|| e.nb_ext().map(int));
    for i in 0..=walked(e.is_done(), e.nb_ext()) + 1 {
        attempt(|| e.square_distance(i).map(real));
        attempt(|| e.parameter_on_e1(i).map(real));
        attempt(|| e.point_on_e1(i).map(pnt));
        attempt(|| e.parameter_on_e2(i).map(real));
        attempt(|| e.point_on_e2(i).map(pnt));
    }
    let (d, p) = e.trimmed_square_distances();
    for i in 0..4 {
        real(d[i]);
        pnt(p[i]);
    }
}

/// `bxx_cc`: `shape1 edge1 shape2 edge2 mode`.
pub fn cc_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let e1 = topods::edge(&sub(&mut r, &a)?)?;
        let b = tp_shape(&mut r)?;
        let e2 = topods::edge(&sub(&mut r, &b)?)?;
        let mode = r.integer();
        let mut one = BRepExtremaExtCC::new();
        match mode {
            0 => attempt(|| {
                cc_state(&BRepExtremaExtCC::with_edges(&e1, &e2)?);
                Ok(())
            }),
            1 => {
                attempt(|| one.initialize(&e2));
                attempt(|| one.perform(&e1));
                cc_state(&one);
                attempt(|| one.perform(&e2));
                cc_state(&one);
            }
            2 => {
                attempt(|| one.perform(&e1));
                cc_state(&one);
            }
            _ => {
                attempt(|| one.initialize(&e1));
                attempt(|| one.initialize(&e2));
                for _ in 0..2 {
                    attempt(|| one.perform(&e1));
                    cc_state(&one);
                }
            }
        }
        Ok(())
    });
    end();
}

/// `tp_bxx_cf_state`.
fn cf_state(e: &BRepExtremaExtCF) {
    flag(e.is_done());
    attempt(|| e.is_parallel().map(flag));
    int(e.nb_ext());
    for i in 0..=e.nb_ext() + 1 {
        attempt(|| e.square_distance(i).map(real));
        attempt(|| e.parameter_on_edge(i).map(real));
        attempt(|| e.parameter_on_face(i).map(pair));
        attempt(|| e.point_on_edge(i).map(pnt));
        attempt(|| e.point_on_face(i).map(pnt));
    }
}

/// `bxx_cf`: `shape1 edge shape2 face mode`.
pub fn cf_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let e = topods::edge(&sub(&mut r, &a)?)?;
        let b = tp_shape(&mut r)?;
        let f = topods::face(&sub(&mut r, &b)?)?;
        let mode = r.integer();
        let mut one = BRepExtremaExtCF::new();
        match mode {
            0 => attempt(|| {
                cf_state(&BRepExtremaExtCF::with_edge_face(&e, &f)?);
                Ok(())
            }),
            1 => {
                attempt(|| one.initialize(&e, &f));
                for _ in 0..2 {
                    attempt(|| one.perform(&e, &f));
                    cf_state(&one);
                }
            }
            2 => {
                attempt(|| one.perform(&e, &f));
                cf_state(&one);
            }
            _ => {
                attempt(|| one.initialize(&e, &f));
                for k in &a.edges {
                    attempt(|| one.perform(k, &f));
                    cf_state(&one);
                }
            }
        }
        Ok(())
    });
    end();
}

/// `tp_bxx_ff_state`.
fn ff_state(e: &BRepExtremaExtFF) {
    flag(e.is_done());
    attempt(|| e.is_parallel().map(flag));
    int(e.nb_ext());
    for i in 0..=e.nb_ext() + 1 {
        attempt(|| e.square_distance(i).map(real));
        attempt(|| e.parameter_on_face1(i).map(pair));
        attempt(|| e.parameter_on_face2(i).map(pair));
        attempt(|| e.point_on_face1(i).map(pnt));
        attempt(|| e.point_on_face2(i).map(pnt));
    }
}

/// `bxx_ff`: `shape1 face1 shape2 face2 mode`.
pub fn ff_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let a = tp_shape(&mut r)?;
        let f1 = topods::face(&sub(&mut r, &a)?)?;
        let b = tp_shape(&mut r)?;
        let f2 = topods::face(&sub(&mut r, &b)?)?;
        let mode = r.integer();
        let mut one = BRepExtremaExtFF::new();
        match mode {
            0 => attempt(|| {
                ff_state(&BRepExtremaExtFF::with_faces(&f1, &f2)?);
                Ok(())
            }),
            1 => {
                attempt(|| one.initialize(&f2));
                for _ in 0..2 {
                    attempt(|| one.perform(&f1, &f2));
                    ff_state(&one);
                }
            }
            2 => {
                attempt(|| one.perform(&f1, &f2));
                ff_state(&one);
            }
            _ => {
                attempt(|| one.initialize(&f2));
                for k in &a.faces {
                    attempt(|| one.perform(k, &f2));
                    ff_state(&one);
                }
            }
        }
        Ok(())
    });
    end();
}
