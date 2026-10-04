#include <BOPAlgo_Builder.hxx>
#include <BOPDS_DS.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRep_Tool.hxx>
#include <TopoDS.hxx>
#include <iostream>

using ImageMap = NCollection_DataMap<TopoDS_Shape,
                                     NCollection_List<TopoDS_Shape>,
                                     TopTools_ShapeMapHasher>;

class FillVertexImagesProbe : public BOPAlgo_Builder {
public:
  void Run(BOPDS_DS& ds) { myDS = &ds; FillImagesVertices(Message_ProgressRange()); }
};

int main() {
  BOPDS_DS ds;
  TopoDS_Vertex source = BRepBuilderAPI_MakeVertex(gp_Pnt(0, 0, 0));
  TopoDS_Vertex middle = BRepBuilderAPI_MakeVertex(gp_Pnt(1, 0, 0));
  TopoDS_Vertex final = BRepBuilderAPI_MakeVertex(gp_Pnt(2, 0, 0));
  const int iSource = ds.Append(source);
  const int iMiddle = ds.Append(middle);
  const int iFinal = ds.Append(final);
  ds.AddShapeSD(iSource, iMiddle);
  ds.AddShapeSD(iMiddle, iFinal);

  FillVertexImagesProbe builder;
  builder.Run(ds);
  std::cout << "images=" << builder.Images().Extent()
            << " origins=" << builder.Origins().Extent()
            << " same_domain=" << builder.ShapesSD().Extent() << '\n';
  for (ImageMap::Iterator it(builder.Images()); it.More(); it.Next()) {
    std::cout << "image " << BRep_Tool::Pnt(TopoDS::Vertex(it.Key())).X() << " ->";
    for (NCollection_List<TopoDS_Shape>::Iterator value(it.Value()); value.More(); value.Next()) {
      std::cout << ' ' << BRep_Tool::Pnt(TopoDS::Vertex(value.Value())).X();
    }
    std::cout << '\n';
  }
  for (ImageMap::Iterator it(builder.Origins()); it.More(); it.Next()) {
    std::cout << "origin " << BRep_Tool::Pnt(TopoDS::Vertex(it.Key())).X() << " <-";
    for (NCollection_List<TopoDS_Shape>::Iterator value(it.Value()); value.More(); value.Next()) {
      std::cout << ' ' << BRep_Tool::Pnt(TopoDS::Vertex(value.Value())).X();
    }
    std::cout << '\n';
  }
}
