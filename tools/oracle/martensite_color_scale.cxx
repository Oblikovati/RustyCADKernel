// Local Martensite oracle for pinned OCCT color-scale mapping and actual drawing submissions.
#include <AIS_ColorScale.hxx>
#include <Prs3d_TextAspect.hxx>
#include <Prs3d_Drawer.hxx>
#include <Standard_Failure.hxx>
#include <Graphic3d_Buffer.hxx>
#include <Graphic3d_IndexBuffer.hxx>
#include <cstring>
#include <iomanip>
#include <iostream>
#include "tkservice_probe_null_driver.cxx"
struct ScaleRecorder {int caseId=0,groups=0;};
class ScaleGroup: public SvGroup {
  ScaleRecorder& recorder;
public:
  ScaleGroup(const occ::handle<Graphic3d_Structure>& structure,ScaleRecorder& record):SvGroup(structure),recorder(record){}
  void AddText(const occ::handle<Graphic3d_Text>& text,const bool) override {
    const auto& p=text->Orientation().Location();
    std::cout<<"text "<<recorder.caseId<<' '<<p.X()<<' '<<p.Y()<<' '<<text->Height()<<' '<<int(text->VerticalAlignment())<<' '<<text->HasOwnAnchorPoint()<<' '<<std::quoted(text->Text().ToCString())<<'\n';
  }
  void AddPrimitiveArray(const Graphic3d_TypeOfPrimitiveArray kind,const occ::handle<Graphic3d_IndexBuffer>& indices,const occ::handle<Graphic3d_Buffer>& attributes,const occ::handle<Graphic3d_BoundBuffer>&,const bool) override {
    std::cout<<"array "<<recorder.caseId<<' '<<int(kind)<<' '<<attributes->NbElements<<' '<<(indices.IsNull()?0:indices->NbElements)<<'\n';
    int index=-1;size_t stride=0;
    const uint8_t* positions=attributes->AttributeData(Graphic3d_TOA_POS,index,stride);
    int colorIndex=-1;size_t colorStride=0;
    const uint8_t* colors=attributes->AttributeData(Graphic3d_TOA_COLOR,colorIndex,colorStride);
    for(int i=0;i<attributes->NbElements;++i) {
      float p[3];std::memcpy(p,positions+i*stride,sizeof(p));
      std::cout<<"vertex "<<recorder.caseId<<' '<<int(kind)<<' '<<i<<' '<<p[0]<<' '<<p[1]<<' '<<p[2];
      if(colors) for(int channel=0;channel<4;++channel) std::cout<<' '<<int(colors[i*colorStride+channel]);
      std::cout<<'\n';
    }
    if(!indices.IsNull()) {std::cout<<"indices "<<recorder.caseId;for(int i=0;i<indices->NbElements;++i) std::cout<<' '<<indices->Index(i);std::cout<<'\n';}
  }
};
class ScaleStructure: public SvCStructure {
  ScaleRecorder& recorder;
public:
  ScaleStructure(const occ::handle<Graphic3d_StructureManager>& mgr,ScaleRecorder& record):SvCStructure(mgr),recorder(record){}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& structure) override {
    ++recorder.groups;occ::handle<Graphic3d_Group> group=new ScaleGroup(structure,recorder);myGroups.Append(group);return group;
  }
};
class ScaleDriver: public SvDriver {
  ScaleRecorder& recorder;
public:
  ScaleDriver(ScaleRecorder& record):recorder(record){}
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& mgr) override {return new ScaleStructure(mgr,recorder);}
};
static void rgb(const Quantity_Color& color) {std::cout<<' '<<color.Red()<<' '<<color.Green()<<' '<<color.Blue();}
static void mappings() {
  for(int count:{1,2,5,10}) for(double value:{-0.1,0.0,0.1,0.5,0.9,1.0,1.1}) {
    Quantity_Color color(Quantity_NOC_GREEN);bool ok=AIS_ColorScale::FindColor(value,0,1,count,color);
    std::cout<<"find "<<count<<' '<<value<<' '<<ok;rgb(color);std::cout<<'\n';
  }
  for(int count:{0,1,2,7,19}) for(double from:{-30.0,0.0,240.0}) {
    auto colors=AIS_ColorScale::MakeUniformColors(count,50,from,from+300);
    std::cout<<"uniform "<<count<<' '<<from<<' '<<colors.Length();for(const auto& color:colors)rgb(color);std::cout<<'\n';
  }
}
static void configure(const occ::handle<AIS_ColorScale>& scale,int id) {
  auto drawer=occ::handle<Prs3d_Drawer>(new Prs3d_Drawer());drawer->SetTextAspect(new Prs3d_TextAspect());scale->Attributes()->SetLink(drawer);scale->SetSize(90,id%5==0?50:241);scale->SetPosition(-3,7);
  scale->SetNumberOfIntervals(id%3+1);scale->SetLabelPosition(Aspect_TypeOfColorScalePosition(id%4));
  scale->SetSmoothTransition((id/4)%2);scale->SetReversed((id/8)%2);scale->SetLabelAtBorder((id/16)%2==0);
  if(id%3==1) scale->SetTitle("Pressure");
  if(id>=16) {scale->SetColorType(Aspect_TOCSD_USER);scale->SetIntervalColor(Quantity_NOC_RED,1);scale->SetIntervalColor(Quantity_NOC_GREEN,2);scale->SetIntervalColor(Quantity_NOC_BLUE,3);}
  if(id>=24) {scale->SetLabelType(Aspect_TOCSD_USER);scale->SetLabel("low",1);scale->SetLabel("",2);scale->SetLabel("high",3);scale->SetLabel("free",4);}
  if(id%7==0) scale->SetColor(Quantity_NOC_CYAN);
  if(id%6==0) scale->SetHueRange(60,300);
}
static void scene(int id) {
  ScaleRecorder recorder;recorder.caseId=id;
  occ::handle<AIS_ColorScale> scale=new AIS_ColorScale();configure(scale,id);
  occ::handle<Graphic3d_StructureManager> mgr=new Graphic3d_StructureManager(new ScaleDriver(recorder));
  occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(mgr);
  scale->Compute(occ::handle<PrsMgr_PresentationManager>(),prs,0);
  std::cout<<"groups "<<id<<' '<<recorder.groups<<'\n';
}
static void labels() {
  occ::handle<AIS_ColorScale> scale=new AIS_ColorScale();scale->SetNumberOfIntervals(4);
  for(int log=0;log<2;++log) for(int border=0;border<2;++border) {
    scale->SetRange(log?1:-2,log?1000:9);scale->SetLogarithmic(log);scale->SetLabelAtBorder(border);
    for(int i=0;i<6;++i) {auto label=scale->GetLabel(i);std::cout<<"label "<<log<<' '<<border<<' '<<i<<' '<<std::quoted(TCollection_AsciiString(label).ToCString())<<'\n';}
  }
  scale->SetLabelType(Aspect_TOCSD_USER);scale->SetLabel("last",3);
  for(int i=-1;i<6;++i) {std::cout<<"custom-label "<<i;try {auto label=scale->GetLabel(i);std::cout<<" ok "<<std::quoted(TCollection_AsciiString(label).ToCString());}catch(const Standard_Failure&){std::cout<<" error";}std::cout<<'\n';}
}
static void additionalMappings() {
  for(int count:{2,7}) for(double span:{0.0,-360.0,720.0}) {
    auto colors=AIS_ColorScale::MakeUniformColors(count,65,-25,-25+span);
    std::cout<<"uniform-extra "<<count<<' '<<span<<' '<<colors.Length();for(const auto& color:colors)rgb(color);std::cout<<'\n';
  }
  for(double value:{0.0,0.3,0.7,1.0}) {
    Quantity_Color color;bool ok=AIS_ColorScale::FindColor(value,0,1,4,NCollection_Vec3<double>(120,0.7,0.6),NCollection_Vec3<double>(-30,0.9,0.8),color);
    std::cout<<"find-hls "<<value<<' '<<ok;rgb(color);std::cout<<'\n';
  }
}
int main() {std::cout<<std::setprecision(17);mappings();labels();additionalMappings();for(int id=0;id<32;++id)scene(id);}
