// Local Martensite oracle: real ViewCube buffers, sizing passes, group order and 3D text frames.
#include <AIS_ViewCube.hxx>
#include <AIS_InteractiveContext.hxx>
#include <V3d_Viewer.hxx>
#include <Prs3d_Presentation.hxx>
#include <Prs3d_Drawer.hxx>
#include <Prs3d_DatumAspect.hxx>
#include <Graphic3d_Buffer.hxx>
#include <Graphic3d_ArrayOfTriangles.hxx>
#include <Graphic3d_IndexBuffer.hxx>
#include <cstring>
#include <iomanip>
#include <iostream>
#include "tkservice_probe_null_driver.cxx"
struct CubeRecorder {int caseId=0;};
static void vector3(const gp_XYZ& value) {std::cout<<' '<<value.X()<<' '<<value.Y()<<' '<<value.Z();}
static void cubeArray(int id,Graphic3d_TypeOfPrimitiveArray kind,const occ::handle<Graphic3d_Buffer>& vertices,const occ::handle<Graphic3d_IndexBuffer>& indices) {
  int posIndex=-1,normIndex=-1;size_t posStride=0,normStride=0;
  const uint8_t* pos=vertices->AttributeData(Graphic3d_TOA_POS,posIndex,posStride);
  const uint8_t* normals=vertices->AttributeData(Graphic3d_TOA_NORM,normIndex,normStride);
  std::cout<<"array "<<id<<' '<<int(kind)<<' '<<vertices->NbElements<<' '<<(indices.IsNull()?0:indices->NbElements)<<' '<<(normals?6:3);
  for(int i=0;i<vertices->NbElements;++i) {
    float point[3];std::memcpy(point,pos+i*posStride,sizeof(point));for(float channel:point)std::cout<<' '<<channel;
    if(normals) {float normal[3];std::memcpy(normal,normals+i*normStride,sizeof(normal));for(float channel:normal)std::cout<<' '<<channel;}
  }
  if(!indices.IsNull()) for(int i=0;i<indices->NbElements;++i)std::cout<<' '<<indices->Index(i);
  std::cout<<'\n';
}
class CubeGroup:public SvGroup {
  CubeRecorder& recorder;
public:
  CubeGroup(const occ::handle<Graphic3d_Structure>& structure,CubeRecorder& record):SvGroup(structure),recorder(record){}
  void AddText(const occ::handle<Graphic3d_Text>& text,const bool) override {
    const auto& frame=text->Orientation();std::cout<<"text "<<recorder.caseId;vector3(frame.Location().XYZ());vector3(frame.Direction().XYZ());vector3(frame.XDirection().XYZ());
    std::cout<<' '<<text->Height()<<' '<<int(text->HorizontalAlignment())<<' '<<int(text->VerticalAlignment())<<' '<<text->HasOwnAnchorPoint()<<' '<<text->HasPlane()<<' '<<std::quoted(text->Text().ToCString())<<'\n';
  }
  void AddPrimitiveArray(const Graphic3d_TypeOfPrimitiveArray kind,const occ::handle<Graphic3d_IndexBuffer>& indices,const occ::handle<Graphic3d_Buffer>& vertices,const occ::handle<Graphic3d_BoundBuffer>&,const bool) override {cubeArray(recorder.caseId,kind,vertices,indices);}
};
class CubeStructure:public SvCStructure {
  CubeRecorder& recorder;
public:
  CubeStructure(const occ::handle<Graphic3d_StructureManager>& manager,CubeRecorder& record):SvCStructure(manager),recorder(record){}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& structure) override {
    occ::handle<Graphic3d_Group> group=new CubeGroup(structure,recorder);myGroups.Append(group);return group;
  }
};
class CubeDriver:public SvDriver {
  CubeRecorder& recorder;
public:
  CubeDriver(CubeRecorder& record):recorder(record){}
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& manager) override {return new CubeStructure(manager,recorder);}
};
class CubeProbe:public AIS_ViewCube {
public:
  using AIS_ViewCube::createBoxPartTriangles;using AIS_ViewCube::createRoundRectangleTriangles;
};
static occ::handle<AIS_InteractiveContext> attachContext(const occ::handle<CubeProbe>& cube) {
  occ::handle<AIS_InteractiveContext> context=new AIS_InteractiveContext(new V3d_Viewer(new SvDriver()));
  cube->SetContext(context);return context;
}
static void configure(const occ::handle<CubeProbe>& cube,int id) {
  if(id==1)cube->SetRoundRadius(0.2);
  if(id==2) {cube->SetSize(43,false);cube->SetRoundRadius(0.5);cube->SetBoxFacetExtension(7);cube->SetBoxEdgeGap(4);cube->SetBoxEdgeMinSize(3);cube->SetBoxCornerMinSize(4);cube->SetAxesPadding(2);cube->SetDrawAxes(false);}
  if(id==3)cube->SetBoxFacetExtension(-3);
  if(id==4) {cube->SetDrawEdges(false);cube->SetDrawVertices(false);}
  if(id==5)cube->SetYup(true,false);
  if(id==6) {cube->Attributes()->DatumAspect()->SetDrawDatumAxes(Prs3d_DatumAxes_XZAxes);cube->Attributes()->DatumAspect()->SetDrawLabels(true);cube->SetAxesLabels("u","v","w");cube->SetFontHeight(22);}
  if(id==7) {cube->SetRoundRadius(0.1);cube->SetBoxFacetExtension(0);cube->SetBoxEdgeMinSize(6);cube->SetBoxCornerMinSize(3);cube->SetDrawAxes(false);cube->SetBoxSideLabel(V3d_Yneg,"");}
  cube->Attributes()->SetFaceBoundaryDraw(id%3==1);
}
static void geometry(int id) {
  occ::handle<CubeProbe> cube=new CubeProbe();auto context=attachContext(cube);configure(cube,id);
  for(int orientation=0;orientation<26;++orientation) {
    int vertices=0,triangles=0;cube->createBoxPartTriangles({},vertices,triangles,V3d_TypeOfOrientation(orientation));
    std::cout<<"capacity "<<id<<' '<<orientation<<' '<<vertices<<' '<<triangles<<'\n';if(vertices==0)continue;
    auto array=occ::handle<Graphic3d_ArrayOfTriangles>(new Graphic3d_ArrayOfTriangles(vertices,triangles*3,Graphic3d_ArrayFlags_VertexNormal));
    vertices=triangles=0;cube->createBoxPartTriangles(array,vertices,triangles,V3d_TypeOfOrientation(orientation));
    cubeArray(id*26+orientation,Graphic3d_TOPA_TRIANGLES,array->Attributes(),array->Indices());
  }
}
static void scene(int id) {
  CubeRecorder recorder;recorder.caseId=id;occ::handle<CubeProbe> cube=new CubeProbe();auto context=attachContext(cube);configure(cube,id%8);
  occ::handle<Graphic3d_StructureManager> manager=new Graphic3d_StructureManager(new CubeDriver(recorder));
  occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(manager);
  cube->Compute({},prs,id<8?0:1);
  std::cout<<"groups "<<id<<' '<<prs->Groups().Length()<<' '<<prs->IsInfinite();for(const auto& group:prs->Groups())std::cout<<' '<<group->IsClosed();std::cout<<'\n';
}
static void rectangles() {
  for(int id=0;id<8;++id) {
    const gp_XY size(id%2?3:11,id%3?7:0);double radius=id%4-1;
    gp_Trsf transform;transform.SetRotation(gp_Ax1(gp_Pnt(2,-3,5),gp_Dir(1,2,3)),0.3);
    int vertices=0,triangles=0;CubeProbe::createRoundRectangleTriangles({},vertices,triangles,size,radius,transform);
    std::cout<<"rectangle "<<id<<' '<<size.X()<<' '<<size.Y()<<' '<<radius<<' '<<vertices<<' '<<triangles<<'\n';
    auto array=occ::handle<Graphic3d_ArrayOfTriangles>(new Graphic3d_ArrayOfTriangles(vertices,triangles*3,Graphic3d_ArrayFlags_VertexNormal));
    vertices=triangles=0;CubeProbe::createRoundRectangleTriangles(array,vertices,triangles,size,radius,transform);
    cubeArray(208+id,Graphic3d_TOPA_TRIANGLES,array->Attributes(),array->Indices());
  }
}
int main() {
  std::cout<<std::setprecision(17);
  for(int orientation=0;orientation<26;++orientation)std::cout<<"classify "<<orientation<<' '<<AIS_ViewCube::IsBoxSide(V3d_TypeOfOrientation(orientation))<<' '<<AIS_ViewCube::IsBoxEdge(V3d_TypeOfOrientation(orientation))<<' '<<AIS_ViewCube::IsBoxCorner(V3d_TypeOfOrientation(orientation))<<'\n';
  for(int id=0;id<8;++id)geometry(id);rectangles();for(int id=0;id<10;++id)scene(id);
}
