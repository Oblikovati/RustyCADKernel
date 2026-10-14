// SPDX-License-Identifier: GPL-2.0-only
// OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871: actual MeshVS_Drawer storage/virtual dispatch.
#include <MeshVS_Drawer.hxx>
#include <MeshVS_DrawerAttribute.hxx>
#include <MeshVS_DisplayModeFlags.hxx>
#include <MeshVS_EntityType.hxx>
#include <Standard_ConstructionError.hxx>
#include <iostream>
#include <iomanip>
#include <stdexcept>
class AssignProbe : public MeshVS_Drawer {
public:
  int fault=0;
  void Assign(const occ::handle<MeshVS_Drawer>& source) override {
    std::cout << "callback " << GetRefCount() << " " << source->GetRefCount() << "\n";
    MeshVS_Drawer::Assign(source);
    if(fault==1) throw Standard_ConstructionError("drawer assign after copy");
    if(fault==2) throw std::logic_error("drawer assign foreign after copy");
  }
};
static void Scalars(const occ::handle<MeshVS_Drawer>& drawer) {
  int integer=91; double real=7.25; bool boolean=true;
  std::cout << "miss " << drawer->GetInteger(7,integer) << " " << integer << " "
    << drawer->GetDouble(7,real) << " " << real << " "
    << drawer->GetBoolean(7,boolean) << " " << boolean << "\n";
  drawer->SetInteger(7,-23); drawer->SetDouble(7,3.5); drawer->SetBoolean(7,false);
  drawer->SetInteger(7,42);
  std::cout << "scalar " << drawer->GetInteger(7,integer) << " " << integer << " "
    << drawer->GetDouble(7,real) << " " << real << " "
    << drawer->GetBoolean(7,boolean) << " " << boolean << "\n";
}
static void Values(const occ::handle<MeshVS_Drawer>& drawer) {
  Quantity_Color color(.125,.25,.5,Quantity_TOC_RGB), output(Quantity_NOC_RED);
  Graphic3d_MaterialAspect material(Graphic3d_NOM_GOLD), result(Graphic3d_NOM_BRASS);
  TCollection_AsciiString text("native drawer"), copied("sentinel");
  std::cout << "miss-values " << drawer->GetColor(-3,output) << " " << output.Red() << " "
    << drawer->GetMaterial(-3,result) << " " << result.Name() << " "
    << drawer->GetAsciiString(-3,copied) << " " << copied.ToCString() << "\n";
  material.SetTransparency(.375f); drawer->SetColor(-3,color);
  drawer->SetMaterial(-3,material); drawer->SetAsciiString(-3,text);
  material.SetTransparency(.75f); text.SetValue(1,'X');
  drawer->GetColor(-3,output); drawer->GetMaterial(-3,result); drawer->GetAsciiString(-3,copied);
  std::cout << "values " << output.Red() << " " << output.Green() << " " << output.Blue() << " "
    << result.Name() << " " << result.Transparency() << " " << copied.ToCString() << "\n";
}
static void Assignment(const occ::handle<MeshVS_Drawer>& source) {
  occ::handle<MeshVS_Drawer> target=new MeshVS_Drawer;
  target->Assign(source); source->Assign(source); source->Assign(occ::handle<MeshVS_Drawer>());
  source->SetInteger(7,99); int integer=0; target->GetInteger(7,integer);
  std::cout << "assign " << integer << " " << source->GetRefCount() << " " << target->GetRefCount() << "\n";
  std::cout << "remove " << target->RemoveInteger(7) << " " << target->RemoveInteger(7) << " "
    << target->RemoveDouble(7) << " " << target->RemoveBoolean(7) << " "
    << target->RemoveColor(-3) << " " << target->RemoveMaterial(-3) << " " << target->RemoveAsciiString(-3) << "\n";
}
static void Dispatch(const occ::handle<MeshVS_Drawer>& source) {
  AssignProbe* raw=new AssignProbe;
  occ::handle<MeshVS_Drawer> target=raw;
  for(int fault=0;fault<3;++fault) {
    raw->fault=fault;
    try { target->Assign(source); std::cout << "return 0\n"; }
    catch(const Standard_ConstructionError&) {std::cout << "return 1\n";}
    catch(const std::logic_error&) {std::cout << "return 2\n";}
    int value=0; target->GetInteger(7,value);
    std::cout << "after " << value << " " << source->GetRefCount() << " " << target->GetRefCount() << "\n";
  }
}
int main() {
  std::cout << std::setprecision(9);
  occ::handle<MeshVS_Drawer> drawer=new MeshVS_Drawer;
  Scalars(drawer); Values(drawer); Assignment(drawer); Dispatch(drawer);
  std::cout << "rtti " << drawer->DynamicType()->Name() << " "
    << drawer->IsKind(STANDARD_TYPE(Standard_Transient)) << " "
    << drawer->IsInstance(STANDARD_TYPE(MeshVS_Drawer)) << "\n";
  std::cout << "flags " << MeshVS_DA_User << " " << MeshVS_DMF_OCCMask << " "
    << MeshVS_DMF_DeformedMask << " " << MeshVS_ET_Element << " " << MeshVS_ET_All << "\n";
}
