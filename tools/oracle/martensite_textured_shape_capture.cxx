// Real AIS_TexturedShape + PrsMgr display, without selection or image codecs; OCCT3d097a0.
#include "martensite_text_culling_probe.hxx"
#include "martensite_textured_face_fixture.hxx"
#include <AIS_TexturedShape.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Image_PixMap.hxx>
#include <Prs3d_Drawer.hxx>
#include <Prs3d_IsoAspect.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <fstream>
#include <cstring>
static TopoDS_Face textured_capture_face(const std::string& mode) {
  const auto face = BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3()),-2.,4.,-3.,5.).Face();
  // Real meshing retains matching boundary polygons; replacing its mesh would invalidate their indices.
  if (mode=="face-boundaries-meshed") {
    BRepMesh_IncrementalMesh(face,.1,false,.5,false);
    return face;
  }
  BRep_Builder builder;
  builder.UpdateFace(face,native_textured_face_mesh(1),true);
  return face;
}
static occ::handle<Image_PixMap> textured_capture_pixels(bool topDown) {
  const occ::handle<Image_PixMap> pixels = new Image_PixMap;
  pixels->InitZero(Image_Format_RGBA,2,2); pixels->SetTopDown(topDown);
  const uint8_t colors[] = {255,0,0,255,0,255,0,255,0,0,255,255,255,255,255,255};
  std::memcpy(pixels->ChangeData(),colors,sizeof(colors));
  return pixels;
}
static void textured_capture_camera(const occ::handle<V3d_View>& view) {
  const auto camera = view->Camera();
  camera->SetEye(gp_Pnt(0,0,20)); camera->SetCenter(gp_Pnt(0,0,0)); camera->SetUp(gp_Dir(0,1,0));
  camera->SetScale(20); camera->SetAspect(1); camera->SetZRange(1,100);
  camera->SetProjectionType(Graphic3d_Camera::Projection_Orthographic);
  view->SetAutoZFitMode(false);view->SetBackgroundColor(Quantity_Color(0,0,1,Quantity_TOC_RGB));
  view->ChangeRenderingParams().NbMsaaSamples=0;
}
static void configure_textured_capture_mapping(AIS_TexturedShape& shape,const std::string& mode) {
  // map-off is a distinct native state operation, not a numeric UV scenario.
  if (mode=="map-off" || mode.rfind("map-",0) != 0) return;
  const int mapping = std::stoi(mode.substr(4));
  if (mapping < 0 || mapping > 7) throw Standard_Failure(("received mapping="+mode+"; expected map-0 through map-7").c_str());
  shape.SetTextureOrigin((mapping & 1)!=0,.25,-.5);
  shape.SetTextureRepeat((mapping & 2)!=0,2.,3.);
  shape.SetTextureScale((mapping & 4)!=0,.5,2.);
}
static void configure_textured_capture(AIS_TexturedShape& shape,const std::string& mode) {
  const occ::handle<Prs3d_Drawer> defaults = new Prs3d_Drawer;defaults->SetupOwnDefaults();
  shape.Attributes()->SetLink(defaults);shape.Attributes()->SetupOwnDefaults();
  shape.Attributes()->SetAutoTriangulation(false);shape.Attributes()->UIsoAspect()->SetNumber(0);
  shape.Attributes()->VIsoAspect()->SetNumber(0);shape.SetColor(Quantity_Color(Quantity_NOC_WHITE));
  shape.Attributes()->ShadingAspect()->Aspect()->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  shape.SetTexturePixMap(textured_capture_pixels(mode!="bottom-up"));
  configure_textured_capture_mapping(shape,mode);
  if (mode=="no-modulate") shape.DisableTextureModulate();
  if (mode=="map-off") shape.SetTextureMapOff();
  if (mode=="transparent") shape.SetTransparency(.5);
  if (mode=="triangles") shape.ShowTriangles(true);
  if (mode=="face-boundaries" || mode=="face-boundaries-meshed") shape.Attributes()->SetFaceBoundaryDraw(true);
}
static void mutate_textured_capture(AIS_TexturedShape& shape,const std::string& mode,
                                   const occ::handle<PrsMgr_PresentationManager>& owner) {
  if (mode=="red" || mode=="unset-red") shape.SetColor(Quantity_Color(Quantity_NOC_RED));
  if (mode=="unset-red") shape.UnsetColor();
  if (mode=="gold" || mode=="unset-gold") shape.SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));
  if (mode=="unset-gold") shape.UnsetMaterial();
  if (mode!="updated-pixels") return;
  const auto pixels = textured_capture_pixels(true);
  pixels->ChangeData()[0]=0;pixels->ChangeData()[1]=255;pixels->ChangeData()[2]=255;
  shape.SetTexturePixMap(pixels);owner->Update(&shape,3);
}
static void write_textured_capture(const occ::handle<V3d_View>& view,const std::string& prefix) {
  for (int index = 0;index<2;++index) {
    Image_PixMap pixels;const auto kind=index==0?Graphic3d_BT_RGBA:Graphic3d_BT_Depth;
    if (!view->ToPixMap(pixels,64,64,kind,false,Graphic3d_ZLayerId_Default,true))
      throw Standard_Failure(("received failed capture="+prefix+"; expected64x64nativebuffer").c_str());
    std::ofstream output(prefix+(index==0?".rgba":".depth"),std::ios::binary);
    for (size_t row=0;row<64;++row) output.write(reinterpret_cast<const char*>(pixels.Row(row)),256);
  }
}
static void run_textured_capture(const std::string& mode,const std::string& prefix) {
  const auto probe = create_policy_probe();textured_capture_camera(probe.view);
  const occ::handle<AIS_TexturedShape> shape = new AIS_TexturedShape(textured_capture_face(mode));
  configure_textured_capture(*shape,mode);
  const occ::handle<PrsMgr_PresentationManager> owner = new PrsMgr_PresentationManager(probe.viewer->StructureManager());
  const int displayMode=mode=="wireframe"?0:mode=="shaded"?1:mode=="bbox"?2:3;
  owner->Display(shape,displayMode);probe.view->Redraw();mutate_textured_capture(*shape,mode,owner);
  write_textured_capture(probe.view,prefix);
  std::cout << "MARTENSITE_TEXTURED_CAPTURE " << mode << ' ' << displayMode << ' '
    << shape->TexturePixMap()->IsTopDown() << '\n';
  probe.view->Remove();
}
int main(int argc,char** argv) {
  if (argc!=3) throw Standard_Failure(("received argc="+std::to_string(argc)+"; expected mode and capture prefix").c_str());
  run_textured_capture(argv[1],argv[2]);
  return 0;
}
