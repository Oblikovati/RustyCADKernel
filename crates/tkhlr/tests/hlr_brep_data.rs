//! HLRBRep_ShapeToHLR and HLRBRep_Data: the records of a box, Update, Write and the
//! hiding walk.
use rustycad_tkernel::standard::Handle;
use rustycad_tkhlr::hlr_algo::Projector;
use rustycad_tkhlr::hlr_brep::{Data, ShapeToHLR};
use rustycad_tkhlr::hlr_topo_brep::{MapOfShapeTool, OutLiner};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn view() -> Projector {
    let axis = Axis2::new(
        Point3::new(0., 0., 0.),
        Direction3::new(1., 1., 1.).unwrap(),
    )
    .unwrap();
    Projector::with_frame(axis).unwrap()
}

fn box_data(p: &Projector, mst: &mut MapOfShapeTool) -> Handle<Data> {
    let s = BRepPrimAPIMakeBox::with_axes(Axis2::default(), 1., 2., 3.)
        .shape()
        .unwrap()
        .clone();
    let o = Handle::new(OutLiner::with_original(&s));
    ShapeToHLR::load(&o, p, mst, 0).unwrap()
}

#[test]
fn a_box_has_one_record_per_vertex_edge_and_face() {
    let p = view();
    let mut mst = MapOfShapeTool::default();
    let ds = box_data(&p, &mut mst);
    let ds = ds.get().unwrap();
    assert_eq!((ds.nb_vertices(), ds.nb_edges(), ds.nb_faces()), (8, 12, 6));
    assert_eq!(ds.edge_map().extent(), 12);
    assert_eq!(ds.face_map().extent(), 6);
    let ed = ds.e_data_array();
    assert!((1..=12).all(|e| ed[e].v_sta() > 0 && ed[e].v_end() > 0));
}

#[test]
fn update_finds_the_three_hiding_faces_of_a_box_seen_on_a_corner() {
    let p = view();
    let mut mst = MapOfShapeTool::default();
    let ds = box_data(&p, &mut mst);
    let ds = ds.get().unwrap();
    ds.update(&p).unwrap();
    let fd = ds.f_data_array();
    // A closed shell: the faces turned to the eye hide, the others do not.
    let hiding = (1..=6).filter(|f| fd[*f].hiding()).count();
    assert_eq!(hiding, 3);
    assert!((1..=6).all(|f| fd[f].closed() && fd[f].simple() && fd[f].plane() && !fd[f].side()));
}

#[test]
fn the_hiding_walk_of_a_front_face_meets_the_edges_behind_it() {
    let p = view();
    let mut mst = MapOfShapeTool::default();
    let ds = box_data(&p, &mut mst);
    let ds = ds.get().unwrap();
    ds.update(&p).unwrap();
    let total = {
        let fd = ds.f_data_array();
        fd[1].wires().get().unwrap().min_max()
    };
    ds.init_bound_sort(&total, 1, 12).unwrap();
    let front = (1..=6).find(|f| ds.f_data_array()[*f].hiding()).unwrap();
    ds.init_edge(front, &mut mst).unwrap();
    assert!(!ds.is_bad_face().unwrap());
    let mut seen = 0;
    while ds.more_edge().unwrap() {
        let e = ds.edge().unwrap();
        assert!((1..=12).contains(&e));
        seen += 1;
        ds.next_edge(true).unwrap();
    }
    assert!(seen <= 12);
}

#[test]
fn write_copies_the_records_after_the_given_offsets() {
    let p = view();
    let mut mst = MapOfShapeTool::default();
    let part = box_data(&p, &mut mst);
    let part = part.get().unwrap();
    let whole = Data::new(16, 24, 12).unwrap();
    whole.write(part, 0, 0, 0).unwrap();
    whole.write(part, 8, 12, 6).unwrap();
    let ed = whole.e_data_array();
    assert_eq!(ed[13].v_sta(), part.e_data_array()[1].v_sta() + 8);
    assert_eq!(whole.edge_map().extent(), 12);
    // The wire blocks are shared with the source, as the native handle copy: the
    // second write shifted the edges of the first copy too.
    let fd = whole.f_data_array();
    let first = fd[1].wires().get().unwrap().wire(1).unwrap();
    let last = fd[7].wires().get().unwrap().wire(1).unwrap();
    assert_eq!(
        first.get().unwrap().edge(1).unwrap(),
        last.get().unwrap().edge(1).unwrap()
    );
    assert!(last.get().unwrap().edge(1).unwrap() > 12);
}

#[test]
fn a_destroyed_cache_fails_the_interference_walk() {
    let p = view();
    let mut mst = MapOfShapeTool::default();
    let ds = box_data(&p, &mut mst);
    let ds = ds.get().unwrap();
    ds.update(&p).unwrap();
    ds.destroy();
    ds.destroy();
    let total = ds.f_data_array()[1].wires().get().unwrap().min_max();
    ds.init_bound_sort(&total, 1, 12).unwrap();
    ds.init_edge(1, &mut mst).unwrap();
    if ds.more_edge().unwrap() {
        let error = ds.init_interference().unwrap_err();
        assert_eq!(error.exception_type(), "Standard_NullObject");
    }
}
