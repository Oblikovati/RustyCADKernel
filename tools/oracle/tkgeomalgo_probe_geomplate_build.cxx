// GeomPlate_BuildPlateSurface and GeomPlate_MakeApprox probe operations
// (tkgeomalgo_probe.cxx); the scene readers are in tkgeomalgo_probe_geomplate.cxx and the
// Rust twin is crates/tkgeomalgo/examples/ga_probe/geom_plate_build.rs.
#include <GeomPlate_BuildPlateSurface.hxx>
#include <GeomPlate_MakeApprox.hxx>
#include <GeomPlate_PlateG0Criterion.hxx>
#include <GeomPlate_PlateG1Criterion.hxx>
// A builder of `ctor degree nbPtsOnCur nbIter tol2d tol3d tolAng aniso nbBounds initSurf
// ncurves curves npoints points`: ctor 0 without initial surface (loaded after when given),
// 1 on the initial surface, 2 the old constructor on the arrays of the curves (their orders
// and point counts). The builder lives on the heap, whose memory the default manager clears,
// so members the native constructors leave uninitialized read zeros on both sides.
std::unique_ptr<GeomPlate_BuildPlateSurface> ga_gp_builder(GaReader& r){
 int ctor=r.integer(),degree=r.integer(),nbpts=r.integer(),nbiter=r.integer();
 double tol2d=r.real(),tol3d=r.real(),tolang=r.real();bool aniso=r.flag();int nbbounds=r.integer();
 occ::handle<Geom_Surface> init=ga_gp_surface(r);
 int nc=r.integer();std::vector<occ::handle<GeomPlate_CurveConstraint>> curves;for(int i=0;i<nc;i++)curves.push_back(ga_gp_curve_constraint(r));
 int np=r.integer();std::vector<occ::handle<GeomPlate_PointConstraint>> points;for(int i=0;i<np;i++)points.push_back(ga_gp_point_constraint(r));
 std::unique_ptr<GeomPlate_BuildPlateSurface> b;
 if(ctor==1)b.reset(new GeomPlate_BuildPlateSurface(init,degree,nbpts,nbiter,tol2d,tol3d,tolang,0.1,aniso));
 else if(ctor==2){occ::handle<NCollection_HArray1<int>> npt=new NCollection_HArray1<int>(1,nc),tang=new NCollection_HArray1<int>(1,nc);
  occ::handle<NCollection_HArray1<occ::handle<Adaptor3d_Curve>>> tab=new NCollection_HArray1<occ::handle<Adaptor3d_Curve>>(1,nc);
  for(int i=1;i<=nc;i++){npt->SetValue(i,curves[i-1]->NbPoints());tang->SetValue(i,curves[i-1]->Order());tab->SetValue(i,curves[i-1]->Curve3d());}
  b.reset(new GeomPlate_BuildPlateSurface(npt,tab,tang,degree,nbiter,tol2d,tol3d,tolang,0.1,aniso));}
 else b.reset(new GeomPlate_BuildPlateSurface(degree,nbpts,nbiter,tol2d,tol3d,tolang,0.1,aniso));
 if(ctor!=1&&!init.IsNull())b->LoadInitSurface(init);
 if(ctor!=2)for(const auto& c:curves)b->Add(c);
 for(const auto& p:points)b->Add(p);
 if(nbbounds!=0)b->SetNbBounds(nbbounds);
 return b;
}
void ga_gp_type(const occ::handle<Standard_Transient>& t){if(t.IsNull())ga_text("null");else ga_text(t->DynamicType()->Name());}
void ga_gp_int_array(const occ::handle<NCollection_HArray1<int>>& a){ga_int(a->Length());for(int v:a->Array1())ga_int(v);}
// The curve constraints after Perform: points, 2d curve and gaps.
void ga_gp_build_curves(GeomPlate_BuildPlateSurface& b,int nc,bool gaps){
 for(int i=1;i<=nc;i++){occ::handle<GeomPlate_CurveConstraint> cc=b.CurveConstraint(i);ga_int(cc->NbPoints());
  GA_TRY(ga_gp_curve2d(cc->Curve2dOnSurf(),cc->FirstParameter(),cc->LastParameter()));
  if(gaps){GA_TRY(ga_real(b.G0Error(i)));GA_TRY(ga_real(b.G1Error(i)));GA_TRY(ga_real(b.G2Error(i)));}}
 GA_TRY(ga_gp_int_array(b.Sense()));GA_TRY(ga_gp_int_array(b.Order()));
 GA_TRY(occ::handle<NCollection_HArray1<occ::handle<Geom2d_Curve>>> c2=b.Curves2d();ga_int(c2->Length());
  for(const auto& c:c2->Array1()){ga_flag(c.IsNull());if(!c.IsNull())GA_TRY(ga_gp_curve2d(c,c->FirstParameter(),c->LastParameter()));});
}
// The contours of MakeApprox: the 2d samples and the 3d points and normals.
void ga_gp_build_contours(GeomPlate_BuildPlateSurface& b){
 GA_TRY(NCollection_Sequence<gp_XY> s2;b.Disc2dContour(4,s2);ga_int(s2.Length());for(const gp_XY& p:s2)ga_xy(p));
 for(int order=0;order<2;order++)
  GA_TRY(NCollection_Sequence<gp_XYZ> s3;b.Disc3dContour(4,order,s3);ga_int(s3.Length());for(const gp_XYZ& p:s3)ga_xyz(p));
}
// `builder samples gaps`: Perform, then every result of the builder and the plate surface at
// the sample parameters `(u v)`. The global errors are read only once a check ran (curves, a
// plate surface, done), and the gaps of each curve (G0Error(Index)...) only when `gaps` is set:
// both read uninitialized memory natively otherwise (the corpus sets `gaps` only where every
// value read is written).
void ga_gp_build_case(std::istream& input){
 GaReader r{input};
 GA_TRY(std::unique_ptr<GeomPlate_BuildPlateSurface> b=ga_gp_builder(r);int ns=r.integer();std::vector<double> uv;for(int i=0;i<2*ns;i++)uv.push_back(r.real());bool gaps=r.flag();
  GA_TRY(b->Perform());ga_flag(b->IsDone());occ::handle<GeomPlate_Surface> s=b->Surface();ga_flag(s.IsNull());
  occ::handle<Geom_Surface> init=b->SurfInit();ga_gp_type(init);if(!init.IsNull())GA_TRY(ga_gp_bounds(init));
  if(!s.IsNull()){ga_gp_bounds(s);GA_TRY(double a,bb,c,d;s->RealBounds(a,bb,c,d);ga_real(a);ga_real(bb);ga_real(c);ga_real(d));
   for(int i=0;i<ns;i++)ga_gp_eval(s,uv[2*i],uv[2*i+1]);}
  int nc=0;while(true){try{b->CurveConstraint(nc+1);nc++;}catch(const Standard_Failure& e){ga_text(e.ExceptionType());break;}}
  if(nc>0&&b->IsDone()&&!s.IsNull()){ga_real(b->G0Error());ga_real(b->G1Error());ga_real(b->G2Error());}
  GA_TRY(b->CurveConstraint(0));
  ga_gp_build_curves(*b,nc,gaps);
  int np=0;while(true){try{occ::handle<GeomPlate_PointConstraint> p=b->PointConstraint(np+1);np++;ga_flag(p->HasPnt2dOnSurf());ga_pnt2d(p->Pnt2dOnSurf());}
   catch(const Standard_Failure& e){ga_text(e.ExceptionType());break;}}
  GA_TRY(b->PointConstraint(0));
  // The contours dereference a null initial surface and read the parametric tolerances,
  // which only a Perform that reaches the plate sets.
  if(!s.IsNull())ga_gp_build_contours(*b));
 ga_end();
}
// `init plate mode tol3d nbmax dgmax continuity enlarge ...`: GeomPlate_MakeApprox of the plate
// surface; mode 0 the constructor with CritOrder (dmax critOrder), 1 and 2 an explicit G0 or
// G1 criterion on the plate constraints moved by `offset` (maximum type repartition offset xyz).
void ga_gp_approx_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom_Surface> init=ga_gp_surface(r);Plate_Plate plate;ga_gp_plate(r,plate);
  occ::handle<GeomPlate_Surface> gs=new GeomPlate_Surface(init,plate);int mode=r.integer();double tol3d=r.real();
  int nbmax=r.integer(),dgmax=r.integer();GeomAbs_Shape cont=GeomAbs_Shape(r.integer());double enlarge=r.real();
  std::unique_ptr<GeomPlate_MakeApprox> app;bool errors=true;
  if(mode==0){double dmax=r.real();int crit=r.integer();app.reset(new GeomPlate_MakeApprox(gs,tol3d,nbmax,dgmax,dmax,crit,cont,enlarge));
   // Another CritOrder leaves both errors uninitialized.
   errors=crit>=-1&&crit<=1;}
  else{double maximum=r.real();AdvApp2Var_CriterionType type=AdvApp2Var_CriterionType(r.integer());
   AdvApp2Var_CriterionRepartition rep=AdvApp2Var_CriterionRepartition(r.integer());gp_XYZ offset=r.xyz();
   NCollection_Sequence<gp_XY> s2;NCollection_Sequence<gp_XYZ> s3;gs->Constraints(s2);
   for(const gp_XY& p:s2){gp_Pnt pp;gp_Vec d1,d2;gs->D1(p.X(),p.Y(),pp,d1,d2);
    s3.Append(mode==1?pp.XYZ()+offset:(d1^d2).XYZ()+offset);}
   if(mode==1){GeomPlate_PlateG0Criterion crit(s2,s3,maximum,type,rep);app.reset(new GeomPlate_MakeApprox(gs,crit,tol3d,nbmax,dgmax,cont,enlarge));}
   else{GeomPlate_PlateG1Criterion crit(s2,s3,maximum,type,rep);app.reset(new GeomPlate_MakeApprox(gs,crit,tol3d,nbmax,dgmax,cont,enlarge));}}
  ga_gapi_bsurface_out(app->Surface());if(errors){ga_real(app->ApproxError());ga_real(app->CriterionError());});
 ga_end();
}
