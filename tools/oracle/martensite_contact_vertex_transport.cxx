// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; observations do not certify paired roots.
#include "martensite_partition_copy_fixture.hxx"
#include "martensite_contact_sources.hxx"
#include <BOPAlgo_BuilderFace.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_PaveBlock.hxx>
#include <BRep_Tool.hxx>
#include <iostream>
#include <iomanip>
static void EmitPoint(const char* role,int mode,int source,int copiedIndex,
                      const TopoDS_Shape& vertex)
{
  const auto point=BRep_Tool::Pnt(TopoDS::Vertex(vertex));
  std::cout<<role<<' '<<mode<<' '<<source<<' '<<copiedIndex<<' '
           <<vertex.Orientation()<<' '<<point.X()<<' '<<point.Y()<<' '<<point.Z()<<'\n';
}
static void EmitVertexCopies(int mode,BOPDS_DS& ds,BRepBuilderAPI_Copy& copier,
                             const PartitionShapeIndex& copiedIndex,const PartitionShapeIndex& outputIndex)
{
  for(int source=0;source<ds.NbShapes();++source) {
    const auto& vertex=ds.Shape(source);
    if(vertex.ShapeType()!=TopAbs_VERTEX) continue;
    EmitPoint("native",mode,source,-1,vertex);
    if(!outputIndex.Contains(vertex)) { std::cout<<"unrepresented "<<mode<<' '<<source<<'\n';continue; }
    for(const auto& copied:copier.Modified(vertex))
      EmitPoint("copied",mode,source,copiedIndex.FindIndex(copied),copied);
  }
}
static void EmitDirectedPoints(int mode,const TopoDS_Shape& copied,
                               const PartitionShapeIndex& copiedIndex)
{
  NCollection_List<TopoDS_Shape> inputs;
  const auto support=mode==17
    ?SupportAndInputsOnFace(copied,inputs,BRepBuilderAPI_MakeFace(gp_Pln(gp_Ax3(gp_Pnt(2,3,5),gp::DZ(),gp::DX()))).Face())
    :SupportAndInputs(copied,inputs);
  BOPAlgo_BuilderFace builder;builder.SetFace(support);builder.SetShapes(inputs);builder.Perform();
  int area=0;
  for(const auto& face:builder.Areas()) {
    std::vector<TopoDS_Shape> members;CollectDirectedMembers(face,members);
    for(const auto& member:members) if(member.ShapeType()==TopAbs_VERTEX)
      EmitPoint("directed",mode,area,copiedIndex.FindIndex(member),member);
    ++area;
  }
}
static void EmitContactBlocks(int mode,BOPDS_DS& ds)
{
  for(int source=0;source<ds.NbSourceShapes();++source) {
    if(ds.ShapeInfo(source).ShapeType()!=TopAbs_EDGE) continue;
    for(const auto& own:ds.PaveBlocks(source)) {
      const auto real=ds.RealPaveBlock(own);int a,b,c,d;double u,v,s,t;
      own->Indices(a,b);real->Indices(c,d);own->Range(u,v);real->Range(s,t);
      std::cout<<"block "<<mode<<' '<<source<<' '<<own->OriginalEdge()<<' '<<own->Edge()<<' '
               <<real->OriginalEdge()<<' '<<real->Edge()<<' '<<u<<' '<<v<<' '<<s<<' '<<t<<' '
               <<a<<' '<<b<<' '<<c<<' '<<d<<'\n';
    }
  }
}
static void CaptureContactVertices(int mode)
{
  const auto sources=IsolatedInputs(ContactTransportSources(mode));
  BOPAlgo_PaveFiller filler;filler.SetArguments(sources);filler.SetNonDestructive(true);filler.Perform();
  PartitionBuilder builder;builder.SetArguments(sources);builder.SetNonDestructive(true);builder.PerformWithFiller(filler);
  if(filler.HasErrors()||builder.HasErrors()) throw Standard_Failure("contact observations HasErrors=true; expected completed mapped arrangement");
  BRepBuilderAPI_Copy copier(builder.Shape(),true,true);PartitionShapeIndex copiedIndex;
  TopExp::MapShapes(copier.Shape(),copiedIndex);auto& ds=*filler.PDS();
  PartitionShapeIndex outputIndex;TopExp::MapShapes(builder.Shape(),outputIndex);
  EmitContactBlocks(mode,ds);
  EmitVertexCopies(mode,ds,copier,copiedIndex,outputIndex);
  EmitDirectedPoints(mode,copier.Shape(),copiedIndex);
}
int main()
{
  std::cout<<std::setprecision(17)<<std::unitbuf;
  for(int mode=0;mode<18;++mode) CaptureContactVertices(mode);
  return 0;
}
