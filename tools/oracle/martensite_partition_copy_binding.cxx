// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871; actual mapped images/copies/areas.
#include "martensite_partition_copy_fixture.hxx"
#include <BOPAlgo_BuilderFace.hxx>
#include <BOPDS_DS.hxx>
#include <TopoDS_Compound.hxx>
#include <gp.hxx>
#include <gp_Pln.hxx>
#include <BRepBuilderAPI_Copy.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepLib.hxx>
#include <BRepTools_History.hxx>
#include <BRepTools_WireExplorer.hxx>
#include <BRep_Builder.hxx>
#include <TopExp.hxx>
#include <NCollection_IndexedMap.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <TopoDS.hxx>
#include <iostream>
#include <iomanip>
#include <vector>

static void EmitCopy(int source,BOPDS_DS& ds,const TopoDS_Shape& original,
                     BRepBuilderAPI_Copy& copier,PartitionShapeIndex& copiedIndex)
{
  for(const auto& copied:copier.Modified(original)) {
    std::cout<<"copy "<<source<<' '<<ds.Index(original)<<' '<<original.Orientation()<<' '
             <<copied.Orientation()<<' '<<copiedIndex.FindIndex(copied)<<' '
             <<original.IsSame(copied)<<' '<<BRepTools_History::IsSupportedType(original)<<'\n';
  }
}
static void EmitSourceCopies(BOPDS_DS& ds,const PartitionBuilder& builder,
                             BRepBuilderAPI_Copy& copier,PartitionShapeIndex& copiedIndex)
{
  for(int source=0;source<ds.NbSourceShapes();++source) {
    const auto& original=ds.Shape(source);
    if(original.ShapeType()!=TopAbs_EDGE) continue;
    const auto* images=builder.Images().Seek(original);
    if(!images) {EmitCopy(source,ds,original,copier,copiedIndex);continue;}
    for(const auto& image:*images) EmitCopy(source,ds,image,copier,copiedIndex);
  }
}

static void EmitWireTraversal(int area,const TopoDS_Shape& face)
{
  int wire=0;
  for(TopoDS_Iterator child(face);child.More();child.Next()) {
    if(child.Value().ShapeType()!=TopAbs_WIRE) continue;
    int direct=0,visited=0;
    for(TopoDS_Iterator edge(child.Value());edge.More();edge.Next()) ++direct;
    BRepTools_WireExplorer explorer(TopoDS::Wire(child.Value()),TopoDS::Face(face));
    for(;explorer.More();explorer.Next()) ++visited;
    std::cout<<"wire "<<area<<' '<<wire++<<' '<<direct<<' '<<visited<<'\n';
  }
}

static void EmitAreaMembers(int area,const TopoDS_Shape& face,const PartitionShapeIndex& copiedIndex)
{
  std::vector<TopoDS_Shape> members;CollectDirectedMembers(face,members);
  for(const auto& member:members) {
    std::cout<<"member "<<area<<' '<<member.ShapeType()<<' '<<member.Orientation()<<' '
             <<copiedIndex.FindIndex(member)<<'\n';
  }
  EmitWireTraversal(area,face);
}
static void EmitAreas(const TopoDS_Shape& copied,const PartitionShapeIndex& copiedIndex)
{
  NCollection_List<TopoDS_Shape> inputs;const auto support=SupportAndInputs(copied,inputs);
  BOPAlgo_BuilderFace builder;builder.SetFace(support);builder.SetShapes(inputs);builder.Perform();
  std::cout<<"areas "<<builder.Areas().Size()<<' '<<builder.HasErrors()<<'\n';
  int area=0;
  for(const auto& face:builder.Areas()) {
    EmitAreaMembers(area,face,copiedIndex);
    ++area;
  }
}
static void Capture(int mode)
{
  const auto originals=PartitionBindingSources(mode);const auto sources=IsolatedInputs(originals);
  BOPAlgo_PaveFiller filler;filler.SetArguments(sources);filler.SetNonDestructive(true);filler.Perform();
  PartitionBuilder builder;builder.SetArguments(sources);builder.SetNonDestructive(true);
  builder.PerformWithFiller(filler);
  if(filler.HasErrors()||builder.HasErrors()) throw Standard_Failure("mapped arrangement HasErrors=true; expected owned observations");
  BRepBuilderAPI_Copy copier(builder.Shape(),true,true);PartitionShapeIndex copiedIndex;
  TopExp::MapShapes(copier.Shape(),copiedIndex);std::cout<<"mode "<<mode<<'\n';
  EmitSourceCopies(*filler.PDS(),builder,copier,copiedIndex);EmitAreas(copier.Shape(),copiedIndex);
}
int main()
{
  std::cout<<std::setprecision(17);
  for(int mode=0;mode<7;++mode) Capture(mode);
  return 0;
}
