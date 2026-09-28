//! CellFilter scripts and KDTree Array1 builds/moves (twin of `tkernel_ncollection_trees_cell_probe.cxx`).
use super::number;
use rustycad_tkernel::ncollection::{Array1, CellAction, CellFilter, CellInspector, KDTree};
use std::str::SplitWhitespace;

/// The output of a cell-filter or KD-tree operation, `None` for other operations.
pub fn evaluate(op: &str, input: &mut SplitWhitespace<'_>) -> Option<String> {
    match op {
        "ncoll_trees_cell" => Some(cell(input)),
        "ncoll_trees_kd_array" => Some(kd_array(input)),
        _ => None,
    }
}

/// Records every visited target and purges the one equal to `purge`.
struct Recorder {
    visited: Vec<i32>,
    purge: i32,
}
impl CellInspector for Recorder {
    type Target = i32;
    type Point = [f64; 3];
    const DIMENSION: usize = 2;
    fn coord(axis: usize, point: &[f64; 3]) -> f64 {
        point[axis]
    }
    fn is_equal(first: &i32, second: &i32) -> bool {
        first == second
    }
    fn inspect(&mut self, target: &i32) -> CellAction {
        self.visited.push(*target);
        if *target == self.purge {
            CellAction::Purge
        } else {
            CellAction::Keep
        }
    }
}

fn read_point(input: &mut SplitWhitespace<'_>) -> [f64; 3] {
    std::array::from_fn(|_| number(input))
}
fn cell_inspect(
    input: &mut SplitWhitespace<'_>,
    filter: &mut CellFilter<Recorder>,
    range: bool,
) -> String {
    let mut recorder: Recorder = Recorder {
        visited: Vec::new(),
        purge: number(input),
    };
    let min: [f64; 3] = read_point(input);
    if range {
        filter.inspect_range(&min, &read_point(input), &mut recorder);
    } else {
        filter.inspect(&min, &mut recorder);
    }
    let visited: Vec<String> = recorder.visited.iter().map(i32::to_string).collect();
    format!(" [{}]", visited.join(","))
}
fn cell_reset_array(input: &mut SplitWhitespace<'_>, filter: &mut CellFilter<Recorder>) -> String {
    let lower: i32 = number(input);
    let count: i32 = number(input);
    let mut sizes: Array1<'static, f64> = Array1::with_bounds(lower, lower + count - 1).unwrap();
    for value in sizes.as_mut_slice() {
        *value = number(input);
    }
    filter.reset_sizes(&sizes, None);
    String::from(" S")
}
fn cell_reset(input: &mut SplitWhitespace<'_>, filter: &mut CellFilter<Recorder>) -> String {
    match filter.reset(number(input), None) {
        Ok(()) => String::from(" s"),
        Err(error) => format!(" throw {}", error.exception_type()),
    }
}
fn cell_step(
    step: &str,
    input: &mut SplitWhitespace<'_>,
    filter: &mut CellFilter<Recorder>,
) -> String {
    match step {
        "i" | "I" => return cell_inspect(input, filter, step == "I"),
        "s" => return cell_reset(input, filter),
        "S" => return cell_reset_array(input, filter),
        _ => {}
    }
    let target: i32 = number(input);
    let min: [f64; 3] = read_point(input);
    match step {
        "a" => filter.add(&target, &min),
        "A" => filter.add_range(&target, &min, &read_point(input)),
        "r" => filter.remove(&target, &min),
        _ => filter.remove_range(&target, &min, &read_point(input)),
    }
    String::new()
}
fn cell(input: &mut SplitWhitespace<'_>) -> String {
    let dimension: i32 = number(input);
    let size: f64 = number(input);
    let steps: usize = number(input);
    let mut filter: CellFilter<Recorder> = if dimension < 0 {
        CellFilter::new(size)
    } else {
        CellFilter::with_dimension(dimension as usize, size, None)
    };
    let mut out: String = String::new();
    for _ in 0..steps {
        let step: &str = input.next().expect("cell step");
        out.push_str(&cell_step(step, input, &mut filter));
    }
    out
}

type RadiusTree = KDTree<[f64; 2], 2, true>;

fn print_cloud(out: &mut String, tree: &RadiusTree) {
    out.push_str(&format!(" {} {}", tree.len(), u8::from(tree.is_empty())));
    if tree.is_empty() {
        return;
    }
    let mut gap: f64 = 0.0;
    let best: usize = tree.nearest_weighted(&[0.0, 0.0], Some(&mut gap));
    out.push_str(&format!(
        " {best} {} {} {}",
        gap.to_bits(),
        tree.radius(best).unwrap().to_bits(),
        tree.point(1).unwrap()[0].to_bits()
    ));
}
fn read_array<T>(
    input: &mut SplitWhitespace<'_>,
    mut item: impl FnMut(&mut SplitWhitespace<'_>) -> T,
) -> Array1<'static, T>
where
    T: Default + Clone,
{
    let lower: i32 = number(input);
    let count: i32 = number(input);
    let mut array: Array1<'static, T> = Array1::with_bounds(lower, lower + count - 1).unwrap();
    for value in array.as_mut_slice() {
        *value = item(input);
    }
    array
}
fn kd_array(input: &mut SplitWhitespace<'_>) -> String {
    let points: Array1<'static, [f64; 2]> =
        read_array(input, |input| [number(input), number(input)]);
    let radii: Array1<'static, f64> = read_array(input, number);
    let mut plain: KDTree<[f64; 2], 2> = KDTree::default();
    plain.build_array(&points);
    let mut out: String = format!("{} {}", plain.len(), plain.nearest_point(&[0.0, 0.0], None));
    let mut tree: RadiusTree = KDTree::default();
    if let Err(error) = tree.build_radii_array(&points, &radii) {
        out.push_str(&format!(" throw {}", error.exception_type()));
        return out;
    }
    print_cloud(&mut out, &tree);
    let mut moved: RadiusTree = KDTree::move_from(&mut tree);
    print_cloud(&mut out, &tree);
    print_cloud(&mut out, &moved);
    tree.move_assign(&mut moved);
    print_cloud(&mut out, &moved);
    print_cloud(&mut out, &tree);
    out
}
