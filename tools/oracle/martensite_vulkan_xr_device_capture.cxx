// Actual AIS_XRTrackedDevice context oracle; OCCT3d097a0. No synthetic presentation buffers.
#include "martensite_text_culling_probe.hxx"
#include <AIS_XRTrackedDevice.hxx>
#include <AIS_InteractiveContext.hxx>
#include <Image_Texture.hxx>
#include <Image_PixMap.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_Group.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <fstream>
struct XrNativeView {
  occ::handle<V3d_Viewer> viewer;
  occ::handle<V3d_View> view;
};
static XrNativeView xr_native_view() {
  const occ::handle<Aspect_DisplayConnection> display=new Aspect_DisplayConnection;
  const occ::handle<OpenGl_GraphicDriver> driver=new OpenGl_GraphicDriver(display);
  driver->ChangeOptions().buffersNoSwap=true;
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(driver);
  const auto view=viewer->CreateView();
  const occ::handle<Xw_Window> window=new Xw_Window(display,"Martensite XR oracle",0,0,64,64);
  window->SetVirtual(true);view->SetWindow(window);
  const auto glView=occ::handle<OpenGl_View>::DownCast(view->View());
  std::cerr<<"OCCT_GL_RENDERER="<<glView->GlWindow()->GetGlContext()->core11fwd->glGetString(GL_RENDERER)<<std::endl;
  // V3d_View has a raw viewer pointer: retain its actual owner through capture, never freed state.
  return {viewer,view};
}
class XrCaptureImage : public Image_Texture {
  occ::handle<Image_PixMap> myImage;
public:
  XrCaptureImage():Image_Texture("Martensite-XR-capture"),myImage(new Image_PixMap) {
    myImage->InitZero(Image_Format_RGBA,2,2);myImage->SetTopDown(true);
    const uint8_t pixels[]={255,0,0,255,0,255,0,255,0,0,255,255,255,255,255,255};
    std::memcpy(myImage->ChangeData(),pixels,sizeof(pixels));
  }
  occ::handle<Image_PixMap> ReadImage(const occ::handle<Image_SupportedFormats>&)const override {
    // Named source keeps codec and filesystem availability out of the differential oracle.
    return myImage;
  }
};
static occ::handle<Graphic3d_ArrayOfTriangles> xr_capture_mesh() {
  const occ::handle<Graphic3d_ArrayOfTriangles> mesh=new Graphic3d_ArrayOfTriangles(4,6,true,false,true);
  const float points[][2]={{-5,-5},{5,-5},{5,5},{-5,5}},uv[][2]={{0,0},{1,0},{1,1},{0,1}};
  for(int i=0;i<4;++i)mesh->AddVertex(gp_Pnt(points[i][0],points[i][1],0),gp_Dir(0,0,1),gp_Pnt2d(uv[i][0],uv[i][1]));
  for(int rank:{1,2,3,1,3,4})mesh->AddEdge(rank);
  return mesh;
}
static occ::handle<AIS_XRTrackedDevice> xr_capture_device(const std::string& mode) {
  const occ::handle<Image_Texture> image=mode=="texture"?occ::handle<Image_Texture>(new XrCaptureImage):occ::handle<Image_Texture>();
  const occ::handle<AIS_XRTrackedDevice> device=mode=="axes"?new AIS_XRTrackedDevice:new AIS_XRTrackedDevice(xr_capture_mesh(),image);
  device->SetUnitFactor(30);device->SetInfiniteState(mode.rfind("infinite",0)==0);
  device->SetLaserColor(Quantity_Color(Quantity_NOC_GREEN));
  if(mode.find("ray")!=std::string::npos||mode.find("zero")!=std::string::npos||mode=="color-update")device->SetLaserLength(8);
  if(mode=="axes")return device;
  device->Attributes()->ShadingAspect()->Aspect()->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  device->Attributes()->ShadingAspect()->Aspect()->SetPolygonOffsets(Aspect_POM_Off,0,0);
  return device;
}
static void xr_capture_camera(const occ::handle<V3d_View>& view) {
  const auto camera=view->Camera();camera->SetEyeAndCenter(gp_Pnt(20.125,.125,20),gp_Pnt(.125,.125,0));
  camera->SetUp(gp_Dir(0,1,0));camera->SetScale(20);camera->SetAspect(1);camera->SetZRange(1,1000);
  camera->SetProjectionType(Graphic3d_Camera::Projection_Orthographic);view->SetAutoZFitMode(false);
  view->SetBackgroundColor(Quantity_Color(0,0,1,Quantity_TOC_RGB));view->ChangeRenderingParams().NbMsaaSamples=0;
}
static void xr_capture_dump(const occ::handle<V3d_View>& view,const std::string& prefix) {
  for(auto kind:{Graphic3d_BT_RGBA,Graphic3d_BT_Depth}) {
    Image_PixMap pixels;
    if(!view->ToPixMap(pixels,64,64,kind,false,Graphic3d_ZLayerId_Default,true))throw Standard_Failure("XR capture size64; expected tight RGBA/depth");
    std::ofstream output(prefix+(kind==Graphic3d_BT_RGBA?".rgba":".depth"),std::ios::binary);
    for(int row=0;row<64;++row)output.write(reinterpret_cast<const char*>(pixels.Row(row)),256);
  }
}
static void xr_capture_mutate(const occ::handle<AIS_XRTrackedDevice>& device,const std::string& mode) {
  if(mode.find("zero")!=std::string::npos)device->SetLaserLength(0);
  if(mode=="color-update")device->SetLaserColor(Quantity_Color(Quantity_NOC_RED));
  // Native setter updates retained groups synchronously; no context recompute is substituted.
}
int main(int argc,char** argv) {
  if(argc!=3)throw Standard_Failure("XR capture argc; expected mode and prefix");
  auto probe=xr_native_view();xr_capture_camera(probe.view);
  const occ::handle<AIS_InteractiveContext> context=new AIS_InteractiveContext(probe.view->Viewer());
  const auto device=xr_capture_device(argv[1]);context->Display(device,0,0,false);
  probe.view->Redraw();xr_capture_mutate(device,argv[1]);xr_capture_dump(probe.view,argv[2]);probe.view->Remove();
}
