//! Probe operations of BRepBuilderAPI_VertexInspector and
//! BRepBuilderAPI_BndBoxTreeSelector, the Rust twin of
//! tools/oracle/tktopalgo_probe_sewing_helpers.cxx (the request layouts are documented
//! there).
use super::common::{Reader, end, flag, int, real, xyz};
use rustycad_tkernel::ncollection::{CellFilter, List, TreeSelector, UBTree, UBTreeFiller};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIBndBoxTreeSelector, BRepBuilderAPIVertexInspector,
};

/// `tp_int_list`.
fn int_list(l: &List<i32>) {
    int(l.extent());
    for v in l.iter() {
        int(*v);
    }
}

/// `bba_vertexinspector`.
pub fn vertexinspector_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let tol = r.real();
    let n = r.integer();
    let mut filter: CellFilter<BRepBuilderAPIVertexInspector> = CellFilter::new(tol);
    let mut inspector = BRepBuilderAPIVertexInspector::new(tol);
    for i in 1..=n {
        let p = r.xyz();
        filter.add(&i, &p);
        inspector.add(p);
    }
    let current = r.xyz();
    inspector.set_current(current);
    let (low, high) = (
        BRepBuilderAPIVertexInspector::shift(&current, -tol),
        BRepBuilderAPIVertexInspector::shift(&current, tol),
    );
    filter.inspect_range(&low, &high, &mut inspector);
    int_list(inspector.res_ind());
    for axis in 0..3 {
        real(BRepBuilderAPIVertexInspector::coord(axis, &current));
    }
    xyz(high);
    xyz(low);
    inspector.clear_res_list();
    for _ in 0..r.integer() {
        let action = inspector.inspect_target(r.integer());
        int(action.map_or(-1, |a| a as i32));
    }
    int_list(inspector.res_ind());
    end();
}

/// `tp_bba_box`.
fn read_box(r: &mut Reader<'_>) -> BoundingBox3 {
    let mut b = BoundingBox3::default();
    let (x0, y0, z0, x1, y1, z1) = (r.real(), r.real(), r.real(), r.real(), r.real(), r.real());
    b.update_bounds([x0, y0, z0], [x1, y1, z1]);
    b
}

/// `bba_boxselector`.
pub fn boxselector_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let n = r.integer();
    let mut tree: UBTree<i32, BoundingBox3> = UBTree::default();
    {
        let mut filler = UBTreeFiller::new(&mut tree, None, true);
        for i in 1..=n {
            filler.add(i, read_box(&mut r));
        }
        filler.fill();
    }
    let mut selector = BRepBuilderAPIBndBoxTreeSelector::new();
    selector.set_current(read_box(&mut r));
    int(tree.select(&mut selector));
    int_list(selector.res_ind());
    for _ in 0..r.integer() {
        flag(TreeSelector::reject(&selector, &read_box(&mut r)));
    }
    selector.clear_res_list();
    for _ in 0..r.integer() {
        flag(selector.accept_object(r.integer()));
    }
    int_list(selector.res_ind());
    end();
}
