#include <BOPDS_DS.hxx>
#include <BOPDS_Iterator.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <NCollection_List.hxx>
#include <TopoDS_Shape.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <iostream>

int main() {
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(BRepPrimAPI_MakeBox(1.0, 1.0, 1.0).Shape());
  arguments.Append(BRepPrimAPI_MakeBox(gp_Pnt(0.5, 0.0, 0.0), 1.0, 1.0, 1.0).Shape());
  BOPDS_DS data;
  data.SetArguments(arguments);
  data.Init(1.0e-7);
  BOPDS_Iterator iterator;
  iterator.SetDS(&data);
  iterator.Prepare();

  const TopAbs_ShapeEnum types[] = {
    TopAbs_VERTEX, TopAbs_EDGE, TopAbs_FACE, TopAbs_SOLID,
    TopAbs_SHELL, TopAbs_WIRE, TopAbs_COMPOUND
  };
  for (TopAbs_ShapeEnum first : types) {
    for (TopAbs_ShapeEnum second : types) {
      iterator.Initialize(first, second);
      std::cout << static_cast<int>(first) << ',' << static_cast<int>(second) << ':';
      while (iterator.More()) {
        int i, j;
        iterator.Value(i, j);
        std::cout << i << '-' << j << ',';
        iterator.Next();
      }
      std::cout << '\n';
    }
  }
  iterator.Initialize(TopAbs_VERTEX, TopAbs_EDGE);
  int first = -1, second = -1;
  iterator.Value(first, second);
  iterator.Initialize(TopAbs_VERTEX, TopAbs_COMPOUND);
  int staleFirst = -1, staleSecond = -1;
  iterator.Value(staleFirst, staleSecond);
  std::cout << "invalid " << iterator.ExpectedLength() << ' ' << iterator.More() << ' '
            << staleFirst << '-' << staleSecond << '\n';
}
