// Martensite same-map traversal proof at pinned OCCT 3d097a0; real geometric edge/wire.
#include <AIS_ColoredShape.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Compound.hxx>
#include <TopoDS_Wire.hxx>
#include <TopoDS_Edge.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <iostream>
#include <vector>
class ColoredAliasProbe : public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  using AIS_ColoredShape::bindSubShapes;
  using AIS_ColoredShape::fillSubshapeDrawerMap;
  using AIS_ColoredShape::isShapeEntirelyVisible;
};
struct ColoredAliasInputs
{
  TopoDS_Wire wire;
  TopoDS_Shape edge;
  TopoDS_Compound base;
  std::vector<TopoDS_Shape> rejected;
  ColoredAliasInputs()
  {
    BRep_Builder builder;
    select_wire();
    select_edge();
    builder.Add(wire, edge);
    builder.MakeCompound(base);
    builder.Add(base, wire);
  }
  void select_wire()
  {
    for (int attempt = 0; attempt < 32; ++attempt) {
      BRep_Builder().MakeWire(wire);
      if (TopTools_ShapeMapHasher()(wire) % 101 < 76) return;
      rejected.push_back(wire);
    }
    throw Standard_Failure("wire allocation attempts=32; expected native bucket below 76");
  }
  void select_edge()
  {
    const size_t wireBucket = TopTools_ShapeMapHasher()(wire) % 101;
    for (int attempt = 0; attempt < 4096; ++attempt) {
      edge = BRepBuilderAPI_MakeEdge(gp_Pnt(attempt,0,0),gp_Pnt(attempt+1,0,0)).Edge();
      if (TopTools_ShapeMapHasher()(edge) % 101 > wireBucket) return;
      rejected.push_back(edge);
    }
    throw Standard_Failure("edge allocation attempts=4096; expected a later native bucket");
  }
};
template<class Map>
static void capture_borrowed_alias(ColoredAliasProbe& object, Map& map,
                                   ColoredAliasInputs& input,
                                   const occ::handle<AIS_ColoredDrawer>& drawer)
{
  typename Map::Iterator iterator(map);
  const auto* keyAddress = &iterator.Key();
  const auto* valueAddress = &iterator.Value();
  std::cout << "borrowed " << input.wire.TShape()->GetRefCount() << ' ' << drawer->GetRefCount() << '\n';
  object.bindSubShapes(map, iterator.Key(), iterator.Value());
  std::cout << "inserted " << input.wire.TShape()->GetRefCount() << ' ' << drawer->GetRefCount()
    << ' ' << (&iterator.Key() == keyAddress) << ' ' << (&iterator.Value() == valueAddress) << '\n';
  iterator.Next();
  std::cout << "visited-new " << iterator.More() << ' ' << iterator.Key().IsSame(input.edge) << '\n';
  iterator.Next();
  std::cout << "end " << iterator.More() << '\n';
}
int main()
{
  ColoredAliasInputs input;
  occ::handle<ColoredAliasProbe> object = new ColoredAliasProbe(input.base);
  auto drawer = object->CustomAspects(input.wire);
  auto& map = object->ChangeCustomAspectsMap();
  capture_borrowed_alias(*object, map, input, drawer);
  map.Clear(false); map.Bind(input.wire,drawer);
  object->fillSubshapeDrawerMap(map);
  std::cout << "same-output " << map.Extent() << ' ' << (map.Find(input.edge) == drawer) << '\n';
  return 0;
}
