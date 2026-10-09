// BRepFill_Section and BRepFill_SectionPlacement probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_section.rs.
//
// The shapes are those of a shape program (tktopalgo_probe_shapes.cxx).
#include <BRepFill_Section.hxx>
#include <BRepFill_SectionPlacement.hxx>
// `program profile vertex contact correction` (vertex -1: none): the section, its flags,
// its wire and the ModifiedShape of every shape of the program (index in the wire map).
void bl_bf_section_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape& profile=t.shapes.at(r.integer());int vi=r.integer();
 TopoDS_Vertex v;if(vi>=0)v=TopoDS::Vertex(t.shapes.at(vi));
 bool contact=r.flag(),correction=r.flag();
 BRepFill_Section section;
 try{section=BRepFill_Section(profile,v,contact,correction);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 section.Set(true);
 ga_flag(section.IsLaw());ga_flag(section.IsPunctual());ga_flag(section.WithContact());ga_flag(section.WithCorrection());
 ga_int(tp_shape_index(t,section.OriginalShape()));ga_int(tp_shape_index(t,section.Vertex()));
 GA_TRY(pr_shape(section.Wire()))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> m;TopExp::MapShapes(section.Wire(),m);
 for(const TopoDS_Shape& s:t.shapes)GA_TRY(bl_indexed(m,section.ModifiedShape(s)))
 ga_end();}
// The matrix and translation of a transformation.
void bl_trsf(const gp_Trsf& trsf){
 for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)ga_real(trsf.Value(i,j));}
// `program path section vertex contact correction`: an Edge3DLaw of the path on a
// corrected Frenet trihedron, the placement of the section (located by the path vertex
// unless -1), its transformation and AbscissaOnPath.
void bl_bf_section_placement_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire path=TopoDS::Wire(t.shapes.at(r.integer()));const TopoDS_Shape& section=t.shapes.at(r.integer());
 int vi=r.integer();TopoDS_Shape v;if(vi>=0)v=t.shapes.at(vi);
 bool contact=r.flag(),correction=r.flag();
 try{
  occ::handle<BRepFill_LocationLaw> law=new BRepFill_Edge3DLaw(path,new GeomFill_CurveAndTrihedron(new GeomFill_CorrectedFrenet()));
  BRepFill_SectionPlacement place(law,section,v,contact,correction);
  bl_trsf(place.Transformation());
  GA_TRY(ga_real(place.AbscissaOnPath()))}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
