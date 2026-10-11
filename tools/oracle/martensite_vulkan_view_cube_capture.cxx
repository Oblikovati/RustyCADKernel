// Real default AIS_ViewCube display oracle; pinned OCCT 3d097a0, selection explicitly disabled.
#include <AIS_ViewCube.hxx>
#include <AIS_InteractiveContext.hxx>
#include <Aspect_DisplayConnection.hxx>
#include <OpenGl_GraphicDriver.hxx>
#include <OpenGl_View.hxx>
#include <OpenGl_Window.hxx>
#include <OpenGl_Context.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <Xw_Window.hxx>
#include <Image_PixMap.hxx>
#include <fstream>
#include <iostream>
static occ::handle<V3d_View> cube_view(const occ::handle<V3d_Viewer>& viewer,
                                     const occ::handle<Aspect_DisplayConnection>& display) {
  const auto view = viewer->CreateView();
  const occ::handle<Xw_Window> window = new Xw_Window(display, "Martensite ViewCube oracle", 0, 0, 320, 320);
  window->SetVirtual(true); view->SetWindow(window); view->SetImmediateUpdate(false);
  const auto camera = view->Camera();
  camera->SetEye(gp_Pnt(10, 8, 20)); camera->SetCenter(gp_Pnt(0, 0, 0));
  camera->SetUp(gp_Dir(0, 1, 0)); camera->SetScale(20);
  camera->SetProjectionType(Graphic3d_Camera::Projection_Orthographic);
  camera->SetAspect(1); camera->SetZRange(1, 100); view->SetAutoZFitMode(false);
  view->SetBackgroundColor(Quantity_Color(0, 0, 1, Quantity_TOC_RGB));
  view->ChangeRenderingParams().NbMsaaSamples = 0;
  return view;
}
static void cube_capture(const occ::handle<V3d_View>& view, const std::string& prefix) {
  for (const auto kind : {Graphic3d_BT_RGBA, Graphic3d_BT_Depth}) {
    Image_PixMap pixels;
    if (!view->ToPixMap(pixels, 320, 320, kind, false, Graphic3d_ZLayerId_BotOSD, false))
      throw Standard_Failure("native ViewCube capture failed; expected full-layer 320x320 color/depth");
    std::ofstream output(prefix + (kind == Graphic3d_BT_RGBA ? ".rgba" : ".depth"), std::ios::binary);
    for (int y = 0; y < 320; ++y) output.write(reinterpret_cast<const char*>(pixels.Row(y)), 320 * 4);
    if (!output) throw Standard_Failure("native ViewCube output failed; expected writable prefix");
  }
}
static void set_cube_labels(const occ::handle<AIS_InteractiveContext>& context,
                            const occ::handle<AIS_ViewCube>& cube, bool visible) {
  const occ::handle<AIS_ViewCube> defaults = new AIS_ViewCube;
  for (int side = 0; side < 6; ++side) {
    const auto orientation = V3d_TypeOfOrientation(side);
    cube->SetBoxSideLabel(orientation, visible ? defaults->BoxSideLabel(orientation) : TCollection_AsciiString());
  }
  cube->SetAxesLabels(visible ? "X" : "", visible ? "Y" : "", visible ? "Z" : "");
  context->RecomputePrsOnly(cube, false, false);
}
static void capture_cube_variants(const occ::handle<V3d_Viewer>& viewer,
                                  const occ::handle<V3d_View>& view,
                                  const occ::handle<AIS_InteractiveContext>& context,
                                  const occ::handle<AIS_ViewCube>& cube, const std::string& prefix) {
  cube_capture(view, prefix);
  set_cube_labels(context, cube, false);
  cube_capture(view, std::string(prefix) + "-unlabeled");
  set_cube_labels(context, cube, true);
  viewer->SetDefaultLights(); viewer->SetLightOn();
  cube_capture(view, std::string(prefix) + "-lit");
  set_cube_labels(context, cube, false);
  cube_capture(view, std::string(prefix) + "-lit-unlabeled");
}
static void print_cube_persistence(const occ::handle<AIS_ViewCube>& cube) {
  const auto persistence = cube->TransformPersistence();
  const auto offset = persistence->Offset2d();
  std::cout << "MARTENSITE_NATIVE_CUBE_PERSISTENCE=" << persistence->Mode()
            << ',' << persistence->Corner2d() << ',' << offset.x() << ',' << offset.y() << std::endl;
}
int main(int argc, char** argv) {
  if (argc != 2) throw Standard_Failure("native ViewCube arguments; expected capture prefix");
  const occ::handle<Aspect_DisplayConnection> display = new Aspect_DisplayConnection;
  const occ::handle<OpenGl_GraphicDriver> driver = new OpenGl_GraphicDriver(display);
  driver->ChangeOptions().buffersNoSwap = true;
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(driver);
  const auto view = cube_view(viewer, display);
  const auto glView = occ::handle<OpenGl_View>::DownCast(view->View());
  const auto gl = glView->GlWindow()->GetGlContext();
  std::cout << "OCCT_GL_RENDERER=" << gl->core11fwd->glGetString(GL_RENDERER) << std::endl;
  const occ::handle<AIS_InteractiveContext> context = new AIS_InteractiveContext(viewer);
  const occ::handle<AIS_ViewCube> cube = new AIS_ViewCube;
  context->Display(cube, 0, -1, false);
  std::cout << "MARTENSITE_NATIVE_CUBE_LAYER=" << cube->ZLayer() << std::endl;
  print_cube_persistence(cube);
  capture_cube_variants(viewer, view, context, cube, argv[1]);
  view->Remove();
}
