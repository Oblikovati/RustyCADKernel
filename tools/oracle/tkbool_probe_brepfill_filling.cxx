// BRepFill_Filling probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_filling.rs.
//
// The edges and faces are shapes of a shape program (tktopalgo_probe_shapes.cxx); shapes
// print through pr_shape (tkprim_probe_shape.cxx) and generated shapes through bl_list
// (tkbool_probe_brepfill_generator.cxx).
#include <BRepFill_Filling.hxx>
// `program degree nbpts nbiter aniso tol2d tol3d tolang tolcurv maxdeg maxseg
// n (edge face order bound)... m (face order)... k (0 x y z | 1 u v face order)... init`:
// a BRepFill_Filling with the edges (face -1 for none), the free faces, the points (in
// G0, or on a face) and the initial face (-1 for none); prints the index of each Add,
// IsDone after Build, the face and, when done, the G0, G1 and G2 errors and Generated
// of every shape of the program. G0Error(Index) is not printed: GeomPlate's
// EcartContraintesMil fills NbPt-1 of the myNbPtsOnCur entries it reads, so natively the
// maximum takes uninitialized memory.
static void bl_filling_adds(GaReader& r,const TpShapes& t,BRepFill_Filling& f){
 int n=r.integer();
 for(int i=0;i<n;i++){const TopoDS_Edge e=TopoDS::Edge(t.shapes.at(r.integer()));int face=r.integer();
  GeomAbs_Shape order=(GeomAbs_Shape)r.integer();bool bound=r.flag();
  if(face<0)ga_int(f.Add(e,order,bound));
  else ga_int(f.Add(e,TopoDS::Face(t.shapes.at(face)),order,bound));}
 int m=r.integer();
 for(int i=0;i<m;i++){const TopoDS_Face face=TopoDS::Face(t.shapes.at(r.integer()));
  ga_int(f.Add(face,(GeomAbs_Shape)r.integer()));}
 int k=r.integer();
 for(int i=0;i<k;i++){
  if(r.integer()==0){ga_int(f.Add(r.pnt()));continue;}
  double u=r.real(),v=r.real();const TopoDS_Face face=TopoDS::Face(t.shapes.at(r.integer()));
  ga_int(f.Add(u,v,face,(GeomAbs_Shape)r.integer()));}
 int init=r.integer();
 if(init>=0)f.LoadInitSurface(TopoDS::Face(t.shapes.at(init)));}
void bl_bf_filling_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int degree=r.integer(),nbpts=r.integer(),nbiter=r.integer();bool aniso=r.flag();
 double tol2d=r.real(),tol3d=r.real(),tolang=r.real(),tolcurv=r.real();
 int maxdeg=r.integer(),maxseg=r.integer();
 BRepFill_Filling f(degree,nbpts,nbiter,aniso,tol2d,tol3d,tolang,tolcurv,maxdeg,maxseg);
 try{bl_filling_adds(r,t,f);f.Build();}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(f.IsDone());
 GA_TRY(pr_shape(f.Face()))
 if(!f.IsDone()){ga_end();return;}
 GA_TRY(ga_real(f.G0Error());ga_real(f.G1Error());ga_real(f.G2Error()))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 TopExp::MapShapes(f.Face(),map);
 for(const TopoDS_Shape& g:t.shapes)GA_TRY(bl_list(map,f.Generated(g)))
 ga_end();}
