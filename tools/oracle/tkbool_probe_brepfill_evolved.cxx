// Independent OCCT oracle for the recovered BRepFill_Evolved translation.
// `program spine profile face solid constructor join origin normal xdirection`.
#include <BRepFill_Evolved.hxx>
#include <BRepLib_MakeFace.hxx>

void bl_evolved_history(const BRepFill_Evolved& evolved, const TopoDS_Shape& spine,
                        const TopoDS_Shape& profile) {
  NCollection_IndexedMap<TopoDS_Shape, TopTools_ShapeMapHasher> spines, profiles;
  TopExp::MapShapes(spine, spines); TopExp::MapShapes(profile, profiles);
  for (int i = 1; i <= spines.Extent(); ++i)
    for (int j = 1; j <= profiles.Extent(); ++j) {
      const auto& generated = evolved.GeneratedShapes(spines(i), profiles(j));
      ga_int(generated.Size());
      for (const auto& shape : generated) { ga_int(shape.ShapeType()); ga_int(shape.Orientation()); }
    }
}

void bl_evolved_result(const BRepFill_Evolved& evolved) {
  ga_flag(evolved.IsDone());
  GA_TRY(bl_evolved_digest(evolved.Shape()))
  GA_TRY(bl_evolved_digest(evolved.Top()))
  GA_TRY(bl_evolved_digest(evolved.Bottom()))
}

void bl_bf_evolved_empty_case(std::istream&) {
  BRepFill_Evolved evolved;
  bl_evolved_result(evolved);
  ga_int(evolved.GeneratedShapes(TopoDS_Shape(), TopoDS_Shape()).Size());
  ga_end();
}

void bl_bf_evolved_case(std::istream& input) {
  GaReader r{input};
  try {
    TpShapes table = tp_shapes(r);
    const auto spine = TopoDS::Wire(table.shapes.at(r.integer()));
    const auto profile = TopoDS::Wire(table.shapes.at(r.integer()));
    const bool face = r.flag(), solid = r.flag(), constructor = r.flag();
    const auto join = static_cast<GeomAbs_JoinType>(r.integer());
    const gp_Pnt origin=r.pnt(); const gp_Dir normal=r.dir(), xdir=r.dir();
    const gp_Ax3 axes(origin,normal,xdir);
    BRepFill_Evolved evolved;
    if (face) {
      const auto spineFace = BRepLib_MakeFace(spine, true).Face();
      if (constructor) evolved = BRepFill_Evolved(spineFace, profile, axes, join, solid);
      else evolved.Perform(spineFace, profile, axes, join, solid);
    } else {
      if (constructor) evolved = BRepFill_Evolved(spine, profile, axes, join, solid);
      else evolved.Perform(spine, profile, axes, join, solid);
    }
    bl_evolved_result(evolved); ga_int(evolved.JoinType());
    bl_evolved_history(evolved, spine, profile);
  } catch (const Standard_Failure& error) { std::cout << error.ExceptionType() << ' '; }
  ga_end();
}
