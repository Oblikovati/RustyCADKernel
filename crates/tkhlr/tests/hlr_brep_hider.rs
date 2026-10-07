//! HLRBRep_Hider with its tools: a square above a long rectangle, seen from the top.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{Compound, Shape};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkhlr::hlr_algo::{
    Interference, MinMaxIndices, Projector, add_min_max, copy_min_max, decode_min_max,
    encode_min_max,
};
use rustycad_tkhlr::hlr_brep::{
    Data, EdgeBuilder, EdgeIList, EdgeInterferenceTool, Hider, ShapeToHLR, VertexList,
};
use rustycad_tkhlr::hlr_topo_brep::{MapOfShapeTool, OutLiner};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeFace;

fn rectangle(z: f64, bounds: (f64, f64, f64, f64)) -> Shape {
    let plane = Plane3::new(Point3::new(0., 0., z), Direction3::new(0., 0., 1.).unwrap()).unwrap();
    let f = BRepBuilderAPIMakeFace::from_plane_bounds(plane, bounds)
        .unwrap()
        .face()
        .unwrap();
    Shape::from(f)
}

fn two_squares() -> Shape {
    let mut c = Compound::default();
    let b = BRepBuilder::default();
    b.make_compound(&mut c);
    let mut s = Shape::from(c);
    b.add(&mut s, &rectangle(2., (-1., 1., -1., 1.))).unwrap();
    b.add(&mut s, &rectangle(0., (-3., 3., -0.5, 0.5))).unwrap();
    s
}

#[test]
fn the_upper_square_hides_the_middle_of_the_lower_rectangle() {
    let p = Projector::new().unwrap();
    let o = Handle::new(OutLiner::with_original(&two_squares()));
    let mut mst = MapOfShapeTool::default();
    let handle = ShapeToHLR::load(&o, &p, &mut mst, 0).unwrap();
    let ds = handle.get().unwrap();
    ds.update(&p).unwrap();
    ds.init_bound_sort(&shape_box(ds), 1, ds.nb_edges())
        .unwrap();
    let h = Hider::new(&handle);
    for f in 1..=ds.nb_faces() {
        if ds.f_data_array()[f].hiding() {
            h.hide(f, &mut mst).unwrap();
        }
    }
    // The two long edges of the lower rectangle pass under the upper square: each
    // keeps two visible parts; every other edge stays whole.
    let ed = ds.e_data_array();
    let parts: Vec<i32> = (1..=ds.nb_edges())
        .map(|e| ed[e].status().nb_visible_part())
        .collect();
    assert_eq!(parts.iter().filter(|n| **n == 2).count(), 2);
    assert_eq!(parts.iter().filter(|n| **n == 1).count(), 6);
}

// The box of the faces (as HLRBRep_InternalAlgo::Update).
fn shape_box(ds: &Data) -> MinMaxIndices {
    let fd = ds.f_data_array();
    let (mut smin, mut smax) = (MinMaxIndices::default(), MinMaxIndices::default());
    let (mut tmin, mut tmax) = (MinMaxIndices::default(), MinMaxIndices::default());
    for f in 1..=ds.nb_faces() {
        decode_min_max(
            &fd[f].wires().get().unwrap().min_max(),
            &mut tmin,
            &mut tmax,
        );
        if f == 1 {
            copy_min_max(&tmin, &tmax, &mut smin, &mut smax);
        } else {
            add_min_max(&tmin, &tmax, &mut smin, &mut smax);
        }
    }
    let mut tot = MinMaxIndices::default();
    encode_min_max(&smin, &smax, &mut tot);
    tot
}

#[test]
fn add_interference_keeps_the_list_sorted() {
    let tool = EdgeInterferenceTool::new(&Handle::new(Data::new(0, 0, 0).unwrap()));
    let mut v = Vec::new();
    for p in [3., 1., 2.] {
        let mut i = Interference::new();
        i.change_intersection().set_parameter(p);
        EdgeIList::add_interference_in(&mut v, &i, &tool);
    }
    let params: Vec<f64> = v.iter().map(|i| i.intersection().parameter()).collect();
    assert_eq!(params, vec![1., 2., 3.]);
}

#[test]
fn a_vertex_list_merges_the_edge_ends_and_the_interferences_by_parameter() {
    // A default tool has both edge ends at parameter 0, vertex index 0.
    let tool = EdgeInterferenceTool::new(&Handle::new(Data::new(0, 0, 0).unwrap()));
    let mut list = Vec::new();
    for (p, t) in [(1., Orientation::Forward), (2., Orientation::Reversed)] {
        let mut i = Interference::new();
        i.change_intersection().set_parameter(p);
        i.change_intersection().set_index(5);
        i.change_intersection()
            .set_orientation(Orientation::Internal);
        i.set_transition(t);
        list.push(i);
    }
    let mut v_list = VertexList::new(&tool, &list).unwrap();
    let mut seen = Vec::new();
    while v_list.more() {
        seen.push((v_list.is_boundary(), v_list.current().unwrap().parameter()));
        v_list.next().unwrap();
    }
    assert_eq!(seen, vec![(true, 0.), (true, 0.), (false, 1.), (false, 2.)]);
    // The edge builder of these vertices: the face state is OUT before the first
    // interference, and the edge is OUT after its end: no part to build.
    let mut v_list = VertexList::new(&tool, &list).unwrap();
    let mut eb = EdgeBuilder::new(&mut v_list).unwrap();
    eb.builds(State::In).unwrap();
    assert!(!eb.more_vertices());
}
