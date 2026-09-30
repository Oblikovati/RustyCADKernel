#[derive(Debug)]
struct FuzzInput {
    seed: u32,
    iteration: usize,
    kind: u32,
    index: u32,
    delta: [f64; 3],
}
fn fuzz_decimal<T: std::str::FromStr>(value: &str) -> T {
    value.parse().unwrap_or_else(|_| {
        panic!(
            "invalid field {value:?}: expected decimal {}",
            std::any::type_name::<T>()
        )
    })
}
fn fuzz_float_bits(value: &str) -> f64 {
    let bits = u64::from_str_radix(value, 16)
        .unwrap_or_else(|_| panic!("invalid field {value:?}: expected hexadecimal f64 bits"));
    f64::from_bits(bits)
}
fn fuzz_input(line: &str) -> FuzzInput {
    let fields: Vec<&str> = line.split_whitespace().collect();
    assert_eq!(fields.len(), 7, "input {line:?}: expected seven fields");
    FuzzInput {
        seed: fuzz_decimal(fields[0]),
        iteration: fuzz_decimal(fields[1]),
        kind: fuzz_decimal(fields[2]),
        index: fuzz_decimal(fields[3]),
        delta: [
            fuzz_float_bits(fields[4]),
            fuzz_float_bits(fields[5]),
            fuzz_float_bits(fields[6]),
        ],
    }
}
fn fuzz_initial_graph(cylinder: bool) -> BRepGraph {
    let shape = if cylinder {
        BRepPrimAPIMakeCylinder::new(5.0, 15.0)
            .shape()
            .unwrap()
            .clone()
    } else {
        BRepPrimAPIMakeBox::from_sizes(10.0, 20.0, 30.0)
            .unwrap()
            .shape()
            .unwrap()
            .clone()
    };
    let mut graph = BRepGraph::new();
    graph.clear().unwrap();
    graph.shapes_mut().add(&shape).unwrap();
    assert!(!graph.is_empty());
    assert!(validate::perform_with_options(&graph, &Options::audit()).is_valid());
    graph
}
fn fuzz_bump_edge(graph: &mut BRepGraph, index: u32) {
    let id = EdgeId::new(index);
    assert!(
        !id.is_removed(graph),
        "edge {index}: expected active captured input"
    );
    let mut guard = graph.editor().edges().mutate(id).unwrap();
    let tolerance = guard.tolerance + 1e-4;
    EdgeOps::set_tolerance_guarded(&mut guard, tolerance);
}
fn fuzz_move_vertex(graph: &mut BRepGraph, index: u32, delta: [f64; 3]) {
    let id = VertexId::new(index);
    assert!(
        !id.is_removed(graph),
        "vertex {index}: expected active captured input"
    );
    let mut guard = graph.editor().vertices().mutate(id).unwrap();
    let old = guard.point;
    let point = Point3::new(old.x() + delta[0], old.y() + delta[1], old.z() + delta[2]);
    VertexOps::set_point_guarded(&mut guard, point);
}
fn fuzz_bump_face(graph: &mut BRepGraph, index: u32) {
    let id = FaceId::new(index);
    assert!(
        !id.is_removed(graph),
        "face {index}: expected active captured input"
    );
    let mut guard = graph.editor().faces().mutate(id).unwrap();
    let tolerance = guard.tolerance + 1e-4;
    FaceOps::set_tolerance_guarded(&mut guard, tolerance);
}
fn fuzz_apply_input(graph: &mut BRepGraph, input: &FuzzInput) {
    // The native stream deliberately excludes split/removal (Phase 5.10/5.11).
    // Capture verifies that all 630 original mutations apply without retries/skips.
    match input.kind {
        0 => fuzz_bump_edge(graph, input.index),
        1 => fuzz_move_vertex(graph, input.index, input.delta),
        2 => fuzz_bump_face(graph, input.index),
        kind => panic!("mutation kind {kind}: expected 0, 1 or 2"),
    }
    let audit = validate::perform_with_options(graph, &Options::audit());
    assert!(
        audit.is_valid(),
        "seed {} iteration {}: expected valid graph, issues {:?}",
        input.seed,
        input.iteration,
        audit.issues
    );
}
fn fuzz_fixed_seed(seed: u32, cylinder: bool) {
    let corpus = fuzz_corpus(cylinder);
    let inputs = corpus
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(fuzz_input);
    let mut graph = fuzz_initial_graph(cylinder);
    let mut validated = 0;
    for input in inputs.filter(|input| input.seed == seed) {
        assert_eq!(
            input.iteration, validated,
            "seed {seed}: expected contiguous input sequence"
        );
        fuzz_apply_input(&mut graph, &input);
        validated += 1;
    }
    assert!(validated >= 1);
    assert_eq!(validated, if cylinder { 40 } else { 50 });
}

fn fuzz_corpus(cylinder: bool) -> &'static str {
    if cylinder {
        include_str!("inputs/cylinder.txt")
    } else {
        include_str!("inputs/box.txt")
    }
}
