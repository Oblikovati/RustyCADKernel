#include "martensite_surface_texture_probe.hxx"
int main(int argc,char** argv) {
  if(argc!=3)throw Standard_Failure("surface texture arguments; expected fixture mode and capture prefix");
  auto probe=create_policy_probe();texture_view_fixture(probe.view,argv[1]);
  const auto texture=texture_source(argv[1]);
  const auto structure=texture_geometry(probe.view->Viewer(),texture,argv[1]);
  probe.view->Redraw();texture_capture_mutation(probe.view,structure,texture,argv[1],argv[2]);probe.view->Remove();
}
