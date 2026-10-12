// Native subview Sequence node/handle aliases remain live across valid redraw mutations.
#include "martensite_camera_primitives_oracle_shared.cxx"
class SubviewNodeMutation : public V3d_View {
public:
  int identity,policy=0;
  occ::handle<V3d_View>* parent=nullptr;
  occ::handle<V3d_View>* removed=nullptr;
  occ::handle<V3d_View>* appended=nullptr;
  SubviewNodeMutation(const occ::handle<V3d_Viewer>& viewer,int value):V3d_View(viewer),identity(value) {}
  void Redraw() const override {
    V3d_View::Redraw();
    if(policy==1) {(*parent)->RemoveSubview(removed->get());(*parent)->AddSubview(*appended);}
    if(policy==2) {
      // The original owned nonconst Sequence permits value mutation without unlinking the current node.
      auto& sequence=const_cast<NCollection_Sequence<occ::handle<V3d_View>>&>((*parent)->Subviews());
      sequence.ChangeFirst()=*removed;
    }
  }
};
static int subview_identity(const occ::handle<V3d_View>& view) {
  return static_cast<SubviewNodeMutation*>(view.get())->identity;
}
static occ::handle<V3d_View> subview_node_view(const occ::handle<V3d_Viewer>& viewer,int identity) {
  // Actual native view ownership and viewer registration provide the observed intrusive counts.
  return new SubviewNodeMutation(viewer,identity);
}
static void emit_subview_node(int policy,int before,const occ::handle<V3d_View>& slot,
    const void* address,const occ::handle<V3d_View>& parent) {
  std::cout<<policy<<' '<<before<<' '<<subview_identity(slot)<<' '<<(&slot==address)<<' '
    <<slot->GetRefCount()<<' '<<parent->Subviews().Length()<<'\n';
}
static void run_subview_nodes(int policy) {
  auto scene=primitive_scene(0);occ::handle<V3d_View> parent=scene.view;
  auto first=subview_node_view(scene.viewer,1),second=subview_node_view(scene.viewer,2);
  auto third=subview_node_view(scene.viewer,3),fourth=subview_node_view(scene.viewer,4);
  parent->AddSubview(first);parent->AddSubview(second);parent->AddSubview(third);
  auto* mutation=static_cast<SubviewNodeMutation*>(first.get());mutation->policy=policy;
  mutation->parent=&parent;mutation->removed=&second;mutation->appended=&fourth;
  for(NCollection_Sequence<occ::handle<V3d_View>>::Iterator cursor(parent->Subviews());cursor.More();cursor.Next()) {
    const auto& slot=cursor.Value();const void* address=&slot;int before=subview_identity(slot);
    slot->Redraw();emit_subview_node(policy,before,slot,address,parent);
  }
}
int main() {
  run_subview_nodes(1);run_subview_nodes(2);
  return 0;
}
