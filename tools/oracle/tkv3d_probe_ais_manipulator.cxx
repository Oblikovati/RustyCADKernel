// AIS_Manipulator attached to a point of a 400x300 top view (with a second point framing
// the view): construction, parts, sizes, skins, persistence, presentation and selection
// geometry, detection and selection of parts, interactive transformations and detachment,
// the twin of crates/tkv3d/examples/v3_probe/ais_manipulator.rs. A request is
// `v3_ais_manipulator Count (Op A B C)...`.
#include <AIS_Manipulator.hxx>
#include <AIS_ManipulatorOwner.hxx>
#include <AIS_Point.hxx>
#include <Aspect_NeutralWindow.hxx>
#include <Geom_CartesianPoint.hxx>

class V3ProbeManipulator : public AIS_Manipulator {
public:
  int index() const { return myCurrentIndex; }
  occ::handle<Graphic3d_Group> group(int i, int m) const { return getGroup(i, AIS_ManipulatorMode(m)); }
  void deactivate() { DeactivateCurrentMode(); }
};

static void v3_trsf(const gp_Trsf& t) {
  for (int r = 1; r <= 3; ++r) for (int c = 1; c <= 4; ++c) ga_real(t.Value(r, c));
}

static void v3_manipulator_state(const occ::handle<V3ProbeManipulator>& m, const occ::handle<AIS_InteractiveContext>& ctx,
                                 const occ::handle<AIS_Point>& object) {
  ga_flag(m->IsAttached()); ga_flag(m->HasActiveMode()); ga_int(m->ActiveMode()); ga_int(m->ActiveAxisIndex());
  ga_flag(m->HasActiveTransformation()); ga_flag(m->ZoomPersistence()); ga_int(m->SkinMode()); ga_real(m->Size());
  ga_flag(m->IsModeActivationOnDetection());
  const gp_Ax2& p = m->Position();
  for (double v : {p.Location().X(), p.Location().Y(), p.Location().Z(), p.Direction().X(), p.Direction().Y(),
                   p.Direction().Z(), p.XDirection().X(), p.XDirection().Y(), p.XDirection().Z()}) ga_real(v);
  v3_trsf(m->LocalTransformation()); v3_trsf(object->LocalTransformation()); v3_trsf(m->StartTransformation());
  const occ::handle<Graphic3d_TransformPers>& pers = m->TransformPersistence();
  ga_flag(pers.IsNull());
  if (!pers.IsNull()) { ga_int(pers->Mode()); if (pers->IsZoomOrRotate() || pers->IsAxial()) { gp_Pnt a = pers->AnchorPoint(); ga_real(a.X()); ga_real(a.Y()); ga_real(a.Z()); } }
  ga_flag(ctx->IsDisplayed(m)); ga_int(ctx->NbSelected()); ga_flag(ctx->HasDetected());
  for (int mode = 1; mode <= 4; ++mode) ga_flag(ctx->SelectionManager()->IsActivated(m, mode));
  ga_int(m->Presentations().Size());
  for (int i = 0; i < 3; ++i) for (int mode = 1; mode <= 4; ++mode) ga_flag(m->group(i, mode).IsNull());
}

static void v3_manipulator_selection(const occ::handle<V3ProbeManipulator>& m, int mode) {
  occ::handle<SelectMgr_Selection> s = new SelectMgr_Selection(mode);
  static_cast<SelectMgr_SelectableObject*>(m.get())->ComputeSelection(s, mode);
  ga_int(s->Entities().Size());
  for (const auto& e : s->Entities()) {
    const occ::handle<Select3D_SensitiveEntity>& sens = e->BaseSensitive();
    ga_int(sens->NbSubElements()); ga_int(sens->SensitivityFactor());
    occ::handle<AIS_ManipulatorOwner> owner = occ::down_cast<AIS_ManipulatorOwner>(sens->OwnerId());
    ga_int(owner.IsNull() ? -1 : owner->Index()); ga_int(owner.IsNull() ? -1 : owner->Mode()); ga_int(sens->OwnerId()->Priority());
    Select3D_BndBox3d b = sens->BoundingBox();
    ga_flag(b.IsValid());
    if (b.IsValid()) for (double v : {b.CornerMin().x(), b.CornerMin().y(), b.CornerMin().z(), b.CornerMax().x(), b.CornerMax().y(), b.CornerMax().z()}) ga_real(v);
  }
}

void v3_ais_manipulator(std::istream& input) {
  GaReader r{input};
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer = new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<V3d_View> view = viewer->CreateView();
  occ::handle<Aspect_NeutralWindow> window = new Aspect_NeutralWindow(); window->SetSize(400, 300); view->SetWindow(window);
  occ::handle<AIS_InteractiveContext> ctx = new AIS_InteractiveContext(viewer);
  occ::handle<AIS_Point> object = new AIS_Point(new Geom_CartesianPoint(10.0, 20.0, 0.0));
  occ::handle<AIS_Point> other = new AIS_Point(new Geom_CartesianPoint(150.0, 120.0, 0.0));
  ctx->Display(object, false); ctx->Display(other, false);
  view->SetProj(V3d_TypeOfOrientation_Zup_Top); view->FitAll(0.1, false);
  int px = 0, py = 0; view->Convert(10.0, 20.0, 0.0, px, py); ga_int(px); ga_int(py);
  occ::handle<V3ProbeManipulator> m = new V3ProbeManipulator();
  bool dump = false;
  trace.events.clear();
  v3_manipulator_state(m, ctx, object);
  for (int count = r.integer(); count > 0; --count) {
    const int op = r.integer(), a = r.integer(), b = r.integer(), c = r.integer();
    switch (op) {
      case 0: { AIS_Manipulator::OptionsForAttach o; o.SetAdjustPosition(a != 0).SetAdjustSize(b != 0).SetEnableModes(c != 0); GA_TRY(m->Attach(object, o)) break; }
      case 1: GA_TRY(m->SetPart(a, AIS_ManipulatorMode(b), c != 0)) break;
      case 2: m->SetPart(AIS_ManipulatorMode(a), b != 0); break;
      case 3: m->SetSize(float(a)); break;
      case 4: m->SetGap(float(a) / 10.0f); break;
      case 5: m->SetSkinMode(AIS_Manipulator::ManipulatorSkin(a)); break;
      case 6: GA_TRY(m->SetZoomPersistence(a != 0)) break;
      case 7: m->SetModeActivationOnDetection(a != 0); break;
      case 8: GA_TRY(v3_manipulator_selection(m, a)) break;
      case 9: ga_int(ctx->MoveTo(px + a, py + b, view, false)); break;
      case 10: ga_int(ctx->SelectDetected()); break;
      case 11: GA_TRY(m->StartTransform(px + a, py + b, view)) break;
      case 12: GA_TRY(v3_trsf(m->Transform(px + a, py + b, view))) break;
      case 13: m->StopTransform(a != 0); break;
      case 14: GA_TRY(m->Detach()) break;
      case 15: m->deactivate(); break;
      case 16: GA_TRY(m->RecomputeTransformation(view->Camera())) break;
      case 17: m->SetPosition(gp_Ax2(gp_Pnt(a, b, c), gp::DZ())); break;
      case 18: GA_TRY(ga_flag(m->ProcessDragging(ctx, view, nullptr, NCollection_Vec2<int>(px + b, py + c), NCollection_Vec2<int>(px + b, py + c), AIS_DragAction(a)))) break;
      case 19: dump = a != 0; break;
      case 20: ctx->ClearSelected(false); break;
      case 21: GA_TRY(m->RecomputeSelection(AIS_ManipulatorMode(a))) break;
      case 22: { AIS_Manipulator::BehaviorOnTransform bt; bt.SetFollowTranslation(a != 0).SetFollowRotation(b != 0).SetFollowDragging(c != 0); m->SetTransformBehavior(bt); break; }
      case 23: ctx->Redisplay(m, false); break;
    }
    if (dump) { ga_int((int)trace.events.size()); for (double e : trace.events) ga_real(e); }
    trace.events.clear();
    v3_manipulator_state(m, ctx, object);
  }
  ga_end();
}
