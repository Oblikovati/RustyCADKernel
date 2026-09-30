// BRepTopAdaptor probe operations over faces read by tp_face (tktopalgo_probe_face.cxx);
// the Rust twin is crates/tktopalgo/examples/tp_probe/brep_top_adaptor.rs. Edges are
// printed as their index among the edges of the FORWARD face (tp_edge_index), states as
// TopAbs_State integers.
#include <Adaptor3d_HVertex.hxx>
#include <BRepAdaptor_Curve2d.hxx>
#include <BRepAdaptor_Surface.hxx>
#include <BRepTopAdaptor_FClass2d.hxx>
#include <BRepTopAdaptor_HVertex.hxx>
#include <BRepTopAdaptor_Tool.hxx>
#include <BRepTopAdaptor_TopolTool.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
// `face tol ontol n (u v)...`: BRepTopAdaptor_FClass2d(face, tol); PerformInfinitePoint,
// then per point Perform (reframed or not) and TestOnRestriction(ontol) (reframed or not).
void tp_bta_fclass2d_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);double tol=r.real(),ontol=r.real();
  BRepTopAdaptor_FClass2d c(f.face,tol);GA_TRY(ga_int(c.PerformInfinitePoint()))
  int n=r.integer();
  for(int i=0;i<n;++i){gp_Pnt2d p=r.pnt2d();
   GA_TRY(ga_int(c.Perform(p)))GA_TRY(ga_int(c.Perform(p,false)))
   GA_TRY(ga_int(c.TestOnRestriction(p,ontol)))GA_TRY(ga_int(c.TestOnRestriction(p,ontol,false)))})
 ga_end();}
// `face tol u v`: Perform and TestOnRestriction before and after Destroy.
void tp_bta_destroy_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);double tol=r.real();gp_Pnt2d p=r.pnt2d();
  BRepTopAdaptor_FClass2d c(f.face,tol);ga_int(c.Perform(p));ga_int(c.TestOnRestriction(p,tol));
  c.Destroy();ga_int(c.Perform(p));ga_int(c.TestOnRestriction(p,tol));ga_int(c.PerformInfinitePoint()))
 ga_end();}
// The vertices of the arc `arc` through the tool: orientation, 3d tolerance and point,
// then the HVertex value, parameter, resolution and sameness with the first vertex, and
// the vertex past the last one.
void tp_bta_vertices(const occ::handle<BRepTopAdaptor_TopolTool>& t,const occ::handle<Adaptor2d_Curve2d>& arc){
 t->Initialize(arc);occ::handle<Adaptor3d_HVertex> first;
 for(t->InitVertexIterator();t->MoreVertex();t->NextVertex()){
  occ::handle<Adaptor3d_HVertex> v=t->Vertex();if(first.IsNull())first=v;
  ga_int(t->Orientation(v));GA_TRY(ga_real(t->Tol3d(v)))GA_TRY(ga_pnt(t->Pnt(v)))
  ga_pnt2d(v->Value());GA_TRY(ga_real(v->Parameter(arc)))GA_TRY(ga_real(v->Resolution(arc)))
  ga_flag(v->IsSame(first));}
 GA_TRY(t->Vertex())}
// `face restr`: the arcs of BRepTopAdaptor_TopolTool(BRepAdaptor_Surface(face, restr))
// with their vertices, then the samples and the domain.
void tp_bta_topol_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);bool restr=r.flag();
  occ::handle<BRepAdaptor_Surface> s=new BRepAdaptor_Surface(f.face,restr);
  occ::handle<BRepTopAdaptor_TopolTool> t=new BRepTopAdaptor_TopolTool(s);
  ga_flag(t->Has3d());ga_flag(t->More());
  for(t->Init();t->More();t->Next()){
   occ::handle<Adaptor2d_Curve2d> arc=t->Value();
   ga_int(tp_edge_index(f.face,*(TopoDS_Edge*)t->Edge()));ga_int(t->Orientation(arc));
   GA_TRY(ga_real(t->Tol3d(arc)))ga_real(arc->FirstParameter());ga_real(arc->LastParameter());
   GA_TRY(tp_bta_vertices(t,arc))}
  GA_TRY(ga_flag(t->DomainIsInfinite()))
  GA_TRY(int nu=t->NbSamplesU();int nv=t->NbSamplesV();ga_int(nu);ga_int(nv);ga_int(t->NbSamples());
   const int picks[]={0,1,nu,nu*nv/2,nu*nv-1,nu*nv};
   for(int i:picks){gp_Pnt2d p2;gp_Pnt p3;GA_TRY(t->SamplePoint(i,p2,p3);ga_pnt2d(p2);ga_pnt(p3))}))
 ga_end();}
// `face restr tol n (u v)...`: Classify and IsThePointOn of a tool (reframed or not).
void tp_bta_classify_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);bool restr=r.flag();double tol=r.real();
  occ::handle<BRepAdaptor_Surface> s=new BRepAdaptor_Surface(f.face,restr);
  occ::handle<BRepTopAdaptor_TopolTool> t=new BRepTopAdaptor_TopolTool(s);int n=r.integer();
  for(int i=0;i<n;++i){gp_Pnt2d p=r.pnt2d();
   GA_TRY(ga_int(t->Classify(p,tol)))GA_TRY(ga_int(t->Classify(p,tol,false)))
   GA_TRY(ga_flag(t->IsThePointOn(p,tol)))GA_TRY(ga_flag(t->IsThePointOn(p,tol,false)))}
  t->Destroy();GA_TRY(ga_int(t->Classify(gp_Pnt2d(0.,0.),tol))))
 ga_end();}
// `face`: BRepTopAdaptor_Tool on the face (and on its adaptor), the surface and samples.
void tp_bta_tool_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);
  BRepTopAdaptor_Tool a;ga_flag(a.GetSurface().IsNull());ga_flag(a.GetTopolTool().IsNull());
  BRepTopAdaptor_Tool b(f.face,1e-9);ga_int(b.GetSurface()->GetType());
  ga_real(b.GetSurface()->FirstUParameter());ga_real(b.GetSurface()->LastVParameter());
  GA_TRY(ga_int(b.GetTopolTool()->NbSamples()))
  occ::handle<Adaptor3d_Surface> s=new BRepAdaptor_Surface(f.face,false);
  BRepTopAdaptor_Tool c(s,1e-9);ga_flag(c.GetSurface()==s);GA_TRY(ga_flag(c.GetTopolTool()->DomainIsInfinite()))
  a.Init(f.face,1e-9);ga_int(a.GetSurface()->GetType());a.Init(s,0.);ga_flag(a.GetSurface()==s);
  a.SetTopolTool(c.GetTopolTool());ga_flag(a.GetTopolTool()==c.GetTopolTool());a.Destroy())
 ga_end();}
// `face`: the errors of the tool: Initialize(), a GeomAdaptor surface, a Geom2dAdaptor
// arc, a base vertex, a default tool, and the vertex past the last one.
void tp_bta_errors_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);
  occ::handle<BRepTopAdaptor_TopolTool> t=new BRepTopAdaptor_TopolTool();
  ga_int(t->Classify(gp_Pnt2d(0.,0.),1e-7));GA_TRY(t->Initialize())
  occ::handle<Adaptor3d_Surface> gs=new GeomAdaptor_Surface(BRep_Tool::Surface(f.face));
  GA_TRY(t->Initialize(gs))GA_TRY(BRepTopAdaptor_TopolTool u(gs))
  occ::handle<Adaptor2d_Curve2d> gc=new Geom2dAdaptor_Curve();
  GA_TRY(t->Initialize(gc))GA_TRY(ga_real(t->Tol3d(gc)))
  occ::handle<Adaptor3d_HVertex> hv=new Adaptor3d_HVertex(gp_Pnt2d(1.,2.),TopAbs_REVERSED,0.5);
  ga_int(t->Orientation(hv));GA_TRY(ga_real(t->Tol3d(hv)))GA_TRY(ga_pnt(t->Pnt(hv)))
  occ::handle<BRepTopAdaptor_HVertex> nv=new BRepTopAdaptor_HVertex(TopoDS_Vertex(),nullptr);
  GA_TRY(ga_real(t->Tol3d(nv)))GA_TRY(ga_pnt(t->Pnt(nv)))ga_pnt2d(nv->Value());ga_int(nv->Orientation());
  occ::handle<BRepAdaptor_Surface> s=new BRepAdaptor_Surface(f.face);t->Initialize(s);
  t->Init();if(t->More()){t->Initialize(t->Value());t->InitVertexIterator();
   while(t->MoreVertex())t->NextVertex();GA_TRY(t->Vertex())})
 ga_end();}
