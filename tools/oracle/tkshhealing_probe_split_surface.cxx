#include <ShapeUpgrade_SplitSurface.hxx>
class ShSplitSurfaceProbe : public ShapeUpgrade_SplitSurface {
public:
  void SetRawStatus(int value) { myStatus = value; }
  void PrintFields(bool measure) {
    ga_int(myNbResultingRow);
    ga_int(myNbResultingCol);
    if (measure) {
      ga_real(myArea);
      ga_real(myUsize);
      ga_real(myVsize);
    }
  }
};
void sh_split_surface_midpoint(const occ::handle<Geom_Surface> &surface) {
  double u1, u2, v1, v2;
  surface->Bounds(u1, u2, v1, v2);
  for (double value : {u1, u2, v1, v2})
    if (Precision::IsInfinite(value))
      return;
  GA_TRY(gp_Pnt point; gp_Vec du, dv;
         surface->D1((u1 + u2) * .5, (v1 + v2) * .5, point, du, dv);
         ga_pnt(point); ga_xyz(du.XYZ()); ga_xyz(dv.XYZ()))
}
void sh_split_surface_result(
    const occ::handle<ShapeExtend_CompositeSurface> &result,
    const occ::handle<Geom_Surface> &input) {
  ga_flag(result.IsNull());
  if (result.IsNull())
    return;
  ga_flag(result->Patches().IsNull());
  if (result->Patches().IsNull())
    return;
  ga_int(result->NbUPatches());
  ga_int(result->NbVPatches());
  for (auto values : {result->UJointValues(), result->VJointValues()})
    for (double value : *values)
      ga_real(value);
  for (int row = 1; row <= result->NbUPatches(); row++)
    for (int column = 1; column <= result->NbVPatches(); column++) {
      auto patch = result->Patch(row, column);
      ga_flag(patch == input);
      shc_surface(patch);
      sh_split_surface_midpoint(patch);
    }
}
void sh_split_surface_state(
    ShSplitSurfaceProbe &splitter, const occ::handle<Geom_Surface> &input,
    const occ::handle<ShapeExtend_CompositeSurface> &previous, bool measure) {
  for (auto values : {splitter.USplitValues(), splitter.VSplitValues()}) {
    ga_int(values->Length());
    for (double value : *values)
      ga_real(value);
  }
  for (auto status : {ShapeExtend_OK, ShapeExtend_DONE1, ShapeExtend_DONE2,
                      ShapeExtend_DONE3, ShapeExtend_FAIL})
    ga_flag(splitter.Status(status));
  splitter.PrintFields(measure);
  ga_flag(splitter.ResSurfaces() == previous);
  sh_split_surface_result(splitter.ResSurfaces(), input);
  sh_split_surface_result(previous, input);
}
void sh_split_surface_values(ShSplitSurfaceProbe &splitter, GaReader &reader,
                             bool replace) {
  bool u = reader.flag();
  int count = reader.integer();
  auto values = replace
                    ? (u ? splitter.USplitValues() : splitter.VSplitValues())
                    : occ::handle<NCollection_HSequence<double>>(
                          new NCollection_HSequence<double>);
  if (replace)
    values->Clear();
  for (int i = 0; i < count; i++)
    values->Append(reader.real());
  if (!replace) {
    GA_TRY(if (u) splitter.SetUSplitValues(values);
           else splitter.SetVSplitValues(values))
  }
}
void sh_split_surface_operation(ShSplitSurfaceProbe &splitter,
                                const occ::handle<Geom_Surface> &input,
                                GaReader &reader) {
  int op = reader.integer();
  if (op == 0 || op == 3) {
    sh_split_surface_values(splitter, reader, op == 3);
    return;
  }
  if (op == 4) {
    splitter.SetRawStatus(reader.integer());
    return;
  }
  if (op == 6) {
    GA_TRY(splitter.Init(input)) return;
  }
  bool segment = reader.flag();
  GA_TRY(if (op == 1) splitter.Build(segment);
         else if (op == 2) splitter.Perform(segment);
         else splitter.Compute(segment))
}
void sh_split_surface_request(GaReader &reader) {
  auto input = sh_analysis_surface_geometry(reader);
  ShSplitSurfaceProbe splitter;
  bool natural = reader.flag();
  double u1 = reader.real(), u2 = reader.real(), v1 = reader.real(),
         v2 = reader.real(), area = reader.real();
  if (natural)
    splitter.Init(input);
  else
    splitter.Init(input, u1, u2, v1, v2, area);
  bool measure = !natural && area != 0.;
  sh_split_surface_state(splitter, input, {}, measure);
  int count = reader.integer();
  for (int i = 0; i < count; i++) {
    auto previous = splitter.ResSurfaces();
    sh_split_surface_operation(splitter, input, reader);
    sh_split_surface_state(splitter, input, previous, measure);
  }
  shc_surface(input);
}
void sh_split_surface_case(std::istream &input) {
  GaReader reader{input};
  GA_TRY(sh_split_surface_request(reader)) ga_end();
}
