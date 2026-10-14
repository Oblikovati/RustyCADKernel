// OCCT 3d097a0 actual MeshVS_Tool: retained fixtures never run this oracle in Rust tests.
#include <MeshVS_Tool.hxx>
#include <MeshVS_Drawer.hxx>
#include <MeshVS_DrawerAttribute.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Graphic3d_AspectLine3d.hxx>
#include <Graphic3d_AspectMarker3d.hxx>
#include <Graphic3d_AspectText3d.hxx>
#include <Graphic3d_HatchStyle.hxx>
#include <Standard_Failure.hxx>
#include <Font_FontMgr.hxx>
#include <gp_Vec.hxx>
#include <iostream>
#include <cstring>
#include <vector>
#include <limits>
using Drawer=occ::handle<MeshVS_Drawer>;
using Aspects=occ::handle<Graphic3d_Aspects>;
static void Real(double value) {
  uint64_t bits; std::memcpy(&bits,&value,sizeof(bits)); std::cout << ' ' << bits;
}
static void Color(const Quantity_Color& value) {
  Real(value.Red()); Real(value.Green()); Real(value.Blue());
}
static Aspects Build(int kind,const Drawer& drawer,bool defaults) {
  Graphic3d_MaterialAspect material(Graphic3d_NameOfMaterial_Gold);
  material.SetTransparency(.375f);
  switch(kind) {
    case 0:return MeshVS_Tool::CreateAspectFillArea3d(drawer,defaults);
    case 1:return MeshVS_Tool::CreateAspectFillArea3d(drawer,material,defaults);
    case 2:return MeshVS_Tool::CreateAspectLine3d(drawer,defaults);
    case 3:return MeshVS_Tool::CreateAspectMarker3d(drawer,defaults);
    default:return MeshVS_Tool::CreateAspectText3d(drawer,defaults);
  }
}
static void FillSnapshot(const Aspects& value) {
  std::cout << ' ' << int(value->InteriorStyle());
  Color(value->InteriorColor()); Color(value->BackInteriorColor()); Color(value->EdgeColor());
  std::cout << ' ' << int(value->LineType()); Real(value->EdgeWidth());
  std::cout << ' ' << (value->HatchStyle().IsNull()?-1:value->HatchStyle()->HatchType());
  std::cout << ' ' << int(value->FrontMaterial().Name()) << ' ' << int(value->BackMaterial().Name());
  Real(value->FrontMaterial().Transparency()); Real(value->BackMaterial().Transparency());
}
static void FontSnapshot(const Aspects& value) {
  std::cout << ' ' << int(value->TextStyle()) << ' ' << int(value->TextDisplayType())
    << ' ' << int(value->TextFontAspect()) << ' ';
  if(value->TextFont().IsNull()) {std::cout << '-';return;}
  const unsigned char* bytes=(const unsigned char*)value->TextFont()->ToCString();
  for(;*bytes;++bytes) std::cout << std::hex << int(*bytes) << ':';
  std::cout << std::dec;
}
static void Snapshot(int kind,const Aspects& value) {
  if(value.IsNull()) {std::cout << " null";return;}
  std::cout << " value";
  if(kind<2) {FillSnapshot(value);return;}
  Color(value->InteriorColor());
  if(kind==2) {
    std::cout << ' ' << int(value->LineType()); Real(value->LineWidth());
    std::cout << ' ' << value->LinePattern();
  } else if(kind==3) {std::cout << ' ' << int(value->MarkerType()); Real(value->MarkerScale());}
  else FontSnapshot(value);
}
static void Row(const char* label,int kind,int index,const Drawer& drawer,bool defaults) {
  std::cout << label << ' ' << kind << ' ' << index;
  try {Snapshot(kind,Build(kind,drawer,defaults));}
  catch(const Standard_Failure& error) {std::cout << " failure " << error.ExceptionType();}
  std::cout << ' ' << (drawer.IsNull()?0:drawer->GetRefCount()) << '\n';
}
static void FillDrawer(const Drawer& drawer) {
  drawer->SetColor(MeshVS_DA_InteriorColor,Quantity_Color(.125,.25,.5,Quantity_TOC_RGB));
  drawer->SetColor(MeshVS_DA_BackInteriorColor,Quantity_Color(.5,.25,.125,Quantity_TOC_RGB));
  drawer->SetColor(MeshVS_DA_EdgeColor,Quantity_Color(Quantity_NOC_RED));
  drawer->SetDouble(MeshVS_DA_EdgeWidth,2.25); drawer->SetInteger(MeshVS_DA_InteriorStyle,1);
  drawer->SetInteger(MeshVS_DA_EdgeType,2); drawer->SetInteger(MeshVS_DA_HatchStyle,3);
  drawer->SetInteger(MeshVS_DA_FrontMaterial,3); drawer->SetInteger(MeshVS_DA_BackMaterial,8);
}
static void SimpleDrawer(const Drawer& drawer) {
  drawer->SetColor(MeshVS_DA_BeamColor,Quantity_Color(Quantity_NOC_BLUE));
  drawer->SetDouble(MeshVS_DA_BeamWidth,1.75);drawer->SetInteger(MeshVS_DA_BeamType,1);
  drawer->SetColor(MeshVS_DA_MarkerColor,Quantity_Color(Quantity_NOC_GREEN));
  drawer->SetDouble(MeshVS_DA_MarkerScale,3.25);drawer->SetInteger(MeshVS_DA_MarkerType,7);
}
static void TextDrawer(const Drawer& drawer) {
  drawer->SetColor(MeshVS_DA_TextColor,Quantity_Color(Quantity_NOC_MAGENTA1));
  drawer->SetDouble(MeshVS_DA_TextExpansionFactor,-2.);drawer->SetDouble(MeshVS_DA_TextSpace,3.5);
  drawer->SetAsciiString(MeshVS_DA_TextFont,TCollection_AsciiString("Custom"));
  drawer->SetInteger(MeshVS_DA_TextStyle,1);drawer->SetInteger(MeshVS_DA_TextDisplayType,5);
  drawer->SetInteger(MeshVS_DA_TextFontAspect,2);
}
static void Populate(const Drawer& drawer) {
  FillDrawer(drawer); SimpleDrawer(drawer); TextDrawer(drawer);
}
static void Remove(const Drawer& drawer,int key) {
  drawer->RemoveInteger(key);drawer->RemoveDouble(key);drawer->RemoveColor(key);
  drawer->RemoveAsciiString(key);
}
static void Defaults() {
  for(int kind=0;kind<5;++kind) {
    Drawer drawer=new MeshVS_Drawer;Row("defaults",kind,0,drawer,true);
    if(kind!=0) Row("null",kind,0,Drawer(),true);
    Row("empty",kind,0,drawer,false);
  }
}
static void Missing() {
  const std::vector<std::vector<int>> keys={{7,8},{1,2,3,5,0,4,6},{11,10,9},{13,14,12},{15,18,19,17,20,21,23}};
  for(int kind=0;kind<5;++kind) for(int key:keys[kind]) {
    Drawer drawer=new MeshVS_Drawer;Populate(drawer);Remove(drawer,key);
    Row("missing",kind,key,drawer,false);
  }
}
static void RawEnums() {
  const int kinds[]={1,1,1,2,3,4,4,4};
  const int keys[]={0,4,6,9,12,20,21,23};
  const int lower[]={-4,-8,0,-8,-16,0,0,-4},upper[]={3,7,15,7,15,1,7,3};
  for(int group=0;group<8;++group) for(int value=lower[group];value<=upper[group];++value) {
    Drawer drawer=new MeshVS_Drawer;Populate(drawer);drawer->SetInteger(keys[group],value);
    Row("raw",kinds[group],group*100+value,drawer,false);
  }
}
static void Failures() {
  for(int kind=0;kind<5;++kind) {Drawer drawer=new MeshVS_Drawer;Populate(drawer);Row("custom",kind,0,drawer,false);}
  const double widths[]={0.,-1.,1e-300,std::numeric_limits<double>::quiet_NaN(),std::numeric_limits<double>::infinity()};
  for(int kind=1;kind<4;++kind) for(int index=0;index<5;++index) {
    Drawer drawer=new MeshVS_Drawer;Populate(drawer);
    drawer->SetDouble(kind==1?MeshVS_DA_EdgeWidth:kind==2?MeshVS_DA_BeamWidth:MeshVS_DA_MarkerScale,widths[index]);
    Row("width",kind,index,drawer,false);
  }
}
static void FontBytes() {
  const char bytes[][8]={{'A',0,'Z',0},{char(255),'X',0},{0}};
  const int lengths[]={3,2,0};
  for(int index=0;index<3;++index) {
    Drawer drawer=new MeshVS_Drawer;Populate(drawer);
    drawer->SetAsciiString(MeshVS_DA_TextFont,TCollection_AsciiString(bytes[index],lengths[index]));
    Row("fontbytes",4,index,drawer,false);
  }
}
static void EarlyExit() {
  const int keys[]={MeshVS_DA_InteriorStyle,MeshVS_DA_BeamType,MeshVS_DA_MarkerType};
  const int widths[]={MeshVS_DA_EdgeWidth,MeshVS_DA_BeamWidth,MeshVS_DA_MarkerScale};
  for(int kind=1;kind<4;++kind) {
    Drawer drawer=new MeshVS_Drawer;Populate(drawer);
    drawer->SetDouble(widths[kind-1],0.);drawer->RemoveInteger(keys[kind-1]);
    Row("early",kind,0,drawer,false);
  }
}
static void NormalRow(int index,int lower,const std::vector<double>& values) {
  NCollection_Array1<double> nodes(lower,lower+int(values.size())-1);
  for(int i=0;i<int(values.size());++i) nodes.SetValue(lower+i,values[i]);
  for(int average=0;average<2;++average) {
    gp_Vec result(7,8,9);std::cout << "normal " << index << ' ' << average;
    try {bool planar=average?MeshVS_Tool::GetAverageNormal(nodes,result):MeshVS_Tool::GetNormal(nodes,result);std::cout << ' ' << planar;}
    catch(const Standard_Failure& error) {std::cout << " failure " << error.ExceptionType();}
    Real(result.X());Real(result.Y());Real(result.Z());std::cout << '\n';
  }
}
static void Normals() {
  const std::vector<std::vector<double>> rows={{0,0,0,1,0,0,0,1,0},{0,0,0,0,1,0,1,0,0},
    {0,0,0,1,0,0,1,1,1,0,1,0},{0,0,0,1,0,0,2,0,0},{0,0,0,1,0,0},
    {0,0,0,1,0,0,0,1,0,0,-1,0},{0,0,0,1e-4,0,0,0,1e-4,0},
    {3,0,0,0,1,0,0,0,1,0},{0,0,0,1,0,0,0,1,0},{0,0,0,1,0,0,0,1,0,8,9},
    {4,0,0,0,1,0,0,0,1,0},{-1,0,0,0},{0,0,0,0,0,0,1,1,1}};
  for(int index=0;index<int(rows.size());++index) NormalRow(index,index==8?-3:index==7||index==10||index==11?0:1,rows[index]);
}
static void EnumDiagnostics() {
  for(int value=-8;value<=7;++value)
    std::cerr << "line " << value << ' ' << Graphic3d_Aspects::DefaultLinePatternForType(Aspect_TypeOfLine(value)) << '\n';
  for(int value=-4;value<=3;++value)
    std::cerr << "font " << value << ' ' << Font_FontMgr::FontAspectToString(Font_FontAspect(value)) << '\n';
}
int main() {
  Defaults();Missing();RawEnums();Failures();FontBytes();EarlyExit();Normals();EnumDiagnostics();
}
