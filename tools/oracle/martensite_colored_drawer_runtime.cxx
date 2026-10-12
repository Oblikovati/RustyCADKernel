// Martensite actual application subclass/virtual JSON/flag storage oracle.
#include <AIS_ColoredDrawer.hxx>
#include <sstream>
#include <iostream>
class MartensiteColoredDrawerProbe : public AIS_ColoredDrawer
{
  DEFINE_STANDARD_RTTIEXT(MartensiteColoredDrawerProbe, AIS_ColoredDrawer)
public:
  MartensiteColoredDrawerProbe(const occ::handle<Prs3d_Drawer>& link) : AIS_ColoredDrawer(link)
  {
    // Native parent construction retains the same linked drawer owner.
  }
  void DumpJson(std::ostream& stream, int) const override
  {
    // A named application override detects loss of virtual dispatch through base handles.
    stream << "application-json";
  }
};
IMPLEMENT_STANDARD_RTTIEXT(MartensiteColoredDrawerProbe, AIS_ColoredDrawer)
static void individual_flags(const occ::handle<AIS_ColoredDrawer>& drawer)
{
  bool& material = drawer->myHasOwnMaterial;
  drawer->SetOwnMaterial();
  std::cout << "material " << material << drawer->HasOwnColor() << drawer->HasOwnTransparency() << drawer->HasOwnWidth() << '\n';
  drawer->UnsetOwnMaterial();
  drawer->SetOwnColor(Quantity_Color(Quantity_NOC_RED));
  std::cout << "color " << material << drawer->HasOwnColor() << drawer->HasOwnTransparency() << drawer->HasOwnWidth() << '\n';
  drawer->UnsetOwnColor();
  drawer->SetOwnTransparency(-20.);
  std::cout << "transparency " << material << drawer->HasOwnColor() << drawer->HasOwnTransparency() << drawer->HasOwnWidth() << '\n';
  drawer->UnsetOwnTransparency();
  drawer->SetOwnWidth(-20.);
  std::cout << "width " << material << drawer->HasOwnColor() << drawer->HasOwnTransparency() << drawer->HasOwnWidth() << '\n';
}
static void retained_link()
{
  occ::handle<Prs3d_Drawer> first = new Prs3d_Drawer();
  occ::handle<Prs3d_Drawer> replacement = new Prs3d_Drawer();
  occ::handle<AIS_ColoredDrawer> drawer = new AIS_ColoredDrawer(first);
  const occ::handle<Prs3d_Drawer>& alias = drawer->Link();
  const occ::handle<Prs3d_Drawer>& secondAlias = drawer->Link();
  std::cout << "link-before " << (&alias == &secondAlias) << ' ' << (alias == first)
            << ' ' << first->GetRefCount() << ' ' << replacement->GetRefCount() << '\n';
  drawer->SetLink(replacement);
  std::cout << "link-replaced " << (alias == replacement) << ' ' << first->GetRefCount()
            << ' ' << replacement->GetRefCount() << '\n';
  drawer->SetLink(occ::handle<Prs3d_Drawer>());
  std::cout << "link-cleared " << alias.IsNull() << ' ' << replacement->GetRefCount() << '\n';
}
static void inherited_json(const occ::handle<Prs3d_Drawer>& defaults)
{
  occ::handle<AIS_ColoredDrawer> direct = new AIS_ColoredDrawer(defaults);
  std::ostringstream inheritedJson;
  direct->DumpJson(inheritedJson, 0);
  std::cout << "inherited-json " << (inheritedJson.str().find("Prs3d_Drawer") != std::string::npos) << '\n';
}
static void application_runtime(const occ::handle<Prs3d_Drawer>& defaults)
{
  occ::handle<MartensiteColoredDrawerProbe> application = new MartensiteColoredDrawerProbe(defaults);
  occ::handle<Prs3d_Drawer> base(application.get());
  occ::handle<AIS_ColoredDrawer> colored = occ::handle<AIS_ColoredDrawer>::DownCast(base);
  std::ostringstream json;
  base->DumpJson(json, 0);
  std::cout << "runtime " << colored->DynamicType()->Name() << ' ' << colored->DynamicType()->Parent()->Name()
            << ' ' << colored->DynamicType()->Parent()->Parent()->Name() << ' ' << (colored == application)
            << ' ' << application->GetRefCount() << ' ' << defaults->GetRefCount() << ' ' << json.str() << '\n';
  individual_flags(colored);
  base.Nullify();
  colored.Nullify();
  std::cout << "release " << application->GetRefCount() << ' ' << defaults->GetRefCount() << '\n';
  application.Nullify();
  std::cout << "destroy " << defaults->GetRefCount() << '\n';
}
int main()
{
  occ::handle<Prs3d_Drawer> defaults = new Prs3d_Drawer();
  application_runtime(defaults);
  inherited_json(defaults);
  retained_link();
}
