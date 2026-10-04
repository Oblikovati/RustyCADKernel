use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{BvhBoxSet, BvhTree, BvhTreeOwner, BvhVec3d};
pub(super) enum PairSets<'a> {
    Shared(&'a mut BvhBoxSet<BvhVec3d>),
    Distinct(&'a mut BvhBoxSet<BvhVec3d>, &'a mut BvhBoxSet<BvhVec3d>),
}
type BuiltTrees = (BvhTree<BvhVec3d>, Option<BvhTree<BvhVec3d>>);
impl PairSets<'_> {
    pub fn read(&self) -> (&BvhBoxSet<BvhVec3d>, &BvhBoxSet<BvhVec3d>) {
        match self {
            Self::Shared(set) => (set, set),
            Self::Distinct(first, second) => (first, second),
        }
    }
    pub fn take_trees(&mut self) -> Result<BuiltTrees, Failure> {
        match self {
            Self::Shared(set) => Ok((set.take_built_bvh()?, None)),
            Self::Distinct(first, second) => take_distinct(first, second),
        }
    }
    pub fn restore_trees(
        &mut self,
        first_tree: BvhTree<BvhVec3d>,
        second_tree: Option<BvhTree<BvhVec3d>>,
    ) {
        match self {
            Self::Shared(set) => set.restore_bvh(first_tree),
            Self::Distinct(first, second) => {
                first.restore_bvh(first_tree);
                second.restore_bvh(second_tree.expect("distinct set traversal has two trees"));
            }
        }
    }
}
fn take_distinct(
    first: &mut BvhBoxSet<BvhVec3d>,
    second: &mut BvhBoxSet<BvhVec3d>,
) -> Result<BuiltTrees, Failure> {
    let tree = first.take_built_bvh()?;
    match second.take_built_bvh() {
        Ok(second_tree) => Ok((tree, Some(second_tree))),
        Err(error) => {
            first.restore_bvh(tree);
            Err(error)
        }
    }
}
