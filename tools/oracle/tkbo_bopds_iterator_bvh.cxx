#include <BOPDS_DS.hxx>
#include <IntTools_Context.hxx>
#include <BOPDS_Iterator.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <NCollection_List.hxx>
#include <TopoDS_Shape.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <iostream>

void run(int layout,int obb) {
  std::cout << "case " << layout << " " << obb << "\n";
  NCollection_List<TopoDS_Shape> arguments;
  double origins[2][3]={{1.,.5,0.},{-1.,0.,.5}};
  for(double x:origins[layout])arguments.Append(BRepPrimAPI_MakeBox(gp_Pnt(x,0,0),1.,1.,1.).Shape());
  BOPDS_DS data;
  data.SetArguments(arguments);
  data.Init(1.0e-7);
  BOPDS_Iterator iterator;
  iterator.SetDS(&data);
  occ::handle<IntTools_Context> context=new IntTools_Context;
  iterator.Prepare(context,obb,1e-7);

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

int main(){for(int layout=0;layout<2;layout++)for(int obb=0;obb<2;obb++)run(layout,obb);}
