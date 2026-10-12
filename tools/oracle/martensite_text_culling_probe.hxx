// Actual workspace/GL policy oracle for Martensite; OCCT 3d097a0. Replay tests are CPU-only.
#include <Aspect_DisplayConnection.hxx>
#include <Graphic3d_AspectText3d.hxx>
#include <OpenGl_Aspects.hxx>
#include <OpenGl_Context.hxx>
#include <OpenGl_GraphicDriver.hxx>
#include <OpenGl_View.hxx>
#include <OpenGl_Window.hxx>
#include <OpenGl_Workspace.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <Xw_Window.hxx>
#include <iostream>
struct NativeTextPolicyProbe {
  // Checked main 700823fd1: V3d_View borrows its viewer, which must outlive both borrowers.
  occ::handle<V3d_Viewer> viewer;
  occ::handle<V3d_View> view;
  occ::handle<OpenGl_Workspace> workspace;
};
static NativeTextPolicyProbe create_policy_probe() {
  const occ::handle<Aspect_DisplayConnection> display = new Aspect_DisplayConnection;
  const occ::handle<OpenGl_GraphicDriver> driver = new OpenGl_GraphicDriver(display);
  driver->ChangeOptions().buffersNoSwap = true;
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(driver);
  const occ::handle<V3d_View> view = viewer->CreateView();
  const occ::handle<Xw_Window> window = new Xw_Window(display, "Martensite policy oracle", 0, 0, 64, 64);
  window->SetVirtual(true);
  view->SetWindow(window);
  const auto glView = occ::handle<OpenGl_View>::DownCast(view->View());
  const auto context = glView->GlWindow()->GetGlContext();
  std::cerr << "OCCT_GL_RENDERER=" << context->core11fwd->glGetString(GL_RENDERER) << std::endl;
  return {viewer, view, new OpenGl_Workspace(glView.get(), glView->GlWindow())};
}
