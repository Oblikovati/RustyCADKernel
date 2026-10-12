// Martensite native flag/link/owned-base oracle at OCCT 3d097a0.
#include <AIS_ColoredDrawer.hxx>
#include <Graphic3d_AspectLine3d.hxx>
#include <Prs3d_LineAspect.hxx>
#include <limits>
#include <iostream>

static void emit(int index, const occ::handle<AIS_ColoredDrawer>& drawer)
{
  std::cout << index << ' ' << drawer->IsHidden() << drawer->HasOwnMaterial()
            << drawer->HasOwnColor() << drawer->HasOwnTransparency() << drawer->HasOwnWidth()
            << ' ' << drawer->Discretisation() << ' ' << drawer->HasOwnLineAspect()
            << ' ' << drawer->LineAspect()->Aspect()->Width()
            << ' ' << drawer->HasOwnShadingAspect()
            << ' ' << (drawer->LineAspect()->Aspect()->Color() == Quantity_Color(Quantity_NOC_BLUE)) << '\n';
}
static void set_flags(const occ::handle<AIS_ColoredDrawer>& drawer, double argument)
{
  drawer->SetHidden(true);
  drawer->SetOwnMaterial();
  drawer->SetOwnColor(Quantity_Color(Quantity_NOC_RED));
  drawer->SetOwnTransparency(argument);
  drawer->SetOwnWidth(argument);
}
static void clear_flags(const occ::handle<AIS_ColoredDrawer>& drawer)
{
  drawer->SetHidden(false);
  drawer->UnsetOwnMaterial();
  drawer->UnsetOwnColor();
  drawer->UnsetOwnTransparency();
  drawer->UnsetOwnWidth();
}
static void emit_aliases(const occ::handle<AIS_ColoredDrawer>& drawer,
                         const occ::handle<Prs3d_Drawer>& defaults)
{
  occ::handle<Prs3d_Drawer> base(drawer.get());
  occ::handle<AIS_ColoredDrawer> back = occ::handle<AIS_ColoredDrawer>::DownCast(base);
  std::cout << "aliases " << (back == drawer) << ' ' << (base == drawer)
            << ' ' << (drawer->Link() == defaults) << ' ' << drawer->GetRefCount()
            << ' ' << defaults->GetRefCount() << ' ' << drawer->DynamicType()->Name()
            << ' ' << drawer->DynamicType()->Parent()->Name() << '\n';
}
static void probe_public_fields(const occ::handle<AIS_ColoredDrawer>& drawer)
{
  bool& retainedHidden = drawer->myIsHidden;
  drawer->SetHidden(true);
  std::cout << "field " << retainedHidden << ' ';
  retainedHidden = false;
  drawer->myHasOwnMaterial = true;
  drawer->myHasOwnColor = true;
  drawer->myHasOwnTransp = true;
  drawer->myHasOwnWidth = true;
  emit(40, drawer);
}
static void ignored_arguments(const occ::handle<AIS_ColoredDrawer>& drawer)
{
  const double args[] = {-2., 0., 2., std::numeric_limits<double>::infinity(), std::numeric_limits<double>::quiet_NaN()};
  for (int index = 0; index < 5; ++index) {
    set_flags(drawer, args[index]);
    emit(1 + 2 * index, drawer);
    clear_flags(drawer);
    emit(2 + 2 * index, drawer);
  }
}
static void unlinked_drawer()
{
  occ::handle<Prs3d_Drawer> nullLink;
  occ::handle<AIS_ColoredDrawer> unlinked = new AIS_ColoredDrawer(nullLink);
  std::cout << "unlinked " << unlinked->HasLink() << ' ' << unlinked->IsHidden()
            << unlinked->HasOwnMaterial() << unlinked->HasOwnColor()
            << unlinked->HasOwnTransparency() << unlinked->HasOwnWidth() << '\n';
}
int main()
{
  occ::handle<Prs3d_Drawer> defaults = new Prs3d_Drawer();
  defaults->SetDiscretisation(31);
  defaults->SetLineAspect(new Prs3d_LineAspect(Quantity_NOC_BLUE, Aspect_TOL_SOLID, 2.5));
  occ::handle<AIS_ColoredDrawer> drawer = new AIS_ColoredDrawer(defaults);
  emit(0, drawer);
  ignored_arguments(drawer);
  defaults->SetDiscretisation(73);
  emit(20, drawer);
  drawer->SetDiscretisation(91);
  emit(21, drawer);
  emit_aliases(drawer, defaults);
  probe_public_fields(drawer);
  unlinked_drawer();
  std::cout << "types " << AIS_ColoredDrawer::get_type_name() << ' '
            << (AIS_ColoredDrawer::get_type_descriptor() == drawer->DynamicType()) << '\n';
}
