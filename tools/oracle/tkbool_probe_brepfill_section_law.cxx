// BRepFill_SectionLaw probe operations (BRepFill_ShapeLaw, BRepFill_NSections); the Rust
// twin is crates/tkbool/examples/bl_probe/brep_fill_section_law.rs.
//
// The sections are shapes of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx).
#include <BRepFill_NSections.hxx>
#include <BRepFill_ShapeLaw.hxx>
#include <GeomFill_SectionLaw.hxx>
#include <Law_Linear.hxx>
// The queries of a built law at `param`: NbLaw; Vertex for 1..n+1; VertexTol and
// Continuity (angular 1e-2) for 0..n; the section shape, poles and weights of
// ConcatenedLaw at `param`.
void bl_built_law_queries(const occ::handle<BRepFill_SectionLaw>& law,double param){
 int n=0;
 GA_TRY(n=law->NbLaw();ga_int(n))
 for(int i=1;i<=n+1;++i)GA_TRY(pr_shape(law->Vertex(i,param)))
 for(int i=0;i<=n;++i){GA_TRY(ga_real(law->VertexTol(i,param))) GA_TRY(ga_int(law->Continuity(i,1.e-2)))}
 GA_TRY(occ::handle<GeomFill_SectionLaw> c=law->ConcatenedLaw();tp_type(c);
  if(!c.IsNull()){int np,nk,dg;c->SectionShape(np,nk,dg);ga_int(np);ga_int(nk);ga_int(dg);
   NCollection_Array1<gp_Pnt> poles(1,np);NCollection_Array1<double> w(1,np);c->D0(param,poles,w);
   for(int i=1;i<=np;++i){ga_pnt(poles(i));ga_real(w(i));}})}
// The queries of any law: the flags, the built queries, D0 at `param` and IndexOfEdge of
// every edge of the program.
void bl_section_law_queries(const occ::handle<BRepFill_SectionLaw>& law,const TpShapes& t,bool build,double param){
 ga_flag(law->IsVertex());ga_flag(law->IsConstant());ga_flag(law->IsUClosed());ga_flag(law->IsVClosed());
 ga_flag(law->IsDone());
 if(build&&law->IsDone())bl_built_law_queries(law,param);
 if(law->IsDone()){TopoDS_Shape s;GA_TRY(law->D0(param,s);pr_shape(s))}
 for(const TopoDS_Shape& s:t.shapes){
  if(s.IsNull()||s.ShapeType()!=TopAbs_EDGE)continue;
  GA_TRY(ga_int(law->IndexOfEdge(s)))}}
// `program index build haslaw [pdeb valdeb pfin valfin] param`: a BRepFill_ShapeLaw of the
// vertex or wire `index` (scaled by a Law_Linear when `haslaw`), its queries, then for a
// wire Init(W) and CurrentEdge until the end (table index, -1 for a null edge).
void bl_bf_shape_law_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape& s=t.shapes.at(r.integer());bool build=r.flag();bool haslaw=r.flag();
 occ::handle<Law_Linear> lin;
 if(haslaw){lin=new Law_Linear();double a=r.real(),b=r.real(),c=r.real();lin->Set(a,b,c,r.real());}
 double param=r.real();
 occ::handle<BRepFill_ShapeLaw> law;
 try{
  if(s.ShapeType()==TopAbs_VERTEX)law=new BRepFill_ShapeLaw(TopoDS::Vertex(s),build);
  else if(haslaw)law=new BRepFill_ShapeLaw(TopoDS::Wire(s),lin,build);
  else law=new BRepFill_ShapeLaw(TopoDS::Wire(s),build);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 bl_section_law_queries(law,t,build,param);
 if(s.ShapeType()==TopAbs_WIRE){
  occ::handle<BRepFill_SectionLaw> base=law;base->Init(TopoDS::Wire(s));
  for(int k=0;k<=t.shapes.size();++k){TopoDS_Edge e;GA_TRY(e=law->CurrentEdge();ga_int(tp_shape_index(t,e)))
   if(e.IsNull())break;}}
 ga_end();}
// `program n (wire)... build param`: a BRepFill_NSections of the wires and its queries.
void bl_bf_nsections_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 NCollection_Sequence<TopoDS_Shape> sections;int n=r.integer();
 for(int k=0;k<n;++k)sections.Append(t.shapes.at(r.integer()));
 bool build=r.flag();double param=r.real();
 occ::handle<BRepFill_NSections> law;
 try{law=new BRepFill_NSections(sections,build);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 bl_section_law_queries(law,t,build,param);
 ga_end();}
