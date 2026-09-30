class ShSplitCurveProbe : public ShSplitCurveBase {
public:
  void SetRawStatus(int status) { myStatus = status; }
};
using ShSplitCurves = occ::handle<NCollection_HArray1<occ::handle<ShCurve>>>;
void curves(const ShSplitCurves &curves) {
  ga_flag(curves.IsNull());
  if (curves.IsNull())
    return;
  ga_int(curves->Length());
  for (const auto &curve : *curves) {
    digest(curve);
  }
}
void state(const ShSplitCurveBase &splitter, const ShSplitCurves &previous) {
  auto values = splitter.SplitValues();
  ga_int(values->Length());
  for (double value : *values)
    ga_real(value);
  for (auto status : {ShapeExtend_OK, ShapeExtend_DONE1, ShapeExtend_DONE2,
                      ShapeExtend_DONE3, ShapeExtend_FAIL})
    ga_flag(splitter.Status(status));
  ga_flag(splitter.GetCurves() == previous);
  curves(splitter.GetCurves());
  curves(previous);
}
void operation(ShSplitCurveProbe &splitter, GaReader &reader) {
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
void request(GaReader &reader) {
  auto input = read_curve(reader);
  ShSplitCurveProbe splitter;
  bool natural = reader.flag();
  double first = reader.real(), last = reader.real();
  if (natural)
    splitter.Init(input);
  else
    splitter.Init(input, first, last);
  state(splitter, {});
  int count = reader.integer();
  for (int i = 0; i < count; i++) {
    auto previous = splitter.GetCurves();
    operation(splitter, reader);
    state(splitter, previous);
  }
  digest(input);
}
void run(std::istream &input) {
  GaReader reader{input};
  GA_TRY(request(reader))
  ga_end();
}
