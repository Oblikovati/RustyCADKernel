#include <BOPDS_DS.hxx>
#include <BOPDS_Iterator.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <NCollection_List.hxx>
#include <TopoDS_Shape.hxx>
#include <iostream>

int main() {
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(BRepPrimAPI_MakeBox(1.0, 1.0, 1.0).Shape());
  arguments.Append(BRepPrimAPI_MakeBox(gp_Pnt(0.5, 0.0, 0.0), 1.0, 1.0, 1.0).Shape());
  arguments.Append(BRepPrimAPI_MakeBox(gp_Pnt(1.0, 0.0, 0.0), 1.0, 1.0, 1.0).Shape());
  BOPDS_DS data;
  data.SetArguments(arguments);
  data.Init(1.0e-7);
  BOPDS_Iterator iterator;
  iterator.SetDS(&data);
  iterator.Prepare();
  iterator.Initialize(TopAbs_VERTEX, TopAbs_EDGE);
  std::cout << data.NbSourceShapes() << ' ' << data.NbRanges() << ' '
            << iterator.ExpectedLength() << '\n';
  while (iterator.More()) {
    int first, second;
    iterator.Value(first, second);
    std::cout << first << '-' << second << ',';
    iterator.Next();
  }
  std::cout << '\n';
}
