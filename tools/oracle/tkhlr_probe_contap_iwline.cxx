#include <Contap_TheIWLineOfTheIWalking.hxx>
#include <NCollection_IncAllocator.hxx>
void ct_iwline_point(const IntSurf_PntOn2S& point){
 ga_pnt(point.Value());double u,v,s,t;point.Parameters(u,v,s,t);ga_real(u);ga_real(v);ga_real(s);ga_real(t);
}
void ct_iwline_report(const occ::handle<Contap_TheIWLineOfTheIWalking>& l){
 int n=l->NbPoints();ga_int(n);for(int i=1;i<=n;i++)ct_iwline_point(l->Value(i));
 ga_flag(l->IsClosed());ga_flag(l->HasFirstPoint());ga_flag(l->HasLastPoint());
 GA_TRY(ga_int(l->FirstPointIndex()));GA_TRY(ga_pnt(l->FirstPoint().Value()));
 GA_TRY(ga_int(l->LastPointIndex()));GA_TRY(ga_pnt(l->LastPoint().Value()));
 ga_int(l->NbPassingPoint());for(int i=1;i<=l->NbPassingPoint();i++){int a,b;l->PassingPoint(i,a,b);ga_int(a);ga_int(b);}
 int idx=0;const gp_Vec& v=l->TangentVector(idx);ga_xyz(v.XYZ());ga_int(idx);
 ga_flag(l->IsTangentAtBegining());ga_flag(l->IsTangentAtEnd());
}
void ct_iwline_case(std::istream& input){
 GaReader r{input};occ::handle<Contap_TheIWLineOfTheIWalking> l=new Contap_TheIWLineOfTheIWalking(new NCollection_IncAllocator());
 int n=r.integer();
 for(int i=0;i<n;++i){int c=r.integer();GA_TRY(
  if(c==1){gp_Pnt x=r.pnt();double a=r.real(),b=r.real(),e=r.real(),d=r.real();IntSurf_PntOn2S q;q.SetValue(x,a,b,e,d);l->AddPoint(q);}
  else if(c==2){bool cl=r.flag();l->AddStatusFirst(cl,r.flag());}
  else if(c==3){bool cl=r.flag(),hf=r.flag();int k=r.integer();gp_Pnt x=r.pnt();double u=r.real();l->AddStatusFirst(cl,hf,k,IntSurf_PathPoint(x,u,r.real()));}
  else if(c==4){bool cl=r.flag(),hf=r.flag();l->AddStatusFirstLast(cl,hf,r.flag());}
  else if(c==5)l->AddStatusLast(r.flag());
  else if(c==6){bool hl=r.flag();int k=r.integer();gp_Pnt x=r.pnt();double u=r.real();l->AddStatusLast(hl,k,IntSurf_PathPoint(x,u,r.real()));}
  else if(c==7)l->AddIndexPassing(r.integer());
  else if(c==8){gp_XYZ v=r.xyz();l->SetTangentVector(gp_Vec(v),r.integer());}
  else if(c==9)l->SetTangencyAtBegining(r.flag());
  else if(c==10)l->SetTangencyAtEnd(r.flag());
  else if(c==11)l->Reverse();
  else l->Cut(r.integer()));}
 ct_iwline_report(l);ga_int(l->Line()->NbPoints());
 GA_TRY(l->Value(0));GA_TRY(l->Value(l->NbPoints()+1));GA_TRY(int a,b;l->PassingPoint(0,a,b));
 GA_TRY(int a,b;l->PassingPoint(l->NbPassingPoint()+1,a,b));
 ga_end();
}
