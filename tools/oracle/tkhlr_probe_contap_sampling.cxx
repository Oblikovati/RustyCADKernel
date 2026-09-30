// Controlled virtual adaptors expose native shared-range and exception state.
#include <Contap_HContTool.hxx>
#include <Adaptor3d_Surface.hxx>
#include <Standard_DomainError.hxx>
class ContourSamplingSurface : public Adaptor3d_Surface {
public:
 GeomAbs_SurfaceType kind; double bounds[4]; int counts[6]; int failureField;
 void check(int field) const {if(failureField==field) throw Standard_DomainError("contour fixture");}
 double bound(int index,int field)const{check(field);return bounds[index];}
 int count(int index,int field)const{check(field);return counts[index];}
 double FirstUParameter()const override{return bound(0,1);}
 double LastUParameter()const override{return bound(2,2);}
 double FirstVParameter()const override{return bound(1,3);}
 double LastVParameter()const override{return bound(3,4);}
 GeomAbs_SurfaceType GetType()const override{check(5);return kind;}
 int NbUKnots()const override{return count(0,6);}
 int UDegree()const override{return count(2,7);}
 int NbVKnots()const override{return count(1,8);}
 int VDegree()const override{return count(3,9);}
 int NbUPoles()const override{return count(4,10);}
 int NbVPoles()const override{return count(5,11);}
};
occ::handle<Adaptor3d_Surface> ct_sampling_surface(GaReader& r){
 auto s=new ContourSamplingSurface;s->kind=GeomAbs_SurfaceType(r.integer());
 for(double& x:s->bounds)x=r.real();for(int& x:s->counts)x=r.integer();
 s->failureField=r.integer();return s;
}
void ct_sample_pair(const occ::handle<Adaptor3d_Surface>& s,int index){
 GA_TRY(double u,v;Contap_HContTool::SamplePoint(s,index,u,v);ga_real(u);ga_real(v));
}
void ct_sampling_case(std::istream& input){
 GaReader r{input};auto baseline=ct_sampling_surface(r);auto changed=ct_sampling_surface(r);
 typedef Contap_HContTool T;
 GA_TRY(ga_int(T::NbSamplePoints(baseline)));
 GA_TRY(ga_int(T::NbSamplesU(changed,-1.,2.)));
 GA_TRY(ga_int(T::NbSamplesV(changed,-1.,2.)));
 for(int i:{-1,0,1,2,3,4,5,6,17,40})ct_sample_pair(changed,i);
 GA_TRY(ga_int(T::NbSamplePoints(changed)));
 for(int i:{-1,0,1,2,3,4,5,6,17,40}){ct_sample_pair(baseline,i);ct_sample_pair(changed,i);}
 ga_end();
}
