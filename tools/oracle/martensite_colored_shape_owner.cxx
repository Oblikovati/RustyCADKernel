// Martensite owned ColoredShape CPU lifecycle oracle, pinned OCCT 3d097a0.
#include <AIS_ColoredShape.hxx>
#include <BRep_Builder.hxx>
#include <BRep_TVertex.hxx>
#include <TopoDS_Vertex.hxx>
#include <PrsMgr_Presentation.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <Prs3d_LineAspect.hxx>
#include <iostream>
#include "tkservice_probe_null_driver.cxx"
class ColoredOwnerProbe : public AIS_ColoredShape
{
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  void defaults() const
  {
    std::cout << "defaults " << myUVOrigin.X() << ' ' << myUVOrigin.Y()
      << ' ' << myUVRepeat.X() << ' ' << myUVRepeat.Y() << ' ' << myUVScale.X()
      << ' ' << myUVScale.Y() << ' ' << myInitAng << ' ' << myCompBB
      << ' ' << myBB.IsVoid() << ' ' << int(TypeOfPresentation3d()) << '\n';
  }
};
class ColoredOwnerPresentation : public PrsMgr_Presentation
{
public:
  ColoredOwnerPresentation(const occ::handle<PrsMgr_PresentationManager>& manager,
    const occ::handle<PrsMgr_PresentableObject>& object)
    : PrsMgr_Presentation(manager, object, 0)
  {
    // A real allocated presentation provides native invalidation state without GPU work.
  }
};
class ColoredReleaseDrawer : public AIS_ColoredDrawer
{
  const AIS_ColoredShape& owner;
  const PrsMgr_Presentation& presentation;
  const char* label;
public:
  ColoredReleaseDrawer(const AIS_ColoredShape& object,
    const PrsMgr_Presentation& prs, const char* name)
    : AIS_ColoredDrawer(object.Attributes()), owner(object), presentation(prs), label(name)
  {
    // Raw borrowed observers cannot extend owner/presentation ownership or create a cycle.
  }
  ~ColoredReleaseDrawer() override
  {
    std::cout << label << ' ' << presentation.MustBeUpdated()
      << ' ' << owner.CustomAspectsMap().Extent() << ' '
      << owner.Shape().TShape()->GetRefCount() << ' ' << owner.Attributes()->GetRefCount() << '\n';
  }
};
class ColoredOwnerDropDrawer : public AIS_ColoredDrawer
{
  TopoDS_Shape retainedKey;
public:
  ColoredOwnerDropDrawer(const TopoDS_Shape& key, const occ::handle<Prs3d_Drawer>& link)
    : AIS_ColoredDrawer(link), retainedKey(key)
  {
    // The actual child owns one topology copy; parent ownership is observed separately.
  }
  ~ColoredOwnerDropDrawer() override
  {
    std::cout << "owner-release " << retainedKey.TShape()->GetRefCount()
      << ' ' << Link()->GetRefCount() << '\n';
  }
};
static void owner_destruction()
{
  TopoDS_Vertex key;
  BRep_Builder().MakeVertex(key, gp_Pnt(1,2,3), 1.e-7);
  occ::handle<AIS_ColoredShape> object = new AIS_ColoredShape(key);
  object->ChangeCustomAspectsMap().Bind(key, new ColoredOwnerDropDrawer(key, object->Attributes()));
  object.Nullify();
}
class ColoredParentDropVertex : public BRep_TVertex
{
  occ::handle<Prs3d_Drawer> retainedDrawer;
public:
  ColoredParentDropVertex(const occ::handle<Prs3d_Drawer>& drawer)
    : retainedDrawer(drawer)
  {
    // The actual vertex owns one reference; its destructor observes the inherited owner reference.
  }
  ~ColoredParentDropVertex() override
  {
    std::cout << "parent-release " << retainedDrawer->GetRefCount() << '\n';
  }
};
static void parent_destruction()
{
  occ::handle<AIS_ColoredShape> object = new AIS_ColoredShape(TopoDS_Shape());
  occ::handle<TopoDS_TShape> topology = new ColoredParentDropVertex(object->Attributes());
  TopoDS_Shape key;
  key.TShape(topology);
  object->SetShape(key);
  topology.Nullify(); key.Nullify();
  object.Nullify();
}
static void owner_header(const occ::handle<ColoredOwnerProbe>& object, const TopoDS_Shape& key)
{
  object->defaults();
  const auto& drawer = object->Attributes();
  const auto& line = drawer->LineAspect();
  std::cout << "parent " << key.TShape()->GetRefCount() << ' '
    << (object->Shape().IsEqual(key)) << ' ' << object->Signature() << ' ' << int(object->Type()) << '\n';
  std::cout << "aspects " << (line == drawer->FreeBoundaryAspect()) << ' '
    << (line == drawer->UnFreeBoundaryAspect()) << ' ' << (line == drawer->SeenLineAspect())
    << ' ' << (line == drawer->FaceBoundaryAspect()) << '\n';
}
static void owner_runtime(const occ::handle<ColoredOwnerProbe>& object)
{
  const auto& type = object->DynamicType();
  std::cout << "runtime-base " << AIS_ColoredShape::get_type_name() << ' '
    << type->Parent()->Name() << ' ' << type->Parent()->Parent()->Name() << ' '
    << (type == AIS_ColoredShape::get_type_descriptor()) << '\n';
}
static void owner_modes(const occ::handle<ColoredOwnerProbe>& object)
{
  std::cout << "display-modes";
  for (int mode = -1; mode <= 3; ++mode) std::cout << ' ' << object->AcceptDisplayMode(mode);
  std::cout << '\n';
}
static void drawer_reference(const occ::handle<ColoredOwnerProbe>& object)
{
  const auto& drawerAlias = object->Attributes();
  const auto* drawerAddress = &drawerAlias;
  object->SetAttributes(new Prs3d_Drawer);
  std::cout << "drawer-reference " << (&object->Attributes() == drawerAddress) << ' '
    << (drawerAlias == object->Attributes()) << ' ' << drawerAlias->GetRefCount() << '\n';
}
static void parent_references(const occ::handle<ColoredOwnerProbe>& object,
  const occ::handle<PrsMgr_Presentation>& prs, const TopoDS_Shape& key)
{
  const auto& shapeAlias = object->Shape();
  const auto* shapeAddress = &shapeAlias;
  TopoDS_Vertex replacement;
  BRep_Builder().MakeVertex(replacement, gp_Pnt(4,5,6), 1.e-7);
  object->SetShape(replacement);
  std::cout << "shape-reference " << (&object->Shape() == shapeAddress) << ' '
    << shapeAlias.IsEqual(replacement) << ' ' << key.TShape()->GetRefCount() << ' '
    << replacement.TShape()->GetRefCount() << ' ' << prs->MustBeUpdated() << '\n';
  drawer_reference(object);
  object->SetShape(key);
}
static void alias_creation(const occ::handle<ColoredOwnerProbe>& object,
  const occ::handle<PrsMgr_Presentation>& prs, const TopoDS_Shape& key)
{
  const auto& retained = object->CustomAspectsMap();
  auto& mutableMap = object->ChangeCustomAspectsMap();
  prs->SetUpdateStatus(false);
  object->UnsetCustomAspects(key, false);
  std::cout << "missing-unset " << prs->MustBeUpdated() << ' ' << retained.Extent() << '\n';
  auto child = object->CustomAspects(key);
  std::cout << "created-alias " << prs->MustBeUpdated() << ' ' << retained.Extent()
    << ' ' << (&retained == &mutableMap) << '\n';
  prs->SetUpdateStatus(false);
  auto repeat = object->CustomAspects(key.Reversed());
  std::cout << "reused-alias " << prs->MustBeUpdated() << ' ' << (repeat == child) << '\n';
}
static void alias_replacement(const occ::handle<ColoredOwnerProbe>& object,
  const occ::handle<PrsMgr_Presentation>& prs, const TopoDS_Shape& key)
{
  const auto& map = object->CustomAspectsMap();
  const auto& alias = map.Find(key);
  const auto* address = &alias;
  auto previous = alias;
  previous->SetHidden(true);
  prs->SetUpdateStatus(false);
  object->UnsetCustomAspects(key, false);
  std::cout << "replaced-alias " << prs->MustBeUpdated() << ' ' << (&map.Find(key) == address)
    << ' ' << (alias != previous) << ' ' << alias->IsHidden() << '\n';
  prs->SetUpdateStatus(false);
  object->UnsetCustomAspects(key, true);
  std::cout << "removed-alias " << prs->MustBeUpdated() << ' ' << map.Extent() << '\n';
}
static void alias_null_slot(const occ::handle<ColoredOwnerProbe>& object,
  const occ::handle<PrsMgr_Presentation>& prs, const TopoDS_Shape& key)
{
  auto& map = object->ChangeCustomAspectsMap();
  prs->SetUpdateStatus(false);
  map.Bind(key.Reversed(), occ::handle<AIS_ColoredDrawer>());
  std::cout << "raw-bind " << prs->MustBeUpdated() << ' ' << map.Extent() << '\n';
  auto child = object->CustomAspects(key);
  std::cout << "filled-null " << prs->MustBeUpdated() << ' ' << map.Extent()
    << ' ' << int(map.Contained(key)->first.get().Orientation()) << '\n';
  object->ClearCustomAspects();
}
static void replace_release(const occ::handle<ColoredOwnerProbe>& object,
  const occ::handle<PrsMgr_Presentation>& prs, const TopoDS_Shape& key, bool unregister)
{
  const char* label = unregister ? "remove-release" : "replace-release";
  object->ChangeCustomAspectsMap().Bind(key, new ColoredReleaseDrawer(*object, *prs, label));
  prs->SetUpdateStatus(false);
  object->UnsetCustomAspects(key, unregister);
  std::cout << label << "-final " << prs->MustBeUpdated() << ' '
    << object->CustomAspectsMap().Extent() << '\n';
}
static void clear_release(const occ::handle<ColoredOwnerProbe>& object,
  const occ::handle<PrsMgr_Presentation>& prs, const TopoDS_Shape& key)
{
  object->ChangeCustomAspectsMap().Bind(key, new ColoredReleaseDrawer(*object, *prs, "clear-release"));
  prs->SetUpdateStatus(false);
  object->ClearCustomAspects();
  std::cout << "clear-final " << prs->MustBeUpdated() << ' ' << object->CustomAspectsMap().Extent() << '\n';
  prs->SetUpdateStatus(false);
  object->ClearCustomAspects();
  std::cout << "empty-clear " << prs->MustBeUpdated() << '\n';
}
static occ::handle<PrsMgr_Presentation> owner_presentation(
  const occ::handle<ColoredOwnerProbe>& object)
{
  occ::handle<Graphic3d_GraphicDriver> driver = new SvDriver;
  occ::handle<Graphic3d_StructureManager> structures = new Graphic3d_StructureManager(driver);
  occ::handle<PrsMgr_PresentationManager> manager = new PrsMgr_PresentationManager(structures);
  occ::handle<PrsMgr_Presentation> prs = new ColoredOwnerPresentation(manager, object);
  object->Presentations().Append(prs);
  prs->SetUpdateStatus(false);
  return prs;
}
int main()
{
  TopoDS_Vertex key;
  BRep_Builder().MakeVertex(key, gp_Pnt(1,2,3), 1.e-7);
  occ::handle<ColoredOwnerProbe> object = new ColoredOwnerProbe(key);
  owner_header(object, key);
  owner_runtime(object);
  owner_modes(object);
  auto prs = owner_presentation(object);
  parent_references(object, prs, key);
  alias_creation(object, prs, key);
  alias_replacement(object, prs, key);
  alias_null_slot(object, prs, key);
  replace_release(object, prs, key, false);
  replace_release(object, prs, key, true);
  clear_release(object, prs, key);
  owner_destruction();
  parent_destruction();
  return 0;
}
