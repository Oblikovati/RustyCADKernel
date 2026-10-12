// Martensite actual pinned NCollection shape/drawer specialization; no invented AIS typedef.
#include <AIS_ColoredDrawer.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Vertex.hxx>
#include <TopTools_ShapeMapHasher.hxx>
#include <NCollection_DataMap.hxx>
#include <gp_Trsf.hxx>
#include <iostream>
#include <Standard_Failure.hxx>
#include <vector>
using DrawerHandle = occ::handle<AIS_ColoredDrawer>;
using ShapeDrawerMap = NCollection_DataMap<TopoDS_Shape, DrawerHandle, TopTools_ShapeMapHasher>;
struct MapInputs
{
  TopoDS_Vertex key;
  TopoDS_Shape reversed;
  TopoDS_Shape located;
  occ::handle<Prs3d_Drawer> parent;
  DrawerHandle first;
  DrawerHandle second;
  MapInputs()
  {
    BRep_Builder().MakeVertex(key, gp_Pnt(1, 2, 3), 1.e-7);
    reversed = key.Reversed();
    gp_Trsf transform;
    transform.SetTranslation(gp_Vec(7, -3, 2));
    located = key.Located(TopLoc_Location(transform));
    parent = new Prs3d_Drawer();
    first = new AIS_ColoredDrawer(parent);
    second = new AIS_ColoredDrawer(parent);
  }
};
static void counts(const char* label, const ShapeDrawerMap& map, const MapInputs& input)
{
  std::cout << label << ' ' << map.Extent() << ' ' << input.key.TShape()->GetRefCount()
            << ' ' << input.first->GetRefCount() << ' ' << input.second->GetRefCount()
            << ' ' << input.parent->GetRefCount() << '\n';
}
static std::vector<TopoDS_Shape> fill_map(ShapeDrawerMap& map, int count = 128)
{
  std::vector<TopoDS_Shape> keys;
  for (int index = 0; index < count; ++index) {
    TopoDS_Vertex vertex;
    BRep_Builder().MakeVertex(vertex, gp_Pnt(index, 8, 9), 1.e-7);
    keys.push_back(vertex);
    map.Bind(keys.back(), DrawerHandle());
  }
  return keys;
}
static void key_identity(ShapeDrawerMap& map, MapInputs& input)
{
  TopTools_ShapeMapHasher hash;
  std::cout << "identity " << input.key.IsSame(input.reversed) << ' ' << input.key.IsEqual(input.reversed)
            << ' ' << (hash(input.key) == hash(input.reversed)) << ' ' << input.key.IsSame(input.located) << '\n';
  std::cout << "first-bind " << map.Bind(input.key, input.first) << '\n';
  const auto pair = map.Contained(input.key).value();
  const TopoDS_Shape& storedKey = pair.first;
  const DrawerHandle& alias = pair.second;
  std::cout << "replace-bind " << map.Bind(input.reversed, input.second) << ' ' << storedKey.Orientation()
            << ' ' << (alias == input.second) << ' ' << (&alias == &map.Find(input.key)) << '\n';
  counts("replace-counts", map, input);
  map.ChangeFind(input.key) = input.first;
  counts("assign-counts", map, input);
  input.key.Closed(true);
  std::cout << "shared-key " << storedKey.Closed() << '\n';
}
static void copied_key_and_failure(ShapeDrawerMap& map, const MapInputs& input)
{
  const auto pair = map.Contained(input.key).value();
  TopoDS_Shape copy = pair.first;
  std::cout << "explicit-copy " << copy.IsSame(input.key) << ' ' << copy.Orientation()
            << ' ' << input.key.TShape()->GetRefCount() << ' ' << pair.second.get()->GetRefCount() << '\n';
}
static void missing_key(const ShapeDrawerMap& map, const MapInputs& input)
{
  try { map.Find(input.key); }
  catch (const Standard_Failure& failure) {
    std::cout << "missing-key " << failure.ExceptionType() << '\n';
  }
}
static void removal_and_release(ShapeDrawerMap& map, const MapInputs& input, const std::vector<TopoDS_Shape>& keys)
{
  const TopoDS_Shape& lastKeyAlias = map.Contained(keys.back())->first;
  const auto* lastAddress = &lastKeyAlias;
  map.UnBind(input.key);
  std::cout << "survivor " << lastKeyAlias.IsSame(keys.back()) << ' '
            << (&map.Contained(keys.back())->first.get() == lastAddress) << '\n';
  counts("unbind-unused-alias", map, input);
  missing_key(map, input);
  map.Clear(false);
  counts("clear-unused-alias", map, input);
  std::cout << "clear-last-key " << keys.back().TShape()->GetRefCount() << '\n';
}
static void growth_and_release(ShapeDrawerMap& map, const MapInputs& input)
{
  const auto pair = map.Contained(input.key).value();
  const TopoDS_Shape& keyAlias = pair.first;
  const DrawerHandle& valueAlias = pair.second;
  DrawerHandle owningValue = valueAlias;
  map.Bind(input.reversed, input.second);
  const auto* keyAddress = &keyAlias;
  const auto* valueAddress = &valueAlias;
  map.ReSize(1000);
  const auto keys = fill_map(map);
  std::cout << "growth " << map.NbBuckets() << ' ' << (&map.Contained(input.key)->first.get() == keyAddress)
            << ' ' << (&map.Find(input.key) == valueAddress) << ' ' << (valueAlias == input.second) << '\n';
  counts("growth-counts", map, input);
  removal_and_release(map, input, keys);
}
static occ::handle<TopLoc_Datum3D> location_ownership(ShapeDrawerMap& map, MapInputs& input)
{
  TopLoc_Location original = input.located.Location();
  occ::handle<TopLoc_Datum3D> datum = original.FirstDatum();
  const auto& storedKey = map.Contained(input.located)->first.get();
  gp_Trsf transform;
  transform.SetTranslation(gp_Vec(7, -3, 2));
  std::cout << "location-identity " << storedKey.IsSame(input.key.Located(original)) << ' '
            << storedKey.IsSame(input.key.Located(TopLoc_Location(transform))) << '\n';
  input.located.Location(TopLoc_Location());
  std::cout << "stored-location " << storedKey.IsSame(input.key.Located(original)) << ' '
            << storedKey.IsSame(input.located) << '\n';
  original.Clear();
  std::cout << "location-datum-owned " << datum->GetRefCount() << '\n';
  return datum;
}
static void location_and_null_keys(ShapeDrawerMap& map, MapInputs& input)
{
  map.Bind(input.key, input.first);
  map.Bind(input.located, input.second);
  const auto& locatedKey = map.Contained(input.located)->first.get();
  std::cout << "location " << map.Extent() << ' ' << locatedKey.IsSame(input.located)
            << ' ' << locatedKey.IsSame(input.key) << '\n';
  const auto datum = location_ownership(map, input);
  counts("location-counts", map, input);
  TopoDS_Shape nullKey;
  map.Bind(nullKey, input.first);
  std::cout << "null-key " << map.IsBound(nullKey.Reversed()) << ' '
            << map.Contained(nullKey.Reversed())->first.get().Orientation() << '\n';
  map.Clear(true);
  counts("clear-release", map, input);
  std::cout << "location-datum-released " << datum->GetRefCount() << '\n';
}
static void destruction_with_unused_alias(const MapInputs& input)
{
  const DrawerHandle* unusedAlias = nullptr;
  {
    ShapeDrawerMap local;
    local.Bind(input.key, input.first);
    unusedAlias = &local.Find(input.key);
  }
  // A dangling pointer may remain unused; it must not postpone destruction of owned fields.
  std::cout << "destruction-unused-alias " << (unusedAlias != nullptr) << ' '
            << input.key.TShape()->GetRefCount() << ' ' << input.first->GetRefCount() << '\n';
}
static void replacement_rehash()
{
  ShapeDrawerMap map;
  const auto keys = fill_map(map, 102);
  const auto& alias = map.Find(keys[0]);
  const auto* address = &alias;
  std::cout << "hit-before " << map.Extent() << ' ' << map.NbBuckets() << '\n';
  const bool inserted = map.Bind(keys[0], DrawerHandle());
  std::cout << "hit-after " << inserted << ' ' << map.Extent() << ' ' << map.NbBuckets()
            << ' ' << (&map.Find(keys[0]) == address) << '\n';
}
int main()
{
  MapInputs input;
  ShapeDrawerMap map;
  counts("initial", map, input);
  key_identity(map, input);
  copied_key_and_failure(map, input);
  growth_and_release(map, input);
  location_and_null_keys(map, input);
  destruction_with_unused_alias(input);
  replacement_rehash();
}
