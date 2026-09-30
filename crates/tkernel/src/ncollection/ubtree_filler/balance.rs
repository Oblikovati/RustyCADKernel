//! Native `CheckTree`: leaf count and depth balance report of the filled tree.
use super::{BoundingTreeInsertion, UBTreeFiller};
use crate::ncollection::{BoundingNode, BoundingVolume, UBTree};
use crate::standard::Failure;
use std::fmt::Write;

impl<T: Clone + Default, B: BoundingVolume, Tree: BoundingTreeInsertion<T, B> + ?Sized>
    UBTreeFiller<'_, T, B, Tree>
{
    /// Write `Checking UBTree:%8d leaves, balance =%7.2f` and a newline, as native
    /// `CheckTree`, and return the leaf count. The balance is
    /// `sqrt(sum(depth^2) / leaves) / log2(leaves)`; a one-leaf tree divides zero by zero
    /// and prints glibc's `-nan`. An empty tree, which natively dereferences a null root,
    /// is refused with Standard_NoSuchObject. Example: `filler.check_tree(&mut text)?`.
    pub fn check_tree(&self, stream: &mut impl Write) -> Result<i32, Failure> {
        let root: &BoundingNode<T, B> = self.tree.tree_view().root().ok_or_else(|| {
            Failure::with_type(
                "Standard_NoSuchObject",
                "UBTree is empty; expected a root node",
                "",
            )
        })?;
        let (length, number): (f64, i32) = depth_squares(self.tree.tree_view(), root);
        let count: f64 = f64::from(number);
        let spread: f64 = (length / count).sqrt();
        let depth: f64 = count.ln() / 2.0_f64.ln();
        writeln!(
            stream,
            "Checking UBTree:{number:8} leaves, balance ={}",
            printf_fixed(spread / depth, 7, 2)
        )
        .map_err(|_| Failure::new("CheckTree stream write failed"))?;
        Ok(number)
    }
}

/// Native `checkNode`: the sum of squared leaf depths, each branch adding child 0's
/// subtotal to child 1's, and the leaf count. Iterative so a degenerate chain cannot
/// overflow the call stack; the pairwise summation order is kept.
fn depth_squares<T: Clone + Default, B: BoundingVolume>(
    tree: &UBTree<T, B>,
    root: &BoundingNode<T, B>,
) -> (f64, i32) {
    let mut leaves: i32 = 0;
    let mut pending: Vec<(&BoundingNode<T, B>, i32, bool)> = vec![(root, 0, false)];
    let mut subtotals: Vec<f64> = Vec::new();
    while let Some((node, depth, children_done)) = pending.pop() {
        if node.is_leaf() {
            leaves += 1;
            subtotals.push(f64::from(depth * depth));
        } else if children_done {
            let second: f64 = subtotals.pop().unwrap();
            let first: f64 = subtotals.pop().unwrap();
            subtotals.push(first + second);
        } else {
            pending.push((node, depth, true));
            pending.push((tree.child(node, 1).unwrap(), depth + 1, false));
            pending.push((tree.child(node, 0).unwrap(), depth + 1, false));
        }
    }
    (subtotals.pop().unwrap(), leaves)
}

/// `%<width>.<precision>f` as glibc prints it, including `nan`/`-nan`/`inf` spellings.
fn printf_fixed(value: f64, width: usize, precision: usize) -> String {
    let text: String = if value.is_nan() {
        String::from(if value.is_sign_negative() {
            "-nan"
        } else {
            "nan"
        })
    } else if value.is_infinite() {
        String::from(if value < 0.0 { "-inf" } else { "inf" })
    } else {
        format!("{value:.precision$}")
    };
    format!("{text:>width$}")
}

#[cfg(test)]
mod tests {
    use super::printf_fixed;

    #[test]
    fn glibc_fixed_spellings() {
        assert_eq!(printf_fixed(1.005, 7, 2), "   1.00");
        assert_eq!(printf_fixed(-f64::NAN, 7, 2), "   -nan");
        assert_eq!(printf_fixed(f64::INFINITY, 7, 2), "    inf");
        assert_eq!(printf_fixed(123_456.789, 7, 2), "123456.79");
    }
}
