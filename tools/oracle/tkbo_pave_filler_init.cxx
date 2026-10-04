#include <BOPAlgo_Alerts.hxx>
#include <BOPAlgo_PaveFiller.hxx>
#include <BOPAlgo_GlueEnum.hxx>
#include <BOPDS_DS.hxx>
#include <BOPDS_Iterator.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <NCollection_List.hxx>
#include <TopoDS_Shape.hxx>
#include <iostream>

class ExposedPaveFiller : public BOPAlgo_PaveFiller {
public:
  void Initialize() { Init(Message_ProgressRange()); }
  void SetPrimary(const bool value) { SetIsPrimary(value); }
  BOPDS_Iterator* PreparedIterator() { return myIterator; }
  void ClearPrepared() { Clear(); }
  bool IsPrimarySetting() const { return IsPrimary(); }
};

static NCollection_List<TopoDS_Shape> overlapping_boxes() {
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(BRepPrimAPI_MakeBox(1.0, 1.0, 1.0).Shape());
  arguments.Append(BRepPrimAPI_MakeBox(gp_Pnt(0.5, 0.0, 0.0), 1.0, 1.0, 1.0).Shape());
  return arguments;
}

static void report_valid_initialization() {
  ExposedPaveFiller filler;
  filler.SetArguments(overlapping_boxes());
  filler.Initialize();
  BOPDS_Iterator* iterator = filler.PreparedIterator();
  iterator->Initialize(TopAbs_VERTEX, TopAbs_EDGE);
  int first = -1, second = -1;
  iterator->Value(first, second);
  std::cout << "valid " << filler.HasErrors() << ' '
            << filler.DS().NbSourceShapes() << ' ' << filler.DS().NbRanges() << ' '
            << iterator->ExpectedLength() << ' ' << first << '-' << second << ' '
            << filler.NonDestructive() << ' ' << filler.Context().IsNull() << '\n';
}

static void report_early_failures() {
  ExposedPaveFiller empty;
  empty.Initialize();
  std::cout << "empty " << empty.HasErrors() << ' ' << (empty.PDS() == nullptr) << '\n';

  ExposedPaveFiller nullShape;
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(TopoDS_Shape());
  nullShape.SetArguments(arguments);
  nullShape.Initialize();
  std::cout << "null " << nullShape.HasErrors() << ' ' << (nullShape.PDS() == nullptr) << '\n';
}

static void report_locked_input() {
  TopoDS_Shape box = BRepPrimAPI_MakeBox(1.0, 1.0, 1.0).Shape();
  box.Locked(true);
  ExposedPaveFiller filler;
  NCollection_List<TopoDS_Shape> arguments;
  arguments.Append(box);
  filler.SetArguments(arguments);
  filler.Initialize();
  std::cout << "locked " << filler.NonDestructive() << '\n';
}

static void report_obb_preparation() {
  ExposedPaveFiller filler;
  filler.SetArguments(overlapping_boxes());
  filler.SetUseOBB(true);
  filler.Initialize();
  BOPDS_Iterator* iterator = filler.PreparedIterator();
  iterator->Initialize(TopAbs_VERTEX, TopAbs_EDGE);
  int first = -1, second = -1;
  if (iterator->More()) {
    iterator->Value(first, second);
  }
  std::cout << "obb_ve " << iterator->ExpectedLength() << ' ' << first << '-' << second << '\n';
  first = -1;
  second = -1;
  iterator->Initialize(TopAbs_VERTEX, TopAbs_FACE);
  if (iterator->More()) {
    iterator->Value(first, second);
  }
  std::cout << "obb_vf " << iterator->ExpectedLength() << ' ' << first << '-' << second << '\n';
}

static void report_retained_state_on_invalid_reinitialization() {
  ExposedPaveFiller filler;
  filler.SetArguments(overlapping_boxes());
  filler.Initialize();
  NCollection_List<TopoDS_Shape> invalid;
  invalid.Append(TopoDS_Shape());
  filler.SetArguments(invalid);
  filler.Initialize();
  std::cout << "stale " << filler.HasErrors() << ' '
            << filler.DS().NbSourceShapes() << '\n';
}

static void report_configuration_after_clear() {
  ExposedPaveFiller filler;
  filler.SetArguments(overlapping_boxes());
  filler.SetNonDestructive(true);
  filler.SetGlue(BOPAlgo_GlueShift);
  filler.SetAvoidBuildPCurve(true);
  filler.SetPrimary(false);
  filler.SetRunParallel(true);
  filler.SetFuzzyValue(0.25);
  filler.SetUseOBB(true);
  filler.ClearPrepared();
  std::cout << "config " << filler.Arguments().Extent() << ' '
            << filler.NonDestructive() << ' ' << filler.Glue() << ' '
            << filler.IsAvoidBuildPCurve() << ' ' << filler.IsPrimarySetting() << ' '
            << filler.RunParallel() << ' ' << filler.FuzzyValue() << ' '
            << filler.UseOBB() << '\n';
}

int main() {
  report_valid_initialization();
  report_early_failures();
  report_locked_input();
  report_obb_preparation();
  report_retained_state_on_invalid_reinitialization();
  report_configuration_after_clear();
}
