//! Poly_MergeNodesTool counterpart.
use super::{Fields, failure, flag, mesh, transform};
use crate::xyz;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::{
    gp::{Transform3, Xyz},
    poly::{MergeNodesTool, Triangulation},
};
fn merged(result: &Handle<Triangulation>) {
    let Some(r) = result.get() else {
        print!("null ");
        return;
    };
    print!(
        "{} {} {} ",
        r.nb_nodes(),
        r.nb_triangles(),
        flag(r.is_double_precision())
    );
    for index in 1..=r.nb_nodes() {
        xyz(r.node(index).unwrap().xyz());
    }
    for index in 1..=r.nb_triangles() {
        let (a, b, c) = r.triangle(index).unwrap().get();
        print!("{a} {b} {c} ");
    }
}
fn counters(tool: &MergeNodesTool) {
    print!(
        "{} {} {} {} ",
        tool.nb_nodes(),
        tool.nb_elements(),
        tool.nb_degenerative_elems(),
        tool.nb_merged_elems()
    );
    (0..4).for_each(|i| print!("{} ", tool.element_node_index(i)));
    print!(
        "{} {} ",
        tool.merge_angle().to_bits(),
        tool.merge_tolerance().to_bits()
    );
}
/// The request fields after the mesh and its transform.
struct MergeRequest {
    transform: Transform3,
    reverse: bool,
    angle: f64,
    tolerance: f64,
    unit: f64,
    options: i32,
    facets: i32,
}
fn node(mesh: &Triangulation, index: i32) -> Result<Xyz, Failure> {
    Ok(mesh.node(index)?.xyz())
}
fn tool_report(mesh: &Handle<Triangulation>, r: &MergeRequest) -> Result<(), Failure> {
    let mut tool = MergeNodesTool::new(r.angle, r.tolerance, r.facets);
    tool.set_merge_opposite(r.options & 1 != 0);
    tool.set_merge_elems(r.options & 2 != 0);
    tool.set_drop_degenerative(r.options & 4 == 0);
    tool.set_unit_factor(r.unit);
    print!(
        "{}{}{} ",
        flag(tool.to_merge_opposite()),
        flag(tool.to_merge_elems()),
        flag(tool.to_drop_degenerative())
    );
    tool.add_triangulation(mesh, r.transform, r.reverse)?;
    counters(&tool);
    if mesh.nb_nodes() >= 4 {
        tool.add_quad(&[
            node(mesh, 1)?,
            node(mesh, 2)?,
            node(mesh, 3)?,
            node(mesh, 4)?,
        ])?;
        counters(&tool);
    }
    tool.add_triangle(&[node(mesh, 1)?, node(mesh, 2)?, node(mesh, 3)?])?;
    counters(&tool);
    *tool.change_element_node(0) = node(mesh, 1)?;
    *tool.change_element_node(1) = node(mesh, 1)?;
    *tool.change_element_node(2) = node(mesh, 2)?;
    tool.push_last_triangle()?;
    counters(&tool);
    if let Err(error) = tool.push_last_element(5) {
        failure(&error);
    }
    print!("{} ", f64::from(tool.compute_tri_normal().x()).to_bits());
    merged(&tool.result()?);
    Ok(())
}
fn map_report(mesh: &Handle<Triangulation>, r: &MergeRequest) -> Result<(), Failure> {
    let mut map = MergeNodesTool::new(r.angle, r.tolerance, -1);
    *map.change_output() = Handle::default();
    map.add_triangulation(mesh, r.transform, r.reverse)?;
    map.add_quad(&[
        node(mesh, 1)?,
        node(mesh, 2)?,
        node(mesh, 3)?,
        node(mesh, 1)?,
    ])?;
    counters(&map);
    print!("{} ", flag(map.result()?.is_null()));
    Ok(())
}
pub(crate) fn merge_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let mesh = mesh(&mut fields);
    let transform = transform(&mut fields);
    let reverse = fields.int() != 0;
    let (angle, tolerance, unit) = (fields.real(), fields.real(), fields.real());
    let (options, facets) = (fields.int(), fields.int());
    let request = MergeRequest {
        transform,
        reverse,
        angle,
        tolerance,
        unit,
        options,
        facets,
    };
    match MergeNodesTool::merge_nodes(
        &mesh,
        transform,
        reverse,
        angle,
        tolerance,
        options & 8 != 0,
    ) {
        Ok(result) => merged(&result),
        Err(error) => failure(&error),
    }
    for report in [tool_report, map_report] {
        if let Err(error) = report(&mesh, &request) {
            failure(&error);
        }
    }
    println!();
}
