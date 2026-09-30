use super::super::{flag, int, real};
use rustycad_tkernel::ncollection::{EBTree, TreeSelector, UBTreeFiller};
use rustycad_tkmath::{bnd::BoundingBox2, gp::Point2};
struct BoxSelector {
    bounds: BoundingBox2,
    objects: Vec<i32>,
}
impl TreeSelector<i32, BoundingBox2> for BoxSelector {
    fn reject(&self, bounds: &BoundingBox2) -> bool {
        self.bounds.is_out_box(bounds)
    }
    fn accept(&mut self, object: &i32) -> bool {
        self.objects.push(*object);
        true
    }
    fn stop(&self) -> bool {
        false
    }
}
fn bound(index: i32) -> BoundingBox2 {
    let mut bounds = BoundingBox2::default();
    let x = f64::from(index);
    bounds.add_point(Point2::new(x, x * 0.5));
    bounds.add_point(Point2::new(x + 1.0, x * 0.5 + 2.0));
    bounds.enlarge(1e-7);
    bounds
}
pub fn editable_tree_case(v: &[i32]) {
    let mut tree: EBTree<i32, BoundingBox2> = EBTree::default();
    {
        let mut filler = UBTreeFiller::new(&mut tree, None, v[1] != 0);
        for index in 0..v[0] {
            filler.add(if v[2] != 0 { index % 3 } else { index }, bound(index));
        }
        int(filler.fill());
        int(filler.check_tree(&mut String::new()).unwrap());
        filler.add(-1, bound(-1));
    }
    for index in -1..v[0] {
        flag(tree.contains(&index));
        if let Ok(node) = tree.find_node(&index) {
            real(node.bounds().square_extent());
        }
    }
    flag(tree.remove(&1));
    flag(tree.contains(&1));
    let mut bounds = BoundingBox2::default();
    bounds.add_point(Point2::new(-100.0, -100.0));
    bounds.add_point(Point2::new(1000.0, 1000.0));
    let mut selector = BoxSelector {
        bounds,
        objects: Vec::new(),
    };
    int(tree.select(&mut selector));
    for index in selector.objects {
        int(index);
    }
    println!();
}
