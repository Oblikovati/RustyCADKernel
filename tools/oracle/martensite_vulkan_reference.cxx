// Pinned OCCT OpenGL capability oracle: an unmapped virtual X11 window and
// offscreen capture.
#include <Aspect_DisplayConnection.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Graphic3d_Group.hxx>
#include <Graphic3d_Structure.hxx>
#include <Image_PixMap.hxx>
#include <OpenGl_Caps.hxx>
#include <OpenGl_GraphicDriver.hxx>
#include <TopLoc_Datum3D.hxx>
#include <V3d_View.hxx>
#include <V3d_Viewer.hxx>
#include <Xw_Window.hxx>
#include <fstream>
#include <iostream>
static void configure_camera(const occ::handle<V3d_View> &view, int width,
                             int height, int perspective) {
  auto camera = view->Camera();
  camera->SetEye(gp_Pnt(0, 0, 20));
  camera->SetCenter(gp_Pnt(0, 0, 0));
  camera->SetUp(gp_Dir(0, 1, 0));
  camera->SetProjectionType(perspective
                                ? Graphic3d_Camera::Projection_Perspective
                                : Graphic3d_Camera::Projection_Orthographic);
  camera->SetScale(20);
  camera->SetAspect(double(width) / height);
  camera->SetZRange(1, 100);
  view->SetAutoZFitMode(false);
  view->SetBackgroundColor(Quantity_Color(0, 0, 1, Quantity_TOC_RGB));
  auto &params = view->ChangeRenderingParams();
  params.NbMsaaSamples = 0;
  params.ShadingModel = Graphic3d_TypeOfShadingModel_Unlit;
}
static occ::handle<Graphic3d_AspectFillArea3d> unlit_aspects() {
  occ::handle<Graphic3d_AspectFillArea3d> aspect =
      new Graphic3d_AspectFillArea3d();
  aspect->SetInteriorStyle(Aspect_IS_SOLID);
  aspect->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  aspect->SetInteriorColor(Quantity_Color(1, 0, 0, Quantity_TOC_RGB));
  aspect->SetPolygonOffsets(Aspect_POM_None, 0.0f, 0.0f);
  return aspect;
}
static occ::handle<Graphic3d_ArrayOfTriangles> triangle_vertices() {
  occ::handle<Graphic3d_ArrayOfTriangles> vertices =
      new Graphic3d_ArrayOfTriangles(3, 3);
  vertices->AddVertex(-8.f, -8.f, 0.f);
  vertices->AddVertex(8.f, -8.f, 0.f);
  vertices->AddVertex(0.f, 8.f, 0.f);
  vertices->AddEdge(1);
  vertices->AddEdge(2);
  vertices->AddEdge(3);
  return vertices;
}
static occ::handle<Graphic3d_Structure>
triangle_scene(const occ::handle<V3d_Viewer> &viewer, double offset) {
  occ::handle<Graphic3d_Structure> structure =
      new Graphic3d_Structure(viewer->StructureManager());
  auto group = structure->NewGroup();
  group->SetGroupPrimitivesAspect(unlit_aspects());
  group->AddPrimitiveArray(triangle_vertices());
  gp_Trsf transform;
  transform.SetTranslation(gp_Vec(offset, 0, 0));
  structure->SetTransformation(new TopLoc_Datum3D(transform));
  structure->Display();
  return structure;
}
#include "martensite_vulkan_ais.cxx"
#include "martensite_vulkan_surfaces.cxx"
static void write_capture(const occ::handle<V3d_View> &view, int width,
                          int height, const std::string &prefix, bool full) {
  // The default-layer capture isolates geometry depth. The later native Topmost
  // layer clears depth (OpenGl_LayerList.cxx:827 at
  // 3d097a0328e71b826377d4814ab05ec3c3d23871), so a full-view final depth dump
  // measures the overlay pass instead.
  const int target =
      full ? Graphic3d_ZLayerId_BotOSD : Graphic3d_ZLayerId_Default;
  Image_PixMap color, depth;
  if (!view->ToPixMap(color, width, height, Graphic3d_BT_RGBA, false, target,
                      !full) ||
      !view->ToPixMap(depth, width, height, Graphic3d_BT_Depth, false, target,
                      !full))
    throw Standard_Failure("offscreen capture failed");
  std::ofstream rgba(prefix + ".rgba", std::ios::binary),
      z(prefix + ".depth", std::ios::binary);
  for (int y = 0; y < height; ++y) {
    rgba.write(reinterpret_cast<const char *>(color.Row(y)), width * 4);
    z.write(reinterpret_cast<const char *>(depth.Row(y)), width * 4);
  }
  if (!rgba || !z)
    throw Standard_Failure("capture files failed");
}
static void print_device(const occ::handle<V3d_View> &view) {
  NCollection_IndexedDataMap<TCollection_AsciiString, TCollection_AsciiString>
      info;
  view->DiagnosticInformation(info, Graphic3d_DiagnosticInfo_Device);
  for (int i = 1; i <= info.Size(); ++i) {
    std::cout << info.FindKey(i).ToCString() << ": "
              << info.FindFromIndex(i).ToCString() << "\n";
  }
}
static occ::handle<V3d_View>
create_view(const occ::handle<Aspect_DisplayConnection> &display,
            const occ::handle<V3d_Viewer> &viewer, int width, int height) {
  auto view = viewer->CreateView();
  occ::handle<Xw_Window> window = new Xw_Window(
      display, "Martensite offscreen oracle", 0, 0, width, height);
  window->SetVirtual(true);
  view->SetWindow(window);
  return view;
}
static void run_scene(int argc, char **argv) {
  const int width = std::stoi(argv[1]), height = std::stoi(argv[2]),
            perspective = std::stoi(argv[3]);
  const double offset = std::stod(argv[4]);
  occ::handle<Aspect_DisplayConnection> display =
      new Aspect_DisplayConnection();
  occ::handle<OpenGl_GraphicDriver> driver = new OpenGl_GraphicDriver(display);
  driver->ChangeOptions().buffersNoSwap = true;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(driver);
  auto view = create_view(display, viewer, width, height);
  configure_camera(view, width, height, perspective);
  const std::string mode = argc == 7 ? argv[6] : "default";
  if (mode.rfind("ais-", 0) == 0) {
    MartensiteOracleAisScene scene(viewer, offset);
    scene.transition(mode, view);
    write_capture(view, width, height, argv[5], false);
    print_device(view);
    scene.context->Erase(scene.object, false);
  } else {
    auto structure = triangle_scene(viewer, offset);
    if (mode.rfind("surface-", 0) == 0) {
      structure->Clear();
      auto group = structure->NewGroup();
      group->SetGroupPrimitivesAspect(unlit_aspects());
      group->AddPrimitiveArray(surface_vertices(mode));
    }
    write_capture(view, width, height, argv[5], mode == "full");
    print_device(view);
    structure->Erase();
  }
  view->Remove();
}
int main(int argc, char **argv) {
  if (argc != 6 && argc != 7) {
    std::cerr << "received " << argc
              << " arguments; expected width height perspective offset "
                 "output-prefix [full]\n";
    return 2;
  }
  try {
    run_scene(argc, argv);
    return 0;
  } catch (const Standard_Failure &error) {
    std::cerr << error.what() << "\n";
    return 1;
  } catch (const std::exception &error) {
    std::cerr << error.what() << "\n";
    return 1;
  }
}
