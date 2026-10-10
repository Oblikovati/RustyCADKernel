// Address-independent BRepFill_Evolved digest. Native Quilt and DataMap traversal
// changes between executions; retain exact sampled geometry and oriented incidence
// while sorting siblings and p-curve representations. A free shell may reverse as
// a whole depending on its seed face. In a solid, consume the shell orientation
// into its face incidences, retaining the physically significant total orientation.
#include <algorithm>
using BlEvolvedKey = std::vector<uint64_t>;
void bl_key_point(BlEvolvedKey& key, const gp_Pnt& p) {
  key.insert(key.end(), {ga_bits(p.X()), ga_bits(p.Y()), ga_bits(p.Z())});
}
void bl_key_type(BlEvolvedKey& key, const occ::handle<Standard_Transient>& value) {
  std::string name = value.IsNull() ? "null" : value->DynamicType()->Name();
  key.push_back(name.size()); for (unsigned char c : name) key.push_back(c);
}
void bl_key_children(BlEvolvedKey& key, std::vector<BlEvolvedKey> children) {
  std::sort(children.begin(), children.end()); key.push_back(children.size());
  for (const auto& child : children) { key.push_back(child.size()); key.insert(key.end(), child.begin(), child.end()); }
}
BlEvolvedKey bl_key_pcurves(const TopoDS_Edge& edge) {
  std::vector<BlEvolvedKey> rows;
  for (int i = 1;; ++i) {
    occ::handle<Geom2d_Curve> curve; occ::handle<Geom_Surface> surface;
    TopLoc_Location location; double first=0., last=0.;
    BRep_Tool::CurveOnSurface(edge, curve, surface, location, first, last, i);
    if (curve.IsNull()) break;
    BlEvolvedKey row{uint64_t(location.IsIdentity()), ga_bits(first), ga_bits(last)};
    bl_key_type(row, surface); bl_key_type(row, curve);
    for (double u : {first, last, 0.5*(first+last)}) {
      gp_Pnt2d p=curve->Value(u); row.insert(row.end(), {ga_bits(p.X()), ga_bits(p.Y())});
    }
    rows.push_back(row);
  }
  BlEvolvedKey key; bl_key_children(key, rows); return key;
}
void bl_key_edge(BlEvolvedKey& key, const TopoDS_Edge& edge) {
  key.insert(key.end(), {ga_bits(BRep_Tool::Tolerance(edge)), uint64_t(BRep_Tool::SameParameter(edge)),
                        uint64_t(BRep_Tool::SameRange(edge)), uint64_t(BRep_Tool::Degenerated(edge))});
  double first=0., last=0.; auto curve=BRep_Tool::Curve(edge, first, last);
  bl_key_type(key, curve);
  if (!curve.IsNull()) {
    key.insert(key.end(), {ga_bits(first), ga_bits(last)});
    for (double u : {first, last, 0.5*(first+last)}) bl_key_point(key, curve->Value(u));
  }
  auto pcurves=bl_key_pcurves(edge); key.insert(key.end(), pcurves.begin(), pcurves.end());
}
void bl_key_face(BlEvolvedKey& key, const TopoDS_Face& face) {
  key.insert(key.end(), {ga_bits(BRep_Tool::Tolerance(face)), uint64_t(BRep_Tool::NaturalRestriction(face))});
  auto surface=BRep_Tool::Surface(face); bl_key_type(key, surface);
  if (surface.IsNull()) return;
  bl_key_point(key, surface->Value(0.25, 0.5));
  gp_Pnt point; gp_Vec du,dv; surface->D1(0.3,0.7,point,du,dv);
  bl_key_point(key, point); bl_key_point(key, gp_Pnt(du.XYZ())); bl_key_point(key, gp_Pnt(dv.XYZ()));
}
BlEvolvedKey bl_evolved_key(const TopoDS_Shape& shape, bool inSolid=false) {
  if (shape.IsNull()) return {UINT64_MAX};
  BlEvolvedKey key{uint64_t(shape.ShapeType()), uint64_t(shape.Closed()), uint64_t(shape.Location().IsIdentity())};
  bool solid=inSolid || shape.ShapeType()==TopAbs_SOLID;
  std::vector<BlEvolvedKey> children, reversed;
  for (TopoDS_Iterator it(shape, solid && shape.ShapeType()==TopAbs_SHELL, true); it.More(); it.Next()) {
    auto child=bl_evolved_key(it.Value(),solid);
    if (shape.ShapeType()==TopAbs_EDGE) child.push_back(ga_bits(BRep_Tool::Parameter(TopoDS::Vertex(it.Value()),TopoDS::Edge(shape))));
    child.insert(child.begin(),solid && it.Value().ShapeType()==TopAbs_SHELL ? 0 : uint64_t(it.Value().Orientation())); children.push_back(child);
    child[0]=uint64_t(TopAbs::Reverse(it.Value().Orientation())); reversed.push_back(child);
  }
  BlEvolvedKey incidence; bl_key_children(incidence,children);
  if (!solid && shape.ShapeType()==TopAbs_SHELL) {
    BlEvolvedKey reverse; bl_key_children(reverse,reversed); incidence=std::min(incidence,reverse);
  }
  key.insert(key.end(),incidence.begin(),incidence.end());
  if (shape.ShapeType()==TopAbs_VERTEX) { bl_key_point(key,BRep_Tool::Pnt(TopoDS::Vertex(shape))); key.push_back(ga_bits(BRep_Tool::Tolerance(TopoDS::Vertex(shape)))); }
  if (shape.ShapeType()==TopAbs_EDGE) bl_key_edge(key,TopoDS::Edge(shape));
  if (shape.ShapeType()==TopAbs_FACE) bl_key_face(key,TopoDS::Face(shape));
  return key;
}
void bl_evolved_digest(const TopoDS_Shape& shape) {
  auto key=bl_evolved_key(shape);
  ga_int(key.size()); for (auto value:key) std::cout<<value<<' ';
  for (int kind=0;kind<8;++kind) {
    NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> shapes;
    if (!shape.IsNull()) TopExp::MapShapes(shape,static_cast<TopAbs_ShapeEnum>(kind),shapes);
    ga_int(shapes.Extent());
  }
}
