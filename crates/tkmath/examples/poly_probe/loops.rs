//! Poly_MakeLoops2D/3D counterpart with straight links: both end tangents are the chord.
use super::{Fields, failure};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    gp::{Direction2, Direction3, Xy},
    poly::make_loops::{
        LeftWay2D, LeftWay3D, Link, LinkFlag, ListOfLink, LoopWay, MakeLoops, MakeLoops2DHelper,
        MakeLoops3DHelper, MakeLoopsHelper,
    },
};
use std::collections::HashMap;
struct LoopHelper {
    points: Vec<Xy>,
    adjacent: HashMap<i32, ListOfLink>,
    none: ListOfLink,
}
impl LoopHelper {
    fn chord(&self, link: &Link) -> Xy {
        let d = self.points[(link.node2 - 1) as usize] - self.points[(link.node1 - 1) as usize];
        if link.is_reversed() { -d } else { d }
    }
}
impl MakeLoopsHelper for LoopHelper {
    fn get_adjacent_links(&self, node: i32) -> &ListOfLink {
        self.adjacent.get(&node).unwrap_or(&self.none)
    }
}
impl MakeLoops3DHelper for LoopHelper {
    fn get_first_tangent(&self, link: &Link) -> Option<Direction3> {
        let c = self.chord(link);
        (c.modulus() >= 1e-9).then(|| Direction3::new(c.x(), c.y(), 0.0).unwrap())
    }
    fn get_last_tangent(&self, link: &Link) -> Option<Direction3> {
        MakeLoops3DHelper::get_first_tangent(self, link)
    }
    fn get_normal(&self, node: i32) -> Option<Direction3> {
        (node % 5 != 4).then_some(Direction3::Z)
    }
}
impl MakeLoops2DHelper for LoopHelper {
    fn get_first_tangent(&self, link: &Link) -> Option<Direction2> {
        let c = self.chord(link);
        (c.modulus() >= 1e-9).then(|| Direction2::new(c.x(), c.y()).unwrap())
    }
    fn get_last_tangent(&self, link: &Link) -> Option<Direction2> {
        MakeLoops2DHelper::get_first_tangent(self, link)
    }
}
fn loop_link(link: &Link) {
    print!("{} {} {} ", link.node1, link.node2, link.flags);
}
/// The orientation edit and the replaced link index read after the links.
struct Edits {
    k: usize,
    orientation: i32,
    r: usize,
}
fn swapped(link: &Link) -> Link {
    let mut swapped = Link::new(link.node2, link.node1);
    swapped.flags = link.flags;
    swapped
}
fn run<W: LoopWay>(algo: &mut MakeLoops<W>, links: &[Link], edits: &Edits) -> Result<(), Failure> {
    for link in links {
        algo.add_link(link)?;
    }
    let (k, orientation) = (edits.k, edits.orientation);
    if k < links.len() {
        let old =
            algo.set_link_orientation(&links[k], LinkFlag::from_native(orientation).unwrap())?;
        print!("{} ", old as i32);
    }
    if edits.r < links.len() {
        algo.replace_link(&links[edits.r], &swapped(&links[edits.r]))?;
    }
    loop_link(&algo.find_link(&links[0]));
    print!("{} {} ", algo.perform()?, algo.get_nb_loops());
    for index in 1..=algo.get_nb_loops() {
        print!("[");
        algo.get_loop(index)?.iter().for_each(loop_link);
        print!("] ");
    }
    let mut hanging = ListOfLink::default();
    algo.get_hanging_links(&mut hanging)?;
    print!("{} ", algo.get_nb_hanging());
    hanging.iter().for_each(loop_link);
    Ok(())
}
pub(crate) fn loops_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (variant, count) = (fields.int(), fields.int());
    let points = (0..count)
        .map(|_| Xy::new(fields.real(), fields.real()))
        .collect();
    let mut helper = LoopHelper {
        points,
        adjacent: HashMap::new(),
        none: ListOfLink::default(),
    };
    let mut links = Vec::new();
    for _ in 0..fields.int() {
        let mut link = Link::new(fields.int(), fields.int());
        link.flags = fields.int();
        helper.adjacent.entry(link.node1).or_default().append(link);
        if link.node2 != link.node1 {
            helper.adjacent.entry(link.node2).or_default().append(link);
        }
        links.push(link);
    }
    let edits = Edits {
        k: fields.int() as usize,
        orientation: fields.int(),
        r: fields.int() as usize,
    };
    // The helper lists replaced links with their new node order; it is only read by
    // Perform, so updating it before the run is the native order of effects.
    if let Some(old) = links.get(edits.r) {
        let now = swapped(old);
        for link in helper
            .adjacent
            .values_mut()
            .flat_map(|list| list.iter_mut())
        {
            if link.node1 == old.node1 && link.node2 == old.node2 {
                (link.node1, link.node2) = (now.node1, now.node2);
            }
        }
    }
    let result = if variant == 2 {
        run(&mut MakeLoops::new(LeftWay3D::new(&helper)), &links, &edits)
    } else {
        run(
            &mut MakeLoops::new(LeftWay2D::new(variant == 0, &helper)),
            &links,
            &edits,
        )
    };
    if let Err(error) = result {
        failure(&error);
    }
    println!();
}
