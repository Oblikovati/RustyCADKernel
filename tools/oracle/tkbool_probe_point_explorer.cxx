// PointExplorer oracle at OCCT 3d097a0. Empty points have an uninitialized
// native tolerance, so only bound points contribute a tolerance token.
#include <TopOpeBRepDS_PointExplorer.hxx>

static void bl_explorer_point(const TopOpeBRepDS_Point& point) {
  ga_pnt(point.Point());
  ga_flag(point.Keep());
}

static void bl_point_source(TopOpeBRepDS_DataStructure& source, int count, int mask) {
  for (int slot = 0; slot < count; ++slot) {
    TopOpeBRepDS_Point point(gp_Pnt(slot + 1, 2, 3), 0.25 * (slot + 1));
    point.ChangeKeep((mask & (1 << slot)) != 0);
    source.AddPoint(point);
  }
}

static void bl_point_walk(TopOpeBRepDS_PointExplorer& explorer) {
  for (; explorer.More(); explorer.Next()) {
    ga_int(explorer.Index()); bl_explorer_point(explorer.Point());
    ga_real(explorer.Point().Tolerance());
  }
  ga_int(explorer.Index()); bl_explorer_point(explorer.Point());
  ga_int(explorer.NbPoint()); ga_flag(explorer.More()); ga_int(explorer.Index());
}

static void bl_point_explorer_case(std::istream& input) {
  GaReader reader{input};
  const int count = reader.integer(), mask = reader.integer();
  const bool kept = reader.integer() != 0;
  const int query = reader.integer();
  TopOpeBRepDS_DataStructure source;
  bl_point_source(source, count, mask);
  TopOpeBRepDS_PointExplorer explorer;
  ga_flag(explorer.More()); ga_int(explorer.Index()); bl_explorer_point(explorer.Point());
  explorer.Next(); ga_int(explorer.Index());
  explorer.Init(source, kept);
  ga_flag(explorer.IsPoint(query)); ga_flag(explorer.IsPointKeep(query));
  bl_explorer_point(explorer.Point(query)); bl_point_walk(explorer);
  explorer.Init(source, !kept); bl_point_walk(explorer); ga_end();
}

static void bl_point_explorer_mutation_case(std::istream&) {
  TopOpeBRepDS_DataStructure source;
  bl_point_source(source, 2, 1);
  TopOpeBRepDS_PointExplorer explorer(source, true);
  source.ChangeKeepPoint(2, true);
  source.AddPoint(TopOpeBRepDS_Point(gp_Pnt(3, 2, 3), 0.75));
  ga_flag(explorer.IsPoint(3)); bl_explorer_point(explorer.Point(3));
  bl_point_walk(explorer);
  explorer.Init(source, true); source.Init();
  ga_flag(explorer.More()); GA_TRY(bl_explorer_point(explorer.Point()))
  bl_explorer_point(explorer.Point(1)); explorer.Next();
  ga_flag(explorer.More()); ga_int(explorer.Index()); ga_end();
}
