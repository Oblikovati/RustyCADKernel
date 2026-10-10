// BRepFill_PipeShell probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_pipe_shell.rs.
//
// The spine, the profiles and the supports are shapes of a shape program
// (tktopalgo_probe_shapes.cxx); shapes print through pr_shape (tkprim_probe_shape.cxx)
// and generated shapes through bl_list (tkbool_probe_brepfill_generator.cxx).
#include <BRepFill_PipeShell.hxx>
#include <Law_Linear.hxx>
// The location mode of a request: 0 none (the corrected Frenet default), 1 Set(true),
// 2 Set(false), 3 SetDiscrete, 4 `origin dir xdir` Set(gp_Ax2), 5 `dir` Set(BiNormal),
// 6 `support` Set(SpineSupport) (prints its result), 7 `guide curvilinear contact`
// Set(AuxiliarySpine, CurvilinearEquivalence, KeepContact).
void bl_pipe_mode(GaReader& r,const TpShapes& t,BRepFill_PipeShell& pipe){
 int mode=r.integer();
 if(mode==1)pipe.Set(true);
 else if(mode==2)pipe.Set(false);
 else if(mode==3)pipe.SetDiscrete();
 else if(mode==4){gp_Pnt o=r.pnt();gp_Dir d=r.dir();gp_Dir x=r.dir();pipe.Set(gp_Ax2(o,d,x));}
 else if(mode==5)pipe.Set(r.dir());
 else if(mode==6)ga_flag(pipe.Set(t.shapes.at(r.integer())));
 else if(mode==7){TopoDS_Wire g=TopoDS::Wire(t.shapes.at(r.integer()));bool c=r.flag();
  pipe.Set(g,c,(BRepFill_TypeOfContact)r.integer());}}
// `program spine mode... transition angmin angmax maxdeg maxseg c1 n (profile vertex
// contact correction)... law [pdeb valdeb pfin valfin] solid`: a pipe shell (allocated
// on the heap) with the location mode, SetTransition, SetMaxDegree, SetMaxSegments,
// SetForceApproxC1 and the profiles (Add, at the spine vertex `vertex` unless -1),
// then with `law` SetLaw of the first profile with a Law_Linear; prints IsReady, Build,
// GetStatus, ErrorOnSurface, the shape and the first and last shapes, with `solid`
// MakeSolid and the shape, Generated of every shape of the program and the digests
// of Simulate(3).
void bl_bf_pipe_shell_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire spine=TopoDS::Wire(t.shapes.at(r.integer()));
 occ::handle<BRepFill_PipeShell> pipe;
 try{pipe=new BRepFill_PipeShell(spine);bl_pipe_mode(r,t,*pipe);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int transition=r.integer();double angmin=r.real(),angmax=r.real();
 pipe->SetTransition((BRepFill_TransitionStyle)transition,angmin,angmax);
 pipe->SetMaxDegree(r.integer());pipe->SetMaxSegments(r.integer());pipe->SetForceApproxC1(r.flag());
 int n=r.integer();TopoDS_Shape first;
 for(int k=0;k<n;++k){const TopoDS_Shape& p=t.shapes.at(r.integer());int v=r.integer();bool c=r.flag(),w=r.flag();
  if(k==0)first=p;
  if(v<0){GA_TRY(pipe->Add(p,c,w))}else{GA_TRY(pipe->Add(p,TopoDS::Vertex(t.shapes.at(v)),c,w))}}
 if(r.flag()){occ::handle<Law_Linear> lin=new Law_Linear();double a=r.real(),b=r.real(),c=r.real();lin->Set(a,b,c,r.real());
  GA_TRY(pipe->SetLaw(first,lin))}
 bool solid=r.flag();
 ga_flag(pipe->IsReady());
 bool built=false;
 try{built=pipe->Build();ga_flag(built);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_int(pipe->GetStatus());
 if(built)ga_real(pipe->ErrorOnSurface());
 GA_TRY(pr_shape(pipe->Shape()))
 GA_TRY(pr_shape(pipe->FirstShape()))
 GA_TRY(pr_shape(pipe->LastShape()))
 if(solid&&built)GA_TRY(ga_flag(pipe->MakeSolid());pr_shape(pipe->Shape()))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 if(!pipe->Shape().IsNull())TopExp::MapShapes(pipe->Shape(),map);
 for(const TopoDS_Shape& s:t.shapes){NCollection_List<TopoDS_Shape> l;GA_TRY(pipe->Generated(s,l);bl_list(map,l))}
 if(built){NCollection_List<TopoDS_Shape> l;GA_TRY(pipe->Simulate(3,l);ga_int(l.Extent());for(const TopoDS_Shape& s:l)pr_shape(s))}
 ga_end();}
