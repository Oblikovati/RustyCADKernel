// Retained native const Subviews sequence reference observes declared public add/remove operations.
#include "martensite_camera_primitives_oracle_shared.cxx"
#include <array>
struct SubviewReferenceScene {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new PrimitiveRecordingDriver);
  occ::handle<V3d_View> parent=new V3d_View(viewer);
  std::array<occ::handle<V3d_View>,3> children;
  SubviewReferenceScene() {
    for(auto& child:children)child=new V3d_View(viewer);
  }
};
static int reference_child_identity(const occ::handle<V3d_View>& child,const SubviewReferenceScene& scene) {
  for(int index=0;index<3;++index)if(child==scene.children[index])return index+1;
  return 0;
}
static int reference_middle_identity(const NCollection_Sequence<occ::handle<V3d_View>>& sequence,
    const SubviewReferenceScene& scene) {
  try {return reference_child_identity(sequence.Value(2),scene);}
  catch(const Standard_OutOfRange&) {return -1;}
}
static void emit_subview_reference(int phase,const SubviewReferenceScene& scene,
    const NCollection_Sequence<occ::handle<V3d_View>>& alias) {
  int first=alias.IsEmpty()?0:reference_child_identity(alias.First(),scene);
  int last=alias.IsEmpty()?0:reference_child_identity(alias.Last(),scene);
  std::cout<<phase<<' '<<(&alias==&scene.parent->Subviews())<<' '<<alias.Lower()<<' '
    <<alias.Upper()<<' '<<alias.Length()<<' '<<alias.IsEmpty()<<' '<<first<<' '<<last<<' '
    <<reference_middle_identity(alias,scene);
  for(const auto& child:scene.children)std::cout<<' '<<child->GetRefCount();
  std::cout<<'\n';
}
static void run_subview_reference() {
  SubviewReferenceScene scene;const auto& alias=scene.parent->Subviews();
  emit_subview_reference(0,scene,alias);
  for(const auto& child:scene.children)scene.parent->AddSubview(child);
  emit_subview_reference(1,scene,alias);
  scene.parent->RemoveSubview(scene.children[1].get());emit_subview_reference(2,scene,alias);
  scene.parent->RemoveSubview(scene.children[0].get());emit_subview_reference(3,scene,alias);
  scene.parent->RemoveSubview(scene.children[2].get());emit_subview_reference(4,scene,alias);
}
int main() {
  run_subview_reference();
  return 0;
}
