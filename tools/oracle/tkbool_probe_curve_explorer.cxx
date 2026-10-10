// Behavioral oracle for TopOpeBRepDS_CurveExplorer at pinned OCCT 3d097a0.
// Input: count gap keep-mask only-kept query-index. Gaps follow the first curve.
#include <TopOpeBRepDS_CurveExplorer.hxx>
#include <TopOpeBRepDS_DataStructure.hxx>
#include <TopOpeBRepDS_Curve.hxx>

static void bl_explorer_curve(const TopOpeBRepDS_Curve& curve) {
  ga_int(curve.DSIndex());
  ga_flag(curve.Keep());
  ga_real(curve.Tolerance());
  ga_flag(curve.Curve().IsNull());
}

static void bl_explorer_walk(TopOpeBRepDS_CurveExplorer& explorer) {
  for (; explorer.More(); explorer.Next()) {
    ga_int(explorer.Index());
    bl_explorer_curve(explorer.Curve());
  }
  ga_int(explorer.Index());
  bl_explorer_curve(explorer.Curve());
  ga_int(explorer.NbCurve());
  ga_flag(explorer.More());
  ga_int(explorer.Index());
}

static void bl_explorer_source(TopOpeBRepDS_DataStructure& curves, int count, int gap, int mask) {
  for (int slot = 0; slot < count; ++slot) {
    if (slot == 1) curves.ChangeNbCurves(curves.NbCurves() + gap);
    TopOpeBRepDS_Curve curve;
    curve.Tolerance(0.25 * (slot + 1));
    curve.ChangeKeep((mask & (1 << slot)) != 0);
    curves.AddCurve(curve);
  }
  curves.ChangeCurve(-1).Tolerance(12.0);
}

static void bl_curve_explorer_case(std::istream& input) {
  GaReader reader{input};
  const int count = reader.integer(), gap = reader.integer(), mask = reader.integer();
  const bool onlyKept = reader.integer() != 0;
  const int query = reader.integer();
  TopOpeBRepDS_DataStructure curves;
  bl_explorer_source(curves, count, gap, mask);
  TopOpeBRepDS_CurveExplorer explorer(curves, onlyKept);
  ga_flag(explorer.IsCurve(query)); ga_flag(explorer.IsCurveKeep(query));
  bl_explorer_curve(explorer.Curve(query)); bl_explorer_walk(explorer);
  explorer.Init(curves, !onlyKept); bl_explorer_walk(explorer); ga_end();
}

static void bl_curve_explorer_mutation_case(std::istream& input) {
  GaReader reader{input};
  const int gap = reader.integer();
  TopOpeBRepDS_DataStructure curves;
  bl_explorer_source(curves, 2, gap, 1);
  TopOpeBRepDS_CurveExplorer explorer(curves, true);
  curves.ChangeKeepCurve(2 + gap, true);
  TopOpeBRepDS_Curve added;
  const int index = curves.AddCurve(added);
  ga_flag(explorer.IsCurve(index)); bl_explorer_curve(explorer.Curve(index));
  bl_explorer_walk(explorer);
  explorer.Init(curves, true); curves.Init(); curves.ChangeCurve(-1).Tolerance(12.0);
  ga_flag(explorer.More()); bl_explorer_curve(explorer.Curve());
  bl_explorer_curve(explorer.Curve(1)); explorer.Next();
  ga_flag(explorer.More()); ga_int(explorer.Index()); ga_end();
}
