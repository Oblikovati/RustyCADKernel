#include <BOPDS_DS.hxx>
#include <BOPDS_FaceInfo.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <iostream>
void output(BOPDS_DS& ds, int stage)
{
  for (int i=0; i<ds.NbSourceShapes(); ++i) {
    if (ds.ShapeInfo(i).ShapeType()!=TopAbs_FACE) continue;
    const auto& info=ds.FaceInfo(i);
    std::cout<<stage<<' '<<i<<' '<<ds.HasFaceInfo(i)<<' '<<info.Index()<<' '
             <<info.PaveBlocksOn().Extent()<<' '<<info.VerticesOn().Extent()<<' '
             <<info.VerticesIn().Extent();
    for (int vertex:info.VerticesOn()) std::cout<<' '<<vertex;
    std::cout<<'\n';
  }
}
void initialize_faces(BOPDS_DS& ds)
{
  for(int i=0;i<ds.NbSourceShapes();++i)
    if(ds.ShapeInfo(i).ShapeType()==TopAbs_FACE) ds.ChangeFaceInfo(i);
}
void update_faces(BOPDS_DS& ds)
{
  for(int i=0;i<ds.NbSourceShapes();++i)
    if(ds.ShapeInfo(i).ShapeType()==TopAbs_FACE) ds.UpdateFaceInfoOn(i);
}
int main()
{
  BOPDS_DS ds; NCollection_List<TopoDS_Shape> args;
  args.Append(BRepPrimAPI_MakeBox(1.,1.,1.).Shape()); ds.SetArguments(args); ds.Init(1e-7);
  output(ds,0); update_faces(ds); output(ds,1);
  initialize_faces(ds); output(ds,2);
  for(int i=0;i<ds.NbSourceShapes();++i)
    if(ds.ShapeInfo(i).ShapeType()==TopAbs_EDGE) ds.ChangePaveBlocks(i);
  output(ds,3); update_faces(ds); output(ds,4);
  initialize_faces(ds); output(ds,5);
}
