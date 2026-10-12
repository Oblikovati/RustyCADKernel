// Real native derived void failures stop traversal and preserve state before the inherited operation.
#define main martensite_view_source_probe_entry
#include "martensite_v3d_view_source.cxx"
#undef main
static void capture_view_failure(int operation) {
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver);std::vector<std::string> rows;
  auto view=mapped_probe(viewer,rows);occ::handle<Graphic3d_ClipPlane> plane=new Graphic3d_ClipPlane;
  view->AddClipPlane(plane);view->InvalidateImmediate();rows.clear();view->failure=operation;
  try {
    if(operation==1)viewer->Redraw();else if(operation==2)viewer->RedrawImmediate();
    else if(operation==3)view->AddClipPlane(plane);else view->RemoveClipPlane(plane);
  }catch(const Standard_Failure& failure) {
    std::cout<<"failure "<<operation<<' '<<failure.what()<<'\n';
  }
  std::cout<<"state "<<operation<<' '<<view->IsInvalidatedImmediate()<<' '<<view->ClipPlanes()->Size()<<' '<<view->GetRefCount()<<' '<<plane->GetRefCount()<<'\n';
  for(const auto& row:rows)std::cout<<row<<'\n';
}
int main() {
  for(int operation=1;operation<=4;++operation)capture_view_failure(operation);
  return 0;
}
