// Martensite native XR texture identity/delegation and actual Texture2D descriptor ancestry.
#include "martensite_xr_device_probe.hxx"
#include <Image_Texture.hxx>
#include <Image_PixMap.hxx>
#include <Image_SupportedFormats.hxx>
#include <Graphic3d_Texture2D.hxx>
#include <Graphic3d_Texture2Dplane.hxx>
#include <Graphic3d_TextureParams.hxx>
#include <type_traits>
class XrImageProbe:public Image_Texture {
public:
  mutable int calls=0;bool fail=false;occ::handle<Image_SupportedFormats> expected;mutable bool same=false;
  occ::handle<Image_PixMap> image=new Image_PixMap();
  XrImageProbe(int kind):Image_Texture(TCollection_AsciiString()) {
    // The fixture controls the protected identity without touching external I/O.
    if(kind!=0)myTextureId="xr/oracle-source";
  }
  occ::handle<Image_PixMap> ReadImage(const occ::handle<Image_SupportedFormats>& formats) const override {
    ++calls;same=formats==expected;
    if(fail)throw Standard_Failure("XR image fail=true; expected delegated image read");
    return image;
  }
};
static void XrTextureRow(int kind,int step,const occ::handle<XrImageProbe>& source,const occ::handle<Graphic3d_TextureMap>& texture,int failure,const occ::handle<Image_PixMap>& output) {
  auto params=texture->GetParams();auto descriptor=texture->DynamicType();
  std::cout<<kind<<' '<<step<<' '<<failure<<' '<<source->calls<<' '<<source->same<<' '<<(output==source->image);
  std::cout<<' '<<(texture->GetId()==source->TextureId())<<' '<<(!texture->GetId().IsEmpty())<<' '<<int(texture->Type());
  std::cout<<' '<<texture->IsColorMap()<<' '<<texture->HasMipmaps()<<' '<<params->IsModulate()<<' '<<params->IsRepeat()<<' '<<int(params->Filter())<<' '<<int(params->TextureUnit());
  std::cout<<' '<<descriptor->Name()<<' '<<descriptor->Parent()->Name()<<' '<<(descriptor==STANDARD_TYPE(Graphic3d_Texture2D))<<' '<<texture->IsKind(STANDARD_TYPE(Graphic3d_TextureMap))<<'\n';
}
static void XrTextureScenario(int kind) {
  auto source=occ::handle<XrImageProbe>(new XrImageProbe(kind));source->expected=new Image_SupportedFormats();
  auto device=occ::handle<XrProbeDevice>(new XrProbeDevice(XrModel(2),source));
  auto shading=device->Attributes()->ShadingAspect();auto texture=shading->Aspect()->TextureMap();
  std::cout<<"shading "<<kind<<' '<<device->Attributes()->HasOwnShadingAspect()<<' '<<shading->Aspect()->TextureMapState();
  auto color=shading->Color();std::cout<<' '<<color.Red()<<' '<<color.Green()<<' '<<color.Blue()<<' '<<int(shading->Material().Name())<<'\n';
  for(int step=0;step<4;++step) {
    source->fail=step==2;int failure=0;occ::handle<Image_PixMap> output;
    try {output=texture->GetImage(step==1?occ::handle<Image_SupportedFormats>():source->expected);}catch(const Standard_Failure&) {failure=1;}
    XrTextureRow(kind,step,source,texture,failure,output);
  }
}
static void XrDeviceRtti() {
  occ::handle<XrProbeDevice> device=new XrProbeDevice();occ::handle<AIS_InteractiveObject> interactive=device;
  auto first=AIS_XRTrackedDevice::get_type_descriptor();auto second=AIS_XRTrackedDevice::get_type_descriptor();
  std::cout<<"rtti "<<AIS_XRTrackedDevice::get_type_name()<<' '<<first->Name()<<' '<<first->Parent()->Name();
  std::cout<<' '<<(first==second)<<' '<<(interactive->DynamicType()==first)<<' '<<interactive->IsInstance(first)<<' '<<interactive->IsKind(STANDARD_TYPE(AIS_InteractiveObject));
  std::cout<<' '<<(occ::down_cast<AIS_XRTrackedDevice>(interactive)==device)<<' '<<std::is_same_v<AIS_XRTrackedDevice::base_type,AIS_InteractiveObject>;
  for(int mode=-1;mode<3;++mode)std::cout<<' '<<device->AcceptDisplayMode(mode);std::cout<<'\n';
}
static void XrPublicTextureRtti() {
  occ::handle<Graphic3d_Texture2D> plain=new Graphic3d_Texture2D("");occ::handle<Graphic3d_Texture2Dplane> plane=new Graphic3d_Texture2Dplane("");
  std::cout<<"texture-rtti "<<plain->DynamicType()->Name()<<' '<<plain->DynamicType()->Parent()->Name()<<' '<<plane->DynamicType()->Name()<<' '<<plane->DynamicType()->Parent()->Name();
  std::cout<<' '<<plain->IsKind(STANDARD_TYPE(Graphic3d_TextureMap))<<' '<<plane->IsKind(STANDARD_TYPE(Graphic3d_Texture2D))<<'\n';
}
int main() {
  std::cout<<std::setprecision(17);
  XrDeviceRtti();XrPublicTextureRtti();
  XrTextureScenario(0);XrTextureScenario(1);
  return 0;
}
