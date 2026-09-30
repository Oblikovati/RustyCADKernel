#include <ShapeUpgrade_SplitCurve3d.hxx>
class ShSplitCurve3dProbe : public ShapeUpgrade_SplitCurve3d {
public:
  void SetRawStatus(int status) { myStatus = status; }
};
using ShSplitCurves3d =
    occ::handle<NCollection_HArray1<occ::handle<Geom_Curve>>>;
void sh_split3d_curves(const ShSplitCurves3d &curves) {
  ga_flag(curves.IsNull());
  if (curves.IsNull())
    return;
  ga_int(curves->Length());
  for (const auto &curve : *curves) {
    shc_curve(curve);
    sh_construct_spline3d(curve);
  }
}
void sh_split3d_state(const ShapeUpgrade_SplitCurve3d &splitter,
                      const ShSplitCurves3d &previous) {
  auto values = splitter.SplitValues();
  ga_int(values->Length());
  for (double value : *values)
    ga_real(value);
  for (auto status : {ShapeExtend_OK, ShapeExtend_DONE1, ShapeExtend_DONE2,
                      ShapeExtend_DONE3, ShapeExtend_FAIL})
    ga_flag(splitter.Status(status));
  ga_flag(splitter.GetCurves() == previous);
  sh_split3d_curves(splitter.GetCurves());
  sh_split3d_curves(previous);
}
void sh_split3d_operation(ShSplitCurve3dProbe &splitter, GaReader &reader) {
  int operation = reader.integer();
  if (operation == 0) {
    occ::handle<NCollection_HSequence<double>> values =
        new NCollection_HSequence<double>;
    int count = reader.integer();
    for (int i = 0; i < count; i++)
      values->Append(reader.real());
    GA_TRY(splitter.SetSplitValues(values))
  } else if (operation == 1 || operation == 2) {
    bool segment = reader.flag();
    GA_TRY(if (operation == 1) splitter.Build(segment);
           else splitter.Perform(segment))
  } else if (operation == 3) {
    double first = reader.real(), last = reader.real();
    GA_TRY(auto values = splitter.SplitValues(); values->SetValue(1, first);
           values->SetValue(values->Length(), last))
  } else
    splitter.SetRawStatus(reader.integer());
}
void sh_split3d_request(GaReader &reader) {
  auto input = tp_geom_curve(reader);
  ShSplitCurve3dProbe splitter;
  bool natural = reader.flag();
  double first = reader.real(), last = reader.real();
  if (natural)
    splitter.Init(input);
  else
    splitter.Init(input, first, last);
  sh_split3d_state(splitter, {});
  int count = reader.integer();
  for (int i = 0; i < count; i++) {
    auto previous = splitter.GetCurves();
    sh_split3d_operation(splitter, reader);
    sh_split3d_state(splitter, previous);
  }
  shc_curve(input);
  sh_construct_spline3d(input);
}
void sh_split_curve3d_case(std::istream &input) {
  GaReader reader{input};
  GA_TRY(sh_split3d_request(reader))
  ga_end();
}
