//! Poly_CoherentTriangulation counterpart; triangles and links are printed by index, as the
//! native probe maps its pointers back to indices (-1 null, -2 outside the used array).
use super::{Fields, failure, flag, flat, fvec, mesh};
use crate::{xy, xyz};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
use rustycad_tkmath::{
    gp::{Point2, Xyz},
    poly::{CoherentLink, CoherentTriangulation, Triangulation, TwoIntegers},
};
fn slot(index: Option<usize>, used: usize) -> i64 {
    match index {
        None => -1,
        Some(i) if i < used => i as i64,
        Some(_) => -2,
    }
}
fn state(c: &CoherentTriangulation) {
    let used = c.link_count();
    print!(
        "{} {} {} {} {} T ",
        c.max_node(),
        c.max_triangle(),
        c.n_nodes(),
        c.n_triangles(),
        c.n_links()
    );
    for index in 0..=c.max_triangle() {
        let t = c.triangle(index as usize).unwrap();
        print!(
            "{} {} {} {} ",
            t.node(0),
            t.node(1),
            t.node(2),
            t.n_connections()
        );
        for k in 0..3 {
            let triangles = (c.max_triangle() + 1) as usize;
            print!(
                "{} {} {} ",
                slot(t.get_connected_tri(k), triangles),
                t.get_connected_node(k),
                slot(t.get_link(k), used)
            );
        }
    }
    print!("L ");
    for index in 0..used {
        let l = c.link(index).unwrap();
        print!(
            "{} {} {} {} ",
            l.node(0),
            l.node(1),
            l.opposite_node(0),
            l.opposite_node(1)
        );
    }
    print!("N ");
    for index in 0..=c.max_node() {
        let node = c.node(index).unwrap();
        print!("{} {} ", flag(node.is_free_node()), node.get_index());
        node.triangles().for_each(|t| print!("{t} "));
        print!("; ");
    }
}
fn edit(c: &mut CoherentTriangulation, fields: &mut Fields<'_>) -> Result<(), Failure> {
    let (op, a, b, d, e) = (
        fields.int(),
        fields.int(),
        fields.int(),
        fields.int(),
        fields.int(),
    );
    print!("E{op} ");
    match op {
        1 => print!("{} ", flag(c.remove_triangle(a as usize)?)),
        2 => print!(
            "{} ",
            slot(c.add_triangle(a, b, d)?, (c.max_triangle() + 1) as usize)
        ),
        3 => print!("{} ", flag(c.replace_nodes(a as usize, b, d, e)?)),
        4 if (a as usize) < c.link_count() => c.remove_link(a as usize)?,
        5 => print!("{} ", c.compute_links()?),
        6 => c.clear_links(),
        7 => print!(
            "{} ",
            c.set_node(Xyz::new(a.into(), b.into(), d.into()), -1)
        ),
        8 => print!("{} ", slot(c.add_link(a as usize, b)?, c.link_count())),
        _ => {}
    }
    Ok(())
}
fn found(c: &CoherentTriangulation) -> Result<(), Failure> {
    let links: Vec<CoherentLink> = c.iter_links().map(|(_, l)| *l).collect();
    let triangles = (c.max_triangle() + 1) as usize;
    for link in links {
        let (ok, [left, right]) = c.find_triangle(&link)?;
        print!(
            "{} {} {} ",
            flag(ok),
            slot(left, triangles),
            slot(right, triangles)
        );
    }
    Ok(())
}
fn exported(r: &Handle<Triangulation>) {
    let Some(r) = r.get() else {
        print!("null ");
        return;
    };
    print!(
        "{} {} {} {} {} ",
        r.nb_nodes(),
        r.nb_triangles(),
        flag(r.has_normals()),
        flag(r.has_uv_nodes()),
        r.deflection().to_bits()
    );
    for index in 1..=r.nb_nodes() {
        xyz(r.node(index).unwrap().xyz());
        if r.has_normals() {
            fvec(r.normal_vec3(index).unwrap());
        }
        if r.has_uv_nodes() {
            xy(r.uv_node(index).unwrap().xy());
        }
    }
    for index in 1..=r.nb_triangles() {
        let (a, b, d) = r.triangle(index).unwrap().get();
        print!("{a} {b} {d} ");
    }
}
fn prepared(fields: &mut Fields<'_>) -> (Handle<Triangulation>, i32, f64) {
    let m = mesh(fields);
    let (options, tolerance) = (fields.int(), fields.real());
    m.set_deflection(tolerance * 3.0);
    if options & 1 != 0 {
        m.compute_normals().unwrap();
    }
    if options & 2 != 0 {
        m.add_uv_nodes().unwrap();
        for index in 1..=m.nb_nodes() {
            let z = m.node(index).unwrap().z();
            m.set_uv_node(index, Point2::new(z, f64::from(index)))
                .unwrap();
        }
    }
    (m, options, tolerance)
}
fn report(
    m: &Handle<Triangulation>,
    options: i32,
    tolerance: f64,
    fields: &mut Fields<'_>,
) -> Result<(), Failure> {
    let edits = fields.int();
    let mut c = CoherentTriangulation::from_triangulation(m)?;
    print!("{} ", c.deflection().to_bits());
    state(&c);
    if options & 4 != 0 {
        print!("{} ", c.compute_links()?);
        state(&c);
        found(&c)?;
    }
    for _ in 0..edits {
        edit(&mut c, fields)?;
        state(&c);
    }
    let mut removed: List<TwoIntegers> = List::default();
    print!(
        "{} ",
        flag(c.remove_degenerated(tolerance, Some(&mut removed))?)
    );
    removed
        .iter()
        .for_each(|p| print!("{} {} ", p.value[0], p.value[1]));
    state(&c);
    let mut free: List<i32> = List::default();
    print!("{} ", flag(c.get_free_nodes(&mut free)));
    free.iter().for_each(|n| print!("{n} "));
    exported(&c.get_triangulation()?);
    match c.clone_triangulation()? {
        None => print!("null "),
        Some(copy) => print!(
            "{} {} {} ",
            copy.n_nodes(),
            copy.n_triangles(),
            copy.deflection().to_bits()
        ),
    }
    let mut dump = Vec::new();
    c.dump(&mut dump).unwrap();
    print!("{} ", flat(&dump));
    Ok(())
}
pub(crate) fn coherent_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (m, options, tolerance) = prepared(&mut fields);
    if let Err(error) = report(&m, options, tolerance, &mut fields) {
        failure(&error);
    }
    println!();
}
