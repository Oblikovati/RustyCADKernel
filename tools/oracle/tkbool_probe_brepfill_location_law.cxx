// BRepFill_LocationLaw probe operations (BRepFill_Edge3DLaw, BRepFill_DraftLaw,
// BRepFill_EdgeOnSurfLaw); the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_location_law.rs.
//
// The path, the support and the section are shapes of a shape program
// (tktopalgo_probe_shapes.cxx); shapes print through pr_shape (tkprim_probe_shape.cxx).
#include <BRepAdaptor_CompCurve.hxx>
#include <BRepFill_ACRLaw.hxx>
#include <BRepFill_DraftLaw.hxx>
#include <GeomFill_GuideTrihedronAC.hxx>
#include <GeomFill_LocationGuide.hxx>
#include <BRepFill_Edge3DLaw.hxx>
#include <BRepFill_EdgeOnSurfLaw.hxx>
#include <GeomFill_ConstantBiNormal.hxx>
#include <GeomFill_CorrectedFrenet.hxx>
#include <GeomFill_CurveAndTrihedron.hxx>
#include <GeomFill_Frenet.hxx>
#include <GeomFill_LocationDraft.hxx>
// A vertex: its point and tolerance (-1 for a null vertex).
void bl_vertex(const TopoDS_Vertex& v){
 if(v.IsNull()){ga_int(-1);return;}
 ga_pnt(BRep_Tool::Pnt(v));ga_real(BRep_Tool::Tolerance(v));}
// The queries of a location law with `n` laws: the abscissa of the middle of the last
// edge (before the lengths are known), GetStatus, IsClosed, IsG1 at 0..n, NbHoles(1e-7)
// and Holes, Vertex at 1..n+1, CurvilinearBounds at 1..n, Parameter at five abscissas,
// PerformVertex of the first section vertex at 0..n for locations -1, 0, 1, D0 of the
// section at a quarter of the path, then the same D0 after TransformInG0Law, after
// TransformInCompatibleLaw(0.1) and after DeleteTransform.
void bl_location_law_queries(const occ::handle<BRepFill_LocationLaw>& law,const TopoDS_Shape& section){
 int n=0;GA_TRY(n=law->NbLaw();ga_int(n))
 if(n>0)GA_TRY(double f,l;law->Law(n)->GetDomain(f,l);ga_real(law->Abscissa(n,0.5*(f+l))))
 GA_TRY(ga_int(law->GetStatus()))GA_TRY(ga_flag(law->IsClosed()))
 for(int i=0;i<=n;++i)GA_TRY(ga_int(law->IsG1(i)))
 GA_TRY(int h=law->NbHoles(1.e-7);ga_int(h);if(h>0){NCollection_Array1<int> d(1,h);law->Holes(d);for(int k=1;k<=h;++k)ga_int(d(k));})
 for(int i=1;i<=n+1;++i)GA_TRY(bl_vertex(law->Vertex(i)))
 double total=0.;
 for(int i=1;i<=n;++i)GA_TRY(double f,l;law->CurvilinearBounds(i,f,l);ga_real(f);ga_real(l);total=l;)
 for(double a:{0.,0.3*total,0.5*total,total,1.1*total+1.}){int ind=0;double u=0.;GA_TRY(law->Parameter(a,ind,u);ga_int(ind);if(ind!=0)ga_real(u))}
 TopoDS_Vertex sv;for(TopExp_Explorer ex(section,TopAbs_VERTEX);ex.More();ex.Next()){sv=TopoDS::Vertex(ex.Current());break;}
 if(!sv.IsNull())for(int i=0;i<=n;++i)for(int loc=-1;loc<=1;++loc){TopoDS_Vertex o;GA_TRY(law->PerformVertex(i,sv,1.e-4,o,loc);bl_vertex(o))}
 GA_TRY(TopoDS_Shape s=section;law->D0(0.25*total,s);pr_shape(s))
 GA_TRY(law->TransformInG0Law();TopoDS_Shape s=section;law->D0(0.25*total,s);pr_shape(s))
 GA_TRY(law->TransformInCompatibleLaw(0.1);TopoDS_Shape s=section;law->D0(0.25*total,s);pr_shape(s))
 GA_TRY(law->DeleteTransform();TopoDS_Shape s=section;law->D0(0.25*total,s);pr_shape(s))}
// `program path section kind ...`: kind 0 `trihedron` (Edge3DLaw on a
// GeomFill_CurveAndTrihedron: 0 corrected Frenet, 1 Frenet, 2 constant binormal +Z),
// 1 `D angle` (DraftLaw on a GeomFill_LocationDraft, then CleanLaw(0.1)), 2 `support`
// (EdgeOnSurfLaw on a shape, printing HasResult), 3 `guide` (ACRLaw on a GeomFill_LocationGuide
// of a GeomFill_GuideTrihedronAC along the BRepAdaptor_CompCurve of a guide wire).
void bl_bf_location_law_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire path=TopoDS::Wire(t.shapes.at(r.integer()));TopoDS_Shape section=t.shapes.at(r.integer());
 int kind=r.integer();occ::handle<BRepFill_LocationLaw> law;
 try{
  if(kind==0){int k=r.integer();occ::handle<GeomFill_TrihedronLaw> tri;
   if(k==0)tri=new GeomFill_CorrectedFrenet();else if(k==1)tri=new GeomFill_Frenet();else tri=new GeomFill_ConstantBiNormal(gp_Dir(0,0,1));
   law=new BRepFill_Edge3DLaw(path,new GeomFill_CurveAndTrihedron(tri));}
  else if(kind==1){gp_Dir d=r.dir();double a=r.real();
   occ::handle<BRepFill_DraftLaw> dl=new BRepFill_DraftLaw(path,new GeomFill_LocationDraft(d,a));dl->CleanLaw(0.1);law=dl;}
  else if(kind==3){occ::handle<Adaptor3d_Curve> guide=new BRepAdaptor_CompCurve(TopoDS::Wire(t.shapes.at(r.integer())));
   occ::handle<GeomFill_LocationGuide> lg=new GeomFill_LocationGuide(new GeomFill_GuideTrihedronAC(guide));
   law=new BRepFill_ACRLaw(path,lg);}
  else{occ::handle<BRepFill_EdgeOnSurfLaw> sl=new BRepFill_EdgeOnSurfLaw(path,t.shapes.at(r.integer()));
   ga_flag(sl->HasResult());law=sl;if(!sl->HasResult()){ga_end();return;}}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 bl_location_law_queries(law,section);
 ga_end();}
