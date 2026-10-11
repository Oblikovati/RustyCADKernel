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
  // POM_None inherits the layer's (1, 1) bias; Martensite fixtures explicitly
  // use POM_Off.
  aspect->SetPolygonOffsets(Aspect_POM_Off, 0.0f, 0.0f);
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
#include "martensite_vulkan_camera_animation.cxx"
#include "martensite_vulkan_ais.cxx"
#include "martensite_vulkan_object_animation.cxx"
#include "martensite_vulkan_ais_line.cxx"
#include "martensite_vulkan_ais_point.cxx"
#include "martensite_vulkan_text.cxx"
#include "martensite_vulkan_ais_axis.cxx"
#include "martensite_vulkan_ais_circle.cxx"
#include "martensite_vulkan_plane_trihedron.cxx"
#include "martensite_vulkan_lines.cxx"
#include "martensite_vulkan_persistence.cxx"
#include "martensite_vulkan_stdprs.cxx"
#include "martensite_vulkan_surfaces.cxx"
#include "martensite_vulkan_clipping.cxx"
#include "martensite_vulkan_homogeneous.cxx"
#include "martensite_vulkan_capping.cxx"
#include "martensite_vulkan_group_flipping.cxx"
#include "martensite_vulkan_ais_text_label.cxx"
#include "martensite_vulkan_plane_presentations.cxx"
#include "martensite_vulkan_ais_plane.cxx"
#include "martensite_vulkan_quadrangles.cxx"
#include "martensite_vulkan_lighting.cxx"
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
#include "martensite_vulkan_window.cxx"
static void run_scene(int argc, char **argv) {
  const int width = std::stoi(argv[1]), height = std::stoi(argv[2]),
            perspective = std::stoi(argv[3]);
  const double offset = std::stod(argv[4]);
  const std::string mode = argc >= 7 ? argv[6] : "default";
  occ::handle<Aspect_DisplayConnection> display =
      new Aspect_DisplayConnection();
  occ::handle<OpenGl_GraphicDriver> driver = new OpenGl_GraphicDriver(display);
  driver->ChangeOptions().buffersNoSwap = mode.rfind("window-",0)!=0;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(driver);
  auto view = create_view(display, viewer, width, height);
  configure_camera(view, width, height, perspective);
  if (mode.rfind("window-",0)==0) {
    run_window_scene(display,driver,viewer,view,width,height,offset,mode,argv[5]);
    return;
  }
  const bool rasterControl = raster_control_option(argc, argv);
  if(mode.rfind("aisplane-",0)==0) {
    MartensiteOracleAisPlaneScene scene(viewer,mode,offset);
    if(mode=="aisplane-detected")scene.detect(view);
    write_capture(view,width,height,argv[5],mode=="aisplane-detected");print_device(view);view->Remove();return;
  }
  if(mode.rfind("quad-",0)==0) {
    auto scene=quadrangle_probe_scene(viewer,mode,offset);
    write_capture(view,width,height,argv[5],mode=="quad-layers"||mode=="quad-clear-depth");print_device(view);view->Remove();return;
  }
  if (mode.rfind("prsplane-",0)==0) {
    auto scene=plane_presentation_scene(viewer,mode,offset);
    write_capture(view,width,height,argv[5],false); print_device(view);
    view->Remove(); return;
  }
  if (mode.rfind("label-",0)==0) {
    MartensiteOracleLabelScene scene(viewer,view,mode,offset);
    write_capture(view,width,height,argv[5],false); print_device(view);
    scene.context->Remove(scene.label,false); view->Remove(); return;
  }
  if (mode.rfind("flip-",0)==0) {
    auto structure=triangle_scene(viewer,offset);
    group_flipping_scene(structure,view,mode);
    write_capture(view,width,height,argv[5],false);
    print_device(view); view->Remove(); return;
  }
  if (mode.rfind("text-", 0) == 0) {
    MartensiteOracleTextScene scene(viewer, offset);
    scene.apply(mode, view);
    write_capture(view, width, height, argv[5], false);
    print_device(view);
    return;
  }
  if (mode.rfind("plane-", 0) == 0) {
    MartensiteOraclePlaneScene scene(viewer, offset, mode);
    scene.transition(mode, view);
    write_capture(view, width, height, argv[5], mode == "plane-detected");
    print_device(view);
    scene.context->Remove(scene.object, false);
    return;
  }
  if (mode.rfind("circle-", 0) == 0) {
    MartensiteOracleCircleScene scene(viewer, offset, mode);
    scene.transition(mode, view);
    write_capture(view, width, height, argv[5], mode == "circle-detected" || mode == "circle-filled-detected");
    print_device(view);
    scene.context->Remove(scene.circle, false);
    view->Remove();
    return;
  }
  if (mode.rfind("axis-", 0) == 0) {
    MartensiteOracleAxisScene scene(viewer, offset, mode);
    scene.transition(mode, view);
    write_capture(view, width, height, argv[5], mode == "axis-detected");
    print_device(view);
    scene.context->Remove(scene.axis, false);
    return;
  }
  if (mode.rfind("animobj-", 0) == 0) {
    MartensiteOracleAisScene scene(viewer, offset);
    object_animation_scene(scene, view, mode, offset);
    write_capture(view, width, height, argv[5], false);
    print_device(view);
    scene.context->Erase(scene.object, false);
  } else if (mode.rfind("point-", 0) == 0) {
    MartensiteOraclePointScene scene(viewer, offset);
    scene.transition(mode, view);
    write_capture(view, width, height, argv[5], mode == "point-detected");
    print_device(view);
    scene.context->Remove(scene.point, false);
  } else if (mode.rfind("datum-", 0) == 0) {
    MartensiteOracleLineScene scene(viewer, offset, mode == "datum-infinite");
    scene.transition(mode, view);
    write_capture(view, width, height, argv[5], mode == "datum-detected");
    print_device(view);
    scene.context->Remove(scene.line, false);
  } else if (mode.rfind("ais-", 0) == 0) {
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
    if (mode.rfind("line-", 0) == 0)
      line_scene(structure, mode);
    if (mode.rfind("stdprs-", 0) == 0)
      stdprs_scene(structure, mode);
    if (mode.rfind("lit-", 0) == 0)
      lighting_scene(structure, view, mode);
    if (mode.rfind("cap-", 0) == 0)
      capping_scene(structure, view, mode);
    if (mode.rfind("hom-", 0) == 0)
      homogeneous_scene(structure, view, mode);
    if (mode.rfind("clip-", 0) == 0)
      clipping_scene(structure, view, mode);
    if (mode.rfind("pers-", 0) == 0)
      persistence_scene(structure, view, mode, rasterControl);
    if (mode.rfind("animcam-", 0) == 0)
      camera_animation_scene(view, mode);
    auto capShadow = mode == "cap-shadow" ? cap_shadow(structure, viewer) : occ::handle<Graphic3d_Structure>();
    write_capture(view, width, height, argv[5], mode == "full");
    print_device(view);
    structure->Erase();
  }
  if (rasterControl)
    std::cout << "MARTENSITE_RASTER_CONTROL=8\n";
  view->Remove();
}
int main(int argc, char **argv) {
  if (argc != 6 && argc != 7 && argc != 8) {
    std::cerr << "received " << argc
              << " arguments; expected width height perspective offset "
                 "output-prefix [mode] [raster8]\n";
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
