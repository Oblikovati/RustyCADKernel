// Actual selected application-owner values and intrusive counts; pinned OCCT 3d097a0.
#include "tkservice_probe_null_driver.cxx"
#include <AIS_InteractiveContext.hxx>
#include <AIS_Point.hxx>
#include <Geom_CartesianPoint.hxx>
#include <V3d_Viewer.hxx>
#include <iostream>

static void recordApplication(int step, const occ::handle<AIS_InteractiveContext>& context,
                              const occ::handle<Standard_Transient>& first,
                              const occ::handle<Standard_Transient>& second) {
  auto application = context->Applicative();
  std::cout << step << ' ' << context->HasApplicative() << ' '
            << (!application.IsNull() ? (application == first ? 1 : 2) : 0) << ' '
            << first->GetRefCount() << ' ' << second->GetRefCount() << '\n';
}

int main() {
  const occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new SvDriver());
  const occ::handle<AIS_InteractiveContext> context = new AIS_InteractiveContext(viewer);
  const occ::handle<AIS_Point> object = new AIS_Point(new Geom_CartesianPoint(0,0,0));
  const occ::handle<SelectMgr_EntityOwner> owner = new SelectMgr_EntityOwner(object);
  const occ::handle<Standard_Transient> first = new Standard_Transient, second = new Standard_Transient;
  context->Selection()->AddSelect(owner); context->InitSelected(); recordApplication(0,context,first,second);
  object->SetOwner(first); recordApplication(1,context,first,second);
  auto retained = context->Applicative(); object->SetOwner(second); recordApplication(2,context,first,second);
  object->ClearOwner(); recordApplication(3,context,first,second);
  retained.Nullify(); recordApplication(4,context,first,second);
  const occ::handle<AIS_Point> other = new AIS_Point(new Geom_CartesianPoint(1,2,3));
  const occ::handle<SelectMgr_EntityOwner> otherOwner = new SelectMgr_EntityOwner(other);
  other->SetOwner(first); context->Selection()->AddSelect(otherOwner); context->NextSelected();
  recordApplication(5,context,first,second);
  object->SetOwner(second); recordApplication(6,context,first,second);
}
