#include <SelectMgr_AxisIntersector.hxx>
#include <Standard_Failure.hxx>
#include <SelectMgr_FrustumBuilder.hxx>
#include <Standard_Type.hxx>
#include <iostream>
int main() {
  SelectMgr_AxisIntersector axis;
  std::cout << "dynamic=" << axis.DynamicType()->Name()
            << " static=" << SelectMgr_AxisIntersector::get_type_descriptor()->Name()
            << " kind_axis=" << axis.IsKind("SelectMgr_AxisIntersector") << '\n';
  try { axis.GetNearPnt(); std::cout << "uninitialized_near=returned\n"; }
  catch (const Standard_Failure& failure) { std::cout << "uninitialized_near=" << failure.ExceptionType() << '\n'; }
  axis.Init(gp_Ax1(gp_Pnt(1,2,3),gp_Dir(0,1,0)));
  auto identity=axis.ScaleAndTransform(9,gp_GTrsf(),nullptr);
  auto copied=axis.CopyWithBuilder(nullptr);
  std::cout << "identity_selection=" << identity->GetSelectionType()
            << " copied_selection=" << copied->GetSelectionType() << '\n';
}
