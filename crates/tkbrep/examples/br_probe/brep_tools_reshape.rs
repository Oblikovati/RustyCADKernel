//! ReShape, History, Substitution, Quilt and PurgeLocations driven by commands over a
//! scene; the Rust twin of tools/oracle/tkbrep_probe_breptools_reshape.cxx.
use super::brep::{point, scene};
use super::brep_tools::written;
use super::common::{
    Reader, attempt, end, failure, flag, int, out_stream, text, word, written_bytes,
};
use super::graph::{Graph, desc};
use rustycad_tkbrep::brep_tools::{
    BRepToolsReShape, History, PurgeLocations, Quilt, ReShape, Substitution,
};
use rustycad_tkbrep::topods::{self, Shape, ShapeIterator};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;

/// `bt_key`: the descriptor as numbers (-9 for null).
fn key(g: &Graph, s: &Shape) -> Vec<i64> {
    if s.is_null() {
        return vec![-9];
    }
    let mut k = vec![i64::from(g.tshape_id(s)), s.orientation() as i64];
    let mut c = s.location().clone();
    while !c.is_identity() {
        k.push(i64::from(g.datum_id(c.first_datum().unwrap())));
        k.push(i64::from(c.first_power().unwrap()));
        c = c.next_location().clone();
    }
    k
}

/// `bt_sorted`.
fn sorted(g: &Graph, l: &[Shape]) {
    let mut keys: Vec<Vec<i64>> = l.iter().map(|s| key(g, s)).collect();
    keys.sort();
    int(keys.len() as i32);
    for k in keys {
        int(k.len() as i32);
        k.into_iter().for_each(|v| print!("{v} "));
    }
}

fn ids(g: &Graph, r: &mut Reader<'_>) -> Vec<Shape> {
    (0..r.integer()).map(|_| g.shape(r.integer())).collect()
}

/// Prints `-` or the exception of a command.
fn done(result: Result<(), Failure>) {
    match result {
        Ok(()) => word("-"),
        Err(error) => failure(&error),
    }
}

/// `bt_reshape_command`.
fn reshape_command(
    g: &mut Graph,
    rs: &BRepToolsReShape,
    r: &mut Reader<'_>,
) -> Result<(), Failure> {
    match r.integer() {
        0 => {
            let a = g.shape(r.integer());
            rs.replace(&a, &g.shape(r.integer()))
        }
        1 => rs.remove(&g.shape(r.integer())),
        2 => {
            let l = ids(g, r);
            rs.merge(&l, &g.shape(r.integer()))
        }
        3 => {
            rs.set_mode_consider_location(r.flag());
            Ok(())
        }
        4 => {
            let v = g.shape(r.integer());
            let tol = r.real();
            let copy = rs.copy_vertex(&topods::vertex(&v)?, tol)?;
            g.shapes.push(copy.into());
            Ok(())
        }
        5 => {
            let v = g.shape(r.integer());
            let (p, tol) = (point(r), r.real());
            let copy = rs.copy_vertex_at(&topods::vertex(&v)?, p, tol)?;
            g.shapes.push(copy.into());
            Ok(())
        }
        _ => {
            rs.clear();
            Ok(())
        }
    }
}

/// `bt_reshape_report`.
fn reshape_report(g: &Graph, rs: &BRepToolsReShape, sh: &Shape) {
    flag(rs.is_recorded(sh));
    flag(rs.is_new_shape(sh));
    attempt(|| rs.value(sh).map(|v| desc(g, &v)));
    attempt(|| rs.value_leaf(sh).map(|v| desc(g, &v)));
    for last in [false, true] {
        attempt(|| {
            let (status, n) = rs.status(sh, last)?;
            int(status);
            desc(g, &n);
            Ok(())
        });
    }
    for until in [ShapeEnum::Shape, ShapeEnum::Face] {
        attempt(|| {
            text(&written(&rs.apply(sh, until)?, true, true, 3)?);
            Ok(())
        });
    }
}

/// `bt_history_report`.
fn history_report(g: &Graph, h: &History) {
    flag(h.has_generated());
    flag(h.has_modified());
    flag(h.has_removed());
    for sh in g.shapes.iter().filter(|sh| !sh.is_null()) {
        flag(History::is_supported_type(sh).unwrap());
        attempt(|| h.generated(sh).map(|l| sorted(g, &l)));
        attempt(|| h.modified(sh).map(|l| sorted(g, &l)));
        attempt(|| h.is_removed(sh).map(flag));
    }
    let mut o = out_stream(6);
    h.dump(&mut o);
    text(&written_bytes(o));
}

/// `br_tools_reshape script ncmd (command)...`.
pub fn reshape_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut s = scene(&mut r);
    let rs = BRepToolsReShape::new();
    for _ in 0..r.integer() {
        done(reshape_command(&mut s.graph, &rs, &mut r));
    }
    for sh in s.graph.shapes.iter().filter(|sh| !sh.is_null()) {
        reshape_report(&s.graph, &rs, sh);
    }
    attempt(|| {
        rs.history()
            .map(|h| history_report(&s.graph, h.get().unwrap()))
    });
    end();
}

/// `bt_history_command`.
fn history_command(
    g: &Graph,
    hs: &[Handle<History>; 2],
    r: &mut Reader<'_>,
) -> Result<(), Failure> {
    let code = r.integer();
    if code >= 20 {
        let (first, second) = (hs[0].get().unwrap(), &hs[1]);
        return if code == 20 {
            first.merge(second.get().unwrap())
        } else {
            first.merge_handle(second)
        };
    }
    let h = hs[usize::from(code >= 10)].get().unwrap();
    let a = g.shape(r.integer());
    match code % 10 {
        0 => h.add_generated(&a, &g.shape(r.integer())),
        1 => h.add_modified(&a, &g.shape(r.integer())),
        2 => h.remove(&a),
        3 => h.replace_generated(&a, &g.shape(r.integer())),
        4 => h.replace_modified(&a, &g.shape(r.integer())),
        _ => {
            h.clear();
            Ok(())
        }
    }
}

/// `br_tools_history script ncmd (command)...`.
pub fn history_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let hs = [Handle::new(History::new()), Handle::new(History::new())];
    for _ in 0..r.integer() {
        done(history_command(&s.graph, &hs, &mut r));
    }
    for h in &hs {
        history_report(&s.graph, h.get().unwrap());
    }
    end();
}

/// `br_tools_subst script ncmd (command)...`.
pub fn subst_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let g = &s.graph;
    let mut sub = Substitution::new();
    for _ in 0..r.integer() {
        let result = match r.integer() {
            0 => {
                let a = g.shape(r.integer());
                let l = ids(g, &mut r);
                sub.substitute(&a, &l)
            }
            1 => sub.build(&g.shape(r.integer())),
            _ => {
                sub.clear();
                Ok(())
            }
        };
        done(result);
    }
    for sh in g.shapes.iter().filter(|sh| !sh.is_null()) {
        flag(sub.is_copied(sh));
        attempt(|| {
            let l = sub.copy(sh)?;
            int(l.len() as i32);
            for c in &l {
                text(&written(c, true, true, 3)?);
            }
            Ok(())
        });
    }
    end();
}

/// `bt_shells`.
fn shells(c: &Shape) -> Result<(), Failure> {
    let mut others = [0; 9];
    int(c.nb_children());
    for x in ShapeIterator::new(c, true, true) {
        let kind = x.shape_type()?;
        if kind != ShapeEnum::Shell {
            others[kind as usize] += 1;
            continue;
        }
        int(x.nb_children());
        flag(x.closed()?);
        flag(x.orientable()?);
        int(x.orientation() as i32);
    }
    others.into_iter().for_each(int);
    Ok(())
}

/// One Quilt command: 0 bind edges, 1 bind vertices, 2 add.
fn quilt_command(g: &Graph, q: &mut Quilt, r: &mut Reader<'_>) -> Result<(), Failure> {
    let code = r.integer();
    let a = g.shape(r.integer());
    match code {
        0 => q.bind_edge(&topods::edge(&a)?, &topods::edge(&g.shape(r.integer()))?),
        1 => {
            q.bind_vertex(
                &topods::vertex(&a)?,
                &topods::vertex(&g.shape(r.integer()))?,
            );
            Ok(())
        }
        _ => q.add(&a),
    }
}

/// `br_tools_quilt script ncmd (command)...`.
pub fn quilt_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let g = &s.graph;
    let mut q = Quilt::new();
    for _ in 0..r.integer() {
        done(quilt_command(g, &mut q, &mut r));
    }
    for sh in g.shapes.iter().filter(|sh| !sh.is_null()) {
        flag(q.is_copied(sh));
        attempt(|| {
            text(&written(&q.copy(sh)?, true, true, 3)?);
            Ok(())
        });
    }
    attempt(|| shells(&q.shells()?));
    end();
}

/// `br_tools_purge script`.
pub fn purge_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let g = &s.graph;
    for sh in g.shapes.iter().filter(|sh| !sh.is_null()) {
        let mut p = PurgeLocations::new();
        attempt(|| {
            flag(p.perform(sh)?);
            flag(p.is_done());
            text(&written(p.result(), true, true, 3)?);
            for x in g.shapes.iter().filter(|x| !x.is_null()) {
                flag(p.modified_shape(x).is_same(x));
            }
            Ok(())
        });
    }
    end();
}
