// Actual pinned AIS_ColoredShape API produces map-only projection; no stand-in parent object.
#include <AIS_ColoredShape.hxx>
#include <BRep_Builder.hxx>
#include <TopoDS_Vertex.hxx>
#include <iostream>
using DrawerHandle = occ::handle<AIS_ColoredDrawer>;
struct LifecycleInputs
{
  TopoDS_Vertex key;
  TopoDS_Shape reversed;
  TopoDS_Vertex outside;
  occ::handle<AIS_ColoredShape> object;
  occ::handle<Prs3d_Drawer> parent;
  int baselineKeys;
  int baselineParent;
  LifecycleInputs()
  {
    BRep_Builder().MakeVertex(key, gp_Pnt(1, 2, 3), 1.e-7);
    reversed = key.Reversed();
    BRep_Builder().MakeVertex(outside, gp_Pnt(1, 2, 3), 1.e-7);
    object = new AIS_ColoredShape(key);
    parent = object->Attributes();
    baselineKeys = key.TShape()->GetRefCount();
    baselineParent = parent->GetRefCount();
  }
};
static int child_count(const DrawerHandle& child)
{
  // Null outputs do not pretend to own a native drawer.
  return child.IsNull() ? 0 : child->GetRefCount();
}
static void snapshot(const char* label, const LifecycleInputs& input, const DrawerHandle* children)
{
  std::cout << label << ' ' << input.object->CustomAspectsMap().Extent()
            << ' ' << input.key.TShape()->GetRefCount() - input.baselineKeys
            << ' ' << input.parent->GetRefCount() - input.baselineParent;
  for (int index = 0; index < 4; ++index) {
    std::cout << ' ' << child_count(children[index]);
  }
  std::cout << '\n';
}
static void initial_custom(LifecycleInputs& input, DrawerHandle* children)
{
  const auto& constMap = input.object->CustomAspectsMap();
  children[0] = input.object->CustomAspects(input.key);
  const auto& valueAlias = constMap.Find(input.key);
  const auto* valueAddress = &valueAlias;
  snapshot("created", input, children);
  DrawerHandle repeat = input.object->CustomAspects(input.reversed);
  std::cout << "reused " << (repeat == children[0]) << ' ' << constMap.Extent() << '\n';
  repeat.Nullify();
  children[0]->SetHidden(true);
  input.object->UnsetCustomAspects(input.key, false);
  std::cout << "replaced " << (&constMap.Find(input.key) == valueAddress) << ' '
            << (valueAlias != children[0]) << ' ' << valueAlias->IsHidden() << '\n';
  children[1] = input.object->CustomAspects(input.reversed);
  snapshot("replacement-counts", input, children);
  input.object->UnsetCustomAspects(input.key, true);
  snapshot("unregistered-unused-alias", input, children);
}
static void null_value_custom(LifecycleInputs& input, DrawerHandle* children)
{
  auto& map = input.object->ChangeCustomAspectsMap();
  map.Bind(input.reversed, DrawerHandle());
  const auto& keyAlias = map.Contained(input.key)->first.get();
  const auto& valueAlias = map.Find(input.key);
  const auto* valueAddress = &valueAlias;
  children[2] = input.object->CustomAspects(input.key);
  std::cout << "null-replaced " << keyAlias.Orientation() << ' ' << (valueAlias == children[2])
            << ' ' << (&map.Find(input.key) == valueAddress) << '\n';
  snapshot("null-replacement-counts", input, children);
  children[3] = input.object->CustomAspects(input.outside);
  std::cout << "outside " << !input.key.IsSame(input.outside) << ' '
            << input.outside.TShape()->GetRefCount() << '\n';
  snapshot("outside-counts", input, children);
}
static void release_custom(LifecycleInputs& input, DrawerHandle* children)
{
  const auto& retainedMap = input.object->CustomAspectsMap();
  input.object->ClearCustomAspects();
  std::cout << "map-reference " << retainedMap.IsEmpty() << ' '
            << (&retainedMap == &input.object->ChangeCustomAspectsMap()) << '\n';
  snapshot("clear-unused-alias", input, children);
  input.object->ClearCustomAspects();
  snapshot("clear-empty", input, children);
  for (int index = 0; index < 4; ++index) { children[index].Nullify(); }
  snapshot("released-outputs", input, children);
}
int main()
{
  LifecycleInputs input;
  DrawerHandle children[4];
  // Raw baseline exposes the real AIS_Shape retention that an isolated Rust map does not claim.
  std::cout << "native-baseline " << input.baselineKeys << ' ' << input.baselineParent << '\n';
  snapshot("initial", input, children);
  input.object->UnsetCustomAspects(input.outside, false);
  snapshot("missing-replace", input, children);
  input.object->UnsetCustomAspects(input.outside, true);
  snapshot("missing-unregister", input, children);
  initial_custom(input, children);
  null_value_custom(input, children);
  release_custom(input, children);
}
