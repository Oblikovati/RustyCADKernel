use rustycad_tkshhealing::shape_analysis::ShapeAnalysisShapeContents;
const COUNTERS: [fn(&ShapeAnalysisShapeContents) -> i32; 33] = [
    ShapeAnalysisShapeContents::nb_solids,
    ShapeAnalysisShapeContents::nb_shells,
    ShapeAnalysisShapeContents::nb_faces,
    ShapeAnalysisShapeContents::nb_wires,
    ShapeAnalysisShapeContents::nb_edges,
    ShapeAnalysisShapeContents::nb_vertices,
    ShapeAnalysisShapeContents::nb_solids_with_voids,
    ShapeAnalysisShapeContents::nb_big_splines,
    ShapeAnalysisShapeContents::nb_c0_surfaces,
    ShapeAnalysisShapeContents::nb_c0_curves,
    ShapeAnalysisShapeContents::nb_offset_surfaces,
    ShapeAnalysisShapeContents::nb_indirect_surfaces,
    ShapeAnalysisShapeContents::nb_offset_curves,
    ShapeAnalysisShapeContents::nb_trimmed_curves_2d,
    ShapeAnalysisShapeContents::nb_trimmed_curves_3d,
    ShapeAnalysisShapeContents::nb_bspline_surfaces,
    ShapeAnalysisShapeContents::nb_bezier_surfaces,
    ShapeAnalysisShapeContents::nb_trimmed_surfaces,
    ShapeAnalysisShapeContents::nb_wires_with_seam,
    ShapeAnalysisShapeContents::nb_wires_with_several_seams,
    ShapeAnalysisShapeContents::nb_faces_with_several_wires,
    ShapeAnalysisShapeContents::nb_missing_pcurves,
    ShapeAnalysisShapeContents::nb_free_faces,
    ShapeAnalysisShapeContents::nb_free_wires,
    ShapeAnalysisShapeContents::nb_free_edges,
    ShapeAnalysisShapeContents::nb_shared_solids,
    ShapeAnalysisShapeContents::nb_shared_shells,
    ShapeAnalysisShapeContents::nb_shared_faces,
    ShapeAnalysisShapeContents::nb_shared_wires,
    ShapeAnalysisShapeContents::nb_shared_free_wires,
    ShapeAnalysisShapeContents::nb_shared_free_edges,
    ShapeAnalysisShapeContents::nb_shared_edges,
    ShapeAnalysisShapeContents::nb_shared_vertices,
];
fn counters(c: &ShapeAnalysisShapeContents) -> Vec<i32> {
    COUNTERS.iter().map(|get| get(c)).collect()
}
pub fn assert_native(name: &str, c: &ShapeAnalysisShapeContents) {
    let mut actual = counters(c);
    actual.push(c.big_spline_sequence().sequence(|s| s.len() as i32));
    actual.push(c.indirect_sequence().sequence(|s| s.len() as i32));
    actual.push(c.offset_surface_sequence().sequence(|s| s.len() as i32));
    actual.push(c.trimmed_3d_sequence().sequence(|s| s.len() as i32));
    actual.push(c.offset_curve_sequence().sequence(|s| s.len() as i32));
    actual.push(c.trimmed_2d_sequence().sequence(|s| s.len() as i32));
    let frozen = include_str!("../../../../../tools/oracle/tkshhealing_shape_contents.expected");
    let row = frozen
        .lines()
        .find(|s| s.starts_with(&format!("{name} ")))
        .unwrap();
    let expected: Vec<i32> = row
        .split_whitespace()
        .skip(1)
        .map(|s| s.parse().unwrap())
        .collect();
    assert_eq!(actual, expected, "{name}");
}
