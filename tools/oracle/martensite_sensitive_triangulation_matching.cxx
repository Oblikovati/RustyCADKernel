// Martensite local triangulation oracle at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#define main martensite_primitive_matching_unused_main
#include "martensite_sensitive_primitive_matching.cxx"
#undef main
#include <Select3D_SensitiveTriangulation.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <Poly_Triangulation.hxx>
#include <NCollection_HArray1.hxx>
#include <Standard_OutOfRange.hxx>
#include <Bnd_Box.hxx>
static occ::handle<Poly_Triangulation> TriMesh(int kind) {
  if(kind==3||kind==4) {
    occ::handle<Poly_Triangulation> mesh=new Poly_Triangulation();
    if(kind==3) {Bnd_Box box;box.Add(gp_Pnt(-2,-1,-.5));box.Add(gp_Pnt(2,3,.5));mesh->SetCachedMinMax(box);}
    return mesh;
  }
  const bool grid=kind==0;const int nodes=grid?36:4,triangles=grid?50:(kind==2?4:2);
  occ::handle<Poly_Triangulation> mesh=new Poly_Triangulation(nodes,triangles,false);
  for(int i=1;i<=nodes;++i)mesh->SetNode(i,grid?gp_Pnt((i-1)%6-3,(i-1)/6-3,-((i-1)%4)*.25):gp_Pnt(i==2?3:(i==3?1:0),i>=3?2:0,i==4&&kind==2?1:0));
  for(int i=1;i<=triangles;++i) {
    const int cell=(i-1)/2,origin=(cell/5)*6+cell%5+1;
    mesh->SetTriangle(i,grid?(i%2?Poly_Triangle(origin,origin+1,origin+6):Poly_Triangle(origin+1,origin+7,origin+6))
      :(kind==2?(i==1?Poly_Triangle(1,2,3):(i==2?Poly_Triangle(1,4,2):(i==3?Poly_Triangle(2,4,3):Poly_Triangle(3,4,1)))):(i==1?Poly_Triangle(1,2,3):Poly_Triangle(1,3,4))));
  }
  if(kind==5) {Bnd_Box box;box.Add(gp_Pnt(-4,-3,-2));box.Add(gp_Pnt(5,6,2));mesh->SetCachedMinMax(box);}
  return mesh;
}
static TopLoc_Location TriLocation(int kind) {
  gp_Trsf transform;
  if(kind==1)transform.SetTranslation(gp_Vec(2,-1,.5));
  if(kind==2)transform.SetRotation(gp_Ax1(gp_Pnt(),gp_Dir(0,0,1)),.7);
  return kind==0?TopLoc_Location():TopLoc_Location(transform);
}
static occ::handle<NCollection_HArray1<int>> TriEdges(int policy,int length=8) {
  if(policy==0)return {};
  const int lower=policy==1?1:(policy==2?0:2);const int values[]={1,2,2,3,3,4,4,1};
  occ::handle<NCollection_HArray1<int>> edges=new NCollection_HArray1<int>(lower,lower+length-1);
  for(int i=0;i<length;++i)edges->SetValue(lower+i,values[i%8]);
  return edges;
}
static void PointColumns(const gp_Pnt& point) {
  // Keep picked/local/transformed positions distinguishable in the committed column stream.
  std::cout<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z();
}
static void TransformColumns(const gp_Trsf& transform) {
  // Full affine coefficients prove location behavior beyond a single transformed point.
  for(int row=1;row<=3;++row)for(int col=1;col<=4;++col)std::cout<<' '<<transform.Value(row,col);
}
static void TriGeometryColumns(const occ::handle<Select3D_SensitiveTriangulation>& probe) {
  std::cout<<' '<<probe->Size()<<' '<<probe->NbSubElements();PointColumns(probe->CenterOfGeometry());
  std::cout<<' '<<probe->HasInitLocation();TransformColumns(probe->GetInitLocation().Transformation());
  for(int row=1;row<=3;++row)for(int col=1;col<=4;++col)std::cout<<' '<<probe->InvInitLocation().Value(row,col);
  const auto box=probe->BoundingBox();std::cout<<' '<<box.IsValid();
  for(int axis=0;axis<3;++axis)std::cout<<' '<<box.CornerMin()[axis];
  for(int axis=0;axis<3;++axis)std::cout<<' '<<box.CornerMax()[axis];
  std::cout<<' '<<probe->SensitivityFactor()<<' '<<probe->ToBuildBVH()<<' '<<probe->OwnerId()->Priority();
}
static void TriDetectionColumns(const occ::handle<Select3D_SensitiveTriangulation>& probe) {
  Poly_Triangle triangle(0,0,0);gp_Pnt points[3]={gp_Pnt(),gp_Pnt(),gp_Pnt()};
  const bool found=probe->LastDetectedTriangle(triangle);const bool nodes=probe->LastDetectedTriangle(triangle,points);
  std::cout<<' '<<probe->LastDetectedTriangleIndex()<<' '<<found<<' '<<nodes;
  for(int i=1;i<=3;++i)std::cout<<' '<<triangle.Value(i);
  for(const auto& point:points)PointColumns(point);
}
static void TriMatchSnapshot(int scenario,int step,const occ::handle<Select3D_SensitiveTriangulation>& probe,int volumeKind) {
  auto volume=Volume(volumeKind,step==1);SelectBasics_PickResult pick;bool hit=false;int failure=0;
  try {hit=probe->Matches(volume,pick);}catch(const Standard_OutOfRange&) {failure=1;}catch(const Standard_Failure&) {failure=2;}
  std::cout<<scenario<<' '<<step<<' '<<failure<<' '<<hit<<' '<<pick.IsValid()<<' '<<(pick.IsValid()?pick.Depth():0)
    <<' '<<(pick.DistToGeomCenter()==RealLast()?0:pick.DistToGeomCenter())<<' '<<pick.HasPickedPoint();
  PointColumns(pick.HasPickedPoint()?pick.PickedPoint():gp_Pnt());
  TriGeometryColumns(probe);TriDetectionColumns(probe);std::cout<<'\n';
}
static void TriScenario(int ordinal) {
  const int volumes[]={0,1,2,3,4,6};const int volume=volumes[ordinal%6],location=ordinal/6%3;
  const int policy=ordinal/18%4,interior=ordinal/72%2,kind=ordinal/144;
  const auto mesh=TriMesh(kind);const auto edges=TriEdges(policy);occ::handle<SelectMgr_EntityOwner> owner=new SelectMgr_EntityOwner(13);
  occ::handle<Select3D_SensitiveTriangulation> original=policy==0
    ?new Select3D_SensitiveTriangulation(owner,mesh,TriLocation(location),interior!=0)
    :new Select3D_SensitiveTriangulation(owner,mesh,TriLocation(location),edges,gp_Pnt(7,8,9),interior!=0);
  original->SetSensitivityFactor(7);
  const auto connected=occ::down_cast<Select3D_SensitiveTriangulation>(original->GetConnected());
  for(int step=0;step<4;++step)TriMatchSnapshot(ordinal,step,step==3?connected:original,volume);
  if(!edges.IsNull())edges->SetValue(edges->Lower(),4);
  original->MarkDirty();connected->MarkDirty();
  TriMatchSnapshot(ordinal,4,original,volume);TriMatchSnapshot(ordinal,5,connected,volume);
}
static void TriOddControls() {
  for(int control=0;control<12;++control) {
    const int kind=control<6?1:3,interior=control/3%2,lengths[]={0,1,3};
    const int length=lengths[control%3],ordinal=864+control;int failure=0;
    try {Select3D_SensitiveTriangulation probe(new SelectMgr_EntityOwner(13),TriMesh(kind),{},TriEdges(1,length),gp_Pnt(7,8,9),interior!=0);}
    catch(const Standard_OutOfRange&) {failure=1;}catch(const Standard_Failure&) {failure=2;}
    std::cout<<ordinal<<" -1 "<<failure<<'\n';
  }
}
#ifndef MARTENSITE_TRIANGULATION_EMBEDDED
int main() {
  std::cout<<std::setprecision(17);
  for(int ordinal=0;ordinal<864;++ordinal)TriScenario(ordinal);
  TriOddControls();return 0;
}

#endif
