// Real retained group and shared-driver view lifetimes; no expired context is inspected.
#define MARTENSITE_STENCIL_CAPTURE_NO_MAIN
#include "martensite_ordered_stencil_capture.cxx"
#undef MARTENSITE_STENCIL_CAPTURE_NO_MAIN
static NativeTextPolicyProbe stencil_adjacent_probe(const NativeTextPolicyProbe& first) {
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(first.viewer->Driver());
  const auto view=viewer->CreateView();
  const occ::handle<Xw_Window> window=new Xw_Window(viewer->Driver()->GetDisplayConnection(),"Martensite stencil lifecycle",0,0,64,64);
  window->SetVirtual(true);view->SetWindow(window);
  const auto glView=occ::handle<OpenGl_View>::DownCast(view->View());
  return {viewer,view,new OpenGl_Workspace(glView.get(),glView->GlWindow())};
}
static void stencil_lifecycle_row(const NativeTextPolicyProbe& probe,const char* stage) {
  const auto context=probe.workspace->GetGlContext();
  if(!context->MakeCurrent())throw Standard_Failure("current context=false; expected live lifecycle drawable");
  const auto gl=context->core11fwd;
  std::cout<<stage<<' '<<int(gl->glIsEnabled(GL_STENCIL_TEST));
  for(const GLenum query:{GL_STENCIL_FUNC,GL_STENCIL_REF,GL_STENCIL_VALUE_MASK,GL_STENCIL_WRITEMASK,
    GL_STENCIL_FAIL,GL_STENCIL_PASS_DEPTH_FAIL,GL_STENCIL_PASS_DEPTH_PASS}) {
    GLint value=0;gl->glGetIntegerv(query,&value);std::cout<<' '<<value;
  }
  std::cout<<'\n';
}
static void stencil_group_retirement(const NativeTextPolicyProbe& first) {
  std::cerr<<"LIFECYCLE_PHASE=first-geometry\n";
  const auto structure=oit_native_geometry(first.viewer,false);
  auto group=structure->Groups().First();group->SetStencilTestOptions(true);
  std::cerr<<"LIFECYCLE_PHASE=first-redraw\n";
  first.view->Redraw();stencil_lifecycle_row(first,"first-enabled");
  group->Clear(false);stencil_lifecycle_row(first,"after-clear-before-redraw");
  first.view->Redraw();stencil_lifecycle_row(first,"after-clear-redraw");
  group->Remove();group.Nullify();stencil_lifecycle_row(first,"after-remove-destroy");
  first.view->Redraw();stencil_lifecycle_row(first,"after-remove-redraw");
  // Structure::Remove nullifies its CStructure without erasing displayed view entries (3d097a0:101).
  // Erase first: later live-view teardown may otherwise dereference that deleted CStructure.
  structure->Erase();structure->Remove();stencil_lifecycle_row(first,"after-erase-remove");
}
static void stencil_view_retirement(NativeTextPolicyProbe& first) {
  auto adjacent=stencil_adjacent_probe(first);
  const auto plain=oit_native_geometry(adjacent.viewer,false);
  adjacent.view->Redraw();stencil_lifecycle_row(adjacent,"adjacent-first");
  const auto structure=oit_native_geometry(first.viewer,false);
  structure->Groups().First()->SetStencilTestOptions(true);
  first.view->Redraw();stencil_lifecycle_row(first,"first-reenabled");
  adjacent.view->Redraw();stencil_lifecycle_row(adjacent,"adjacent-after-first");
  first.view->Remove();adjacent.view->Redraw();stencil_lifecycle_row(adjacent,"adjacent-after-first-destroy");
  auto replacement=stencil_adjacent_probe(adjacent);
  const auto next=oit_native_geometry(replacement.viewer,false);
  replacement.view->Redraw();stencil_lifecycle_row(replacement,"replacement-first");
  replacement.view->Remove();adjacent.view->Remove();
}
int main() {
  std::cout.setf(std::ios::unitbuf);
  auto first=create_policy_probe();stencil_gl_identity(first);
  stencil_group_retirement(first);
  stencil_view_retirement(first);
  return 0;
}
