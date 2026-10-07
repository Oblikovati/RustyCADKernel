//! The frustum culling of `Graphic3d_Layer`: `updateBVH` and `UpdateCulling`.
use super::bvh_cstructure_set::Graphic3dTrsfPersView;
use super::layer::Graphic3dLayer;
use super::{
    Graphic3dCStructure, Graphic3dCStructureMembers as _, Graphic3dCullingContext,
    Graphic3dCullingTool, Graphic3dRenderingParamsFrustumCulling as Culling,
};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Failure, Handle, WeakHandle};
use rustycad_tkmath::bvh::{BvhPrimitiveSetAccess as _, BvhTree, BvhVec3d};

/// `NodeInStack`: a node and whether it is completely inside.
#[derive(Clone, Copy, Default)]
struct NodeInStack {
    id: i32,
    is_full_inside: bool,
}

/// The culling inputs of one traversal.
struct CullingPass<'a> {
    selector: &'a Graphic3dCullingTool,
    ctx: Graphic3dCullingContext,
    view_id: i32,
}

impl CullingPass<'_> {
    /// `IsCulled` of a node box, clearing `inside` when the node is not fully inside.
    fn culled(&self, tree: &BvhTree<BvhVec3d>, node: &mut NodeInStack) -> Result<bool, Failure> {
        let (min, max) = (*tree.min_point(node.id)?, *tree.max_point(node.id)?);
        Ok(self
            .selector
            .is_culled(&self.ctx, &min, &max, Some(&mut node.is_full_inside)))
    }
    /// `IsTooSmall` of a node box.
    fn too_small(&self, tree: &BvhTree<BvhVec3d>, node: &NodeInStack) -> Result<bool, Failure> {
        let (min, max): (Vec3<f64>, Vec3<f64>) =
            (*tree.min_point(node.id)?, *tree.max_point(node.id)?);
        Ok(self.selector.is_too_small(&self.ctx, &min, &max))
    }
    /// Whether the two children are in, updating their "fully inside" flags.
    fn children_in(
        &self,
        tree: &BvhTree<BvhVec3d>,
        node: &NodeInStack,
        left: &mut NodeInStack,
        right: &mut NodeInStack,
    ) -> Result<(bool, bool), Failure> {
        if node.is_full_inside {
            return Ok((!self.too_small(tree, left)?, !self.too_small(tree, right)?));
        }
        let left_in = !self.culled(tree, left)?;
        left.is_full_inside &= left_in;
        let right_in = !self.culled(tree, right)?;
        right.is_full_inside &= right_in;
        Ok((left_in, right_in))
    }
}

impl Graphic3dLayer {
    /// `updateBVH`: after an invalidation, files every structure again as always rendered or
    /// in a BVH set; `layer.update_bvh()`.
    pub fn update_bvh(&self) {
        let structures = {
            let mut state = self.state.write();
            if !state.is_bvh_primitives_needs_reset {
                return;
            }
            state.always_rendered.clear(true);
            state.is_bvh_primitives_needs_reset = false;
            state
                .array
                .iter()
                .flat_map(|map| map.iter().cloned().collect::<Vec<_>>())
                .collect::<Vec<_>>()
        };
        let mut sets = self.bvh_sets();
        sets.primitives.clear();
        sets.trsf_pers.clear();
        for structure in structures.iter().map(WeakHandle::upgrade) {
            let Some(value) = structure.get() else {
                continue;
            };
            if value.is_always_rendered() {
                value.mark_as_not_culled();
                self.state
                    .write()
                    .always_rendered
                    .add(structure.downgrade());
            } else if value.transform_persistence().is_null() {
                sets.primitives.add(&structure);
            } else {
                sets.trsf_pers.add(&structure);
            }
        }
    }
    /// `UpdateCulling`: marks the structures of the BVH sets culled or not for a view.
    /// Example: `layer.update_culling(0, &selector, Graphic3dRenderingParamsFrustumCulling::On)?`.
    pub fn update_culling(
        &self,
        view_id: i32,
        selector: &Graphic3dCullingTool,
        state: Culling,
    ) -> Result<(), Failure> {
        self.update_bvh();
        let nb_structures = self.state.read().nb_structures;
        self.state.write().nb_structures_not_culled = nb_structures;
        if state != Culling::NoUpdate {
            let sets = self.bvh_sets();
            let all = sets
                .primitives
                .structures()
                .iter()
                .chain(sets.trsf_pers.structures().iter());
            for structure in all.map(WeakHandle::upgrade) {
                if let Some(structure) = structure.get() {
                    structure.set_culled(state == Culling::On);
                }
            }
        }
        let empty = {
            let sets = self.bvh_sets();
            sets.primitives.structures().extent() == 0 && sets.trsf_pers.structures().extent() == 0
        };
        if state != Culling::On || empty {
            return Ok(());
        }
        let always = self.state.read().always_rendered.extent() as i32;
        self.state.write().nb_structures_not_culled = always;
        let settings = self.layer_settings();
        let mut ctx = Graphic3dCullingContext::default();
        selector.set_culling_distance(&mut ctx, settings.culling_distance());
        selector.set_culling_size(&mut ctx, settings.culling_size());
        let pass = CullingPass {
            selector,
            ctx,
            view_id,
        };
        self.cull_tree(&pass, false)?;
        self.cull_tree(&pass, true)
    }
    /// Traverses the tree of one BVH set and marks its visible leaves as not culled.
    fn cull_tree(&self, pass: &CullingPass<'_>, is_trsf_pers: bool) -> Result<(), Failure> {
        let mut sets = self.bvh_sets();
        let tree = if is_trsf_pers {
            if sets.trsf_pers.structures().extent() == 0 {
                return Ok(());
            }
            let camera = pass.selector.camera().clone();
            let view = Graphic3dTrsfPersView {
                camera: &camera,
                projection: pass.selector.projection_matrix(),
                world_view: pass.selector.world_view_matrix(),
                viewport: [
                    pass.selector.viewport_width(),
                    pass.selector.viewport_height(),
                ],
                state: pass.selector.world_view_proj_state(),
            };
            sets.trsf_pers.bvh(&view)?.clone()
        } else {
            if sets.primitives.structures().extent() == 0 {
                return Ok(());
            }
            sets.primitives.bvh()?.clone()
        };
        let leaves = self.traverse(pass, &tree)?;
        for id in leaves {
            let structure = if is_trsf_pers {
                sets.trsf_pers.get_structure_by_id(id)
            } else {
                sets.primitives.get_structure_by_id(id)
            };
            self.mark_visible(&structure, pass.view_id);
        }
        drop(sets);
        Ok(())
    }
    /// Marks a visible structure of a reached leaf as not culled and counts it.
    fn mark_visible(&self, structure: &Handle<dyn Graphic3dCStructure>, view_id: i32) {
        if let Some(structure) = structure.get().filter(|s| s.is_visible_in_view(view_id)) {
            structure.mark_as_not_culled();
            self.state.write().nb_structures_not_culled += 1;
        }
    }
    /// The native stack traversal: the primitive indices of the reached leaves, in order;
    /// the queue order alternates between the left and the right child.
    fn traverse(
        &self,
        pass: &CullingPass<'_>,
        tree: &BvhTree<BvhVec3d>,
    ) -> Result<Vec<i32>, Failure> {
        let mut node = NodeInStack {
            id: 0,
            is_full_inside: true,
        };
        let mut leaves = Vec::new();
        if pass.culled(tree, &mut node)? {
            return Ok(leaves);
        }
        let mut stack: Vec<NodeInStack> = Vec::new();
        loop {
            if tree.is_outer(node.id)? {
                leaves.extend(tree.beg_primitive(node.id)?..=tree.end_primitive(node.id)?);
            } else if let Some(next) = self.descend(pass, tree, &node, &mut stack)? {
                node = next;
                continue;
            }
            match stack.pop() {
                Some(next) => node = next,
                None => break,
            }
        }
        Ok(leaves)
    }
    /// The next node below an inner node (`None` when both children are culled), pushing the
    /// second child when both are in.
    fn descend(
        &self,
        pass: &CullingPass<'_>,
        tree: &BvhTree<BvhVec3d>,
        node: &NodeInStack,
        stack: &mut Vec<NodeInStack>,
    ) -> Result<Option<NodeInStack>, Failure> {
        let mut left = NodeInStack {
            id: tree.child::<0>(node.id)?,
            is_full_inside: true,
        };
        let mut right = NodeInStack {
            id: tree.child::<1>(node.id)?,
            is_full_inside: true,
        };
        let (left_in, right_in) = pass.children_in(tree, node, &mut left, &mut right)?;
        if left_in && right_in {
            let mut state = self.state.write();
            let left_first = state.bvh_is_left_child_queued_first;
            stack.push(if left_first { right } else { left });
            state.bvh_is_left_child_queued_first = !left_first;
            return Ok(Some(if left_first { left } else { right }));
        }
        if left_in || right_in {
            return Ok(Some(if left_in { left } else { right }));
        }
        Ok(None)
    }
}
