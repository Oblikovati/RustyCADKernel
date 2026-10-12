// OCCT3d097a0 OpenGl-compatible CPU shadows preserve parent geometry after destruction.
#include "martensite_camera_actions_oracle_shared.cxx"
static void shadow_lifecycle_snapshot(int phase,const occ::handle<Graphic3d_CStructure>& root) {
  std::cout<<phase<<' '<<root->GetRefCount()<<' '<<root->Groups().Length()<<' '<<root->BoundingBox().IsValid();
  std::cout<<' '<<root->Groups().First()->BoundingBox().IsValid()<<'\n';
}
int main() {
  auto scene=action_scene(0);auto manager=scene.scene.viewer->StructureManager();
  auto presentation=scene.context->MainPrsMgr()->Presentation(scene.primary,0,false);
  auto root=presentation->CStructure();shadow_lifecycle_snapshot(0,root);
  auto first=root->ShadowLink(manager);shadow_lifecycle_snapshot(1,root);
  auto second=first->ShadowLink(manager);shadow_lifecycle_snapshot(2,root);
  std::cout<<3<<' '<<first->Groups().Length()<<' '<<second->Groups().Length()<<' '<<first->BoundingBox().IsValid()<<' '<<second->BoundingBox().IsValid()<<'\n';
  first.Nullify();shadow_lifecycle_snapshot(4,root);
  second.Nullify();shadow_lifecycle_snapshot(5,root);
  return 0;
}
