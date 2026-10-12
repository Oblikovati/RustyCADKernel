// Actual native CView/V3d Camera field aliases survive self-assignment without added ownership.
#include "tkservice_probe_null_driver.cxx"
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <iostream>
static void camera_alias_row(const char* phase,const occ::handle<V3d_View>& view,
    const occ::handle<Graphic3d_Camera>& alias,const occ::handle<Graphic3d_Camera>& original) {
  std::cout<<phase<<' '<<(&alias==&view->Camera())<<' '<<(alias==original)<<' '<<original->GetRefCount()<<'\n';
}
int main() {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver);
  occ::handle<V3d_View> view=new V3d_View(viewer);
  const auto& alias=view->Camera();occ::handle<Graphic3d_Camera> original=alias;
  camera_alias_row("before",view,alias,original);
  view->SetCamera(alias);camera_alias_row("self",view,alias,original);
  view->SetCamera(original);camera_alias_row("same-target",view,alias,original);
  return 0;
}
