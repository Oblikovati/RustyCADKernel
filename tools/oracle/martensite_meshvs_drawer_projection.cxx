// OCCT 3d097a0: a similarly named extension accessor cannot replace inherited maps.
#include <MeshVS_Drawer.hxx>
#include <MeshVS_Tool.hxx>
#include <MeshVS_DrawerAttribute.hxx>
#include <Graphic3d_AspectLine3d.hxx>
#include <Graphic3d_AspectFillArea3d.hxx>
#include <Graphic3d_AspectMarker3d.hxx>
#include <Graphic3d_AspectText3d.hxx>
#include <iostream>
class DrawerAccessorControl : public MeshVS_Drawer {
  DEFINE_STANDARD_RTTIEXT(DrawerAccessorControl, MeshVS_Drawer)
public:
  MeshVS_Drawer redirected;
  mutable int calls = 0;
  const MeshVS_Drawer& DrawerBase() const { ++calls; return redirected; }
  DrawerAccessorControl(int value) {
    SetInteger(77,value); redirected.SetInteger(77,value+100);
    SetDouble(MeshVS_DA_BeamWidth,2.5);
    redirected.SetDouble(MeshVS_DA_BeamWidth,8.75);
  }
};
IMPLEMENT_STANDARD_RTTIEXT(DrawerAccessorControl, MeshVS_Drawer)
static void Factories() {
  occ::handle<DrawerAccessorControl> owner = new DrawerAccessorControl(42);
  occ::handle<MeshVS_Drawer> source = owner;
  auto line = MeshVS_Tool::CreateAspectLine3d(source,true);
  Graphic3d_MaterialAspect material;
  std::cout << "factories " << line->Width() << ' '
    << !MeshVS_Tool::CreateAspectFillArea3d(source,true).IsNull() << ' '
    << !MeshVS_Tool::CreateAspectFillArea3d(source,material,true).IsNull() << ' '
    << !MeshVS_Tool::CreateAspectMarker3d(source,true).IsNull() << ' '
    << !MeshVS_Tool::CreateAspectText3d(source,true).IsNull() << ' '
    << owner->calls << ' ' << owner->GetRefCount() << ' ' << line->GetRefCount() << '\n';
}
static void Assignment() {
  occ::handle<DrawerAccessorControl> target = new DrawerAccessorControl(17);
  occ::handle<DrawerAccessorControl> source = new DrawerAccessorControl(42);
  occ::handle<MeshVS_Drawer> targetBase = target, sourceBase = source;
  targetBase->Assign(sourceBase);
  int actual=0, redirected=0;
  target->GetInteger(77,actual); target->redirected.GetInteger(77,redirected);
  std::cout << "assign " << actual << ' ' << redirected << ' ' << target->calls
    << ' ' << source->calls << ' ' << target->GetRefCount() << ' '
    << source->GetRefCount() << '\n';
}
static void Inherited() {
  occ::handle<DrawerAccessorControl> owner = new DrawerAccessorControl(42);
  occ::handle<MeshVS_Drawer> source = owner;
  source->SetInteger(77,51);
  int actual=0, redirected=0;
  owner->GetInteger(77,actual); owner->redirected.GetInteger(77,redirected);
  std::cout << "inherited " << actual << ' ' << redirected << ' ' << owner->calls << '\n';
}
int main() { Factories(); Assignment(); Inherited(); }
