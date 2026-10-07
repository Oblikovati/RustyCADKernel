// Graphic3d_CLight and Graphic3d_LightSet, the Rust twin of crates/tkservice/examples/sv_probe/light.rs.
#include <Graphic3d_CLight.hxx>
#include <Graphic3d_LightSet.hxx>
void sv_light_state(const occ::handle<Graphic3d_CLight>& l){
 sv_vec4f(l->PackedParams());sv_vec4f(l->PackedColor());sv_vec4f(l->PackedDirectionRange());
 ga_pnt(l->Position());ga_real(l->Smoothness());ga_real(l->Intensity());ga_int((int)l->Revision());
 ga_flag(l->IsEnabled());ga_flag(l->ToCastShadows());ga_flag(l->IsHeadlight());ga_flag(l->HasRange());
 GA_TRY(ga_dir(l->Direction()))}
// One light operation `Op Args`: 1 R G B color, 2 On enabled, 3 On shadows, 4 On headlight,
// 5 X Y Z position, 6 X Y Z display position, 7 X Y Z direction, 8 V intensity, 9 V angle,
// 10 C L attenuation, 11 V concentration, 12 V smooth radius, 13 V smooth angle, 14 V range,
// 15 Type copy from a light of that type with fixed settings.
void sv_light_op(GaReader& r,const occ::handle<Graphic3d_CLight>& l){
 int op=r.integer();
 if(op==1){double cr=r.real(),cg=r.real(),cb=r.real();l->SetColor(Quantity_Color(cr,cg,cb,Quantity_TOC_RGB));}
 else if(op==2)l->SetEnabled(r.flag());
 else if(op==3){bool on=r.flag();GA_TRY(l->SetCastShadows(on))}
 else if(op==4){bool on=r.flag();GA_TRY(l->SetHeadlight(on))}
 else if(op==5||op==6||op==7){double x=r.real(),y=r.real(),z=r.real();
  if(op==5){GA_TRY(l->SetPosition(x,y,z))}else if(op==6){GA_TRY(l->SetDisplayPosition(gp_Pnt(x,y,z)))}else{GA_TRY(l->SetDirection(x,y,z))}}
 else if(op==10){float c=(float)r.real(),li=(float)r.real();GA_TRY(l->SetAttenuation(c,li))}
 else if(op==15){occ::handle<Graphic3d_CLight> src=new Graphic3d_CLight((Graphic3d_TypeOfLightSource)r.integer());
  src->SetName("source");src->SetIntensity(3.0f);src->SetColor(Quantity_Color(0.25,0.5,0.75,Quantity_TOC_RGB));
  GA_TRY(src->SetDirection(0.0,0.6,-0.8))GA_TRY(src->SetPosition(1.0,2.0,3.0))GA_TRY(src->SetHeadlight(true))
  GA_TRY(src->SetCastShadows(true))GA_TRY(src->SetAngle(0.25f))GA_TRY(src->SetSmoothRadius(0.5f))GA_TRY(src->SetSmoothAngle(0.125f))
  l->CopyFrom(src);ga_flag(l->Name()=="source");}
 else{float v=(float)r.real();
  if(op==8){GA_TRY(l->SetIntensity(v))}else if(op==9){GA_TRY(l->SetAngle(v))}else if(op==11){GA_TRY(l->SetConcentration(v))}
  else if(op==12){GA_TRY(l->SetSmoothRadius(v))}else if(op==13){GA_TRY(l->SetSmoothAngle(v))}else{GA_TRY(l->SetRange(v))}}}
// `Type N (Op Args)*`: a light, the state after construction and after each operation.
void sv_light_case(std::istream& input){
 GaReader r{input};occ::handle<Graphic3d_CLight> l=new Graphic3d_CLight((Graphic3d_TypeOfLightSource)r.integer());
 sv_light_state(l);int n=r.integer();for(int i=0;i<n;++i){sv_light_op(r,l);sv_light_state(l);}
 ga_end();}
void sv_light_set_state(const occ::handle<Graphic3d_LightSet>& s){
 ga_int((int)s->UpdateRevision());ga_int(s->Extent());
 for(int t=0;t<4;++t){ga_int(s->NbLightsOfType((Graphic3d_TypeOfLightSource)t));ga_int(s->NbEnabledLightsOfType((Graphic3d_TypeOfLightSource)t));}
 ga_int(s->NbEnabled());ga_int(s->NbCastShadows());sv_vec4f(s->AmbientColor());
 ga_text(s->KeyEnabledLong().ToCString());ga_text(s->KeyEnabledShort().ToCString());
 const int filters[]={Graphic3d_LightSet::IterationFilter_None,Graphic3d_LightSet::IterationFilter_ExcludeAmbient,
  Graphic3d_LightSet::IterationFilter_ExcludeDisabled,Graphic3d_LightSet::IterationFilter_ExcludeNoShadow,
  Graphic3d_LightSet::IterationFilter_ExcludeDisabledAndAmbient,Graphic3d_LightSet::IterationFilter_ActiveShadowCasters};
 for(int f:filters){int count=0;for(Graphic3d_LightSet::Iterator it(s,(Graphic3d_LightSet::IterationFilter)f);it.More();it.Next()){ga_int(it.Value()->Type());++count;}ga_int(count);}}
// `N (Type On Shadow Intensity R G B)* Remove Toggle`: lights added to a set (twice), the set
// state, then removing light Remove (1-based, 0 none) and disabling light Toggle.
void sv_light_set_case(std::istream& input){
 GaReader r{input};occ::handle<Graphic3d_LightSet> s=new Graphic3d_LightSet();int n=r.integer();
 std::vector<occ::handle<Graphic3d_CLight>> lights;
 for(int i=0;i<n;++i){occ::handle<Graphic3d_CLight> l=new Graphic3d_CLight((Graphic3d_TypeOfLightSource)r.integer());
  l->SetEnabled(r.flag());bool shadow=r.flag();if(shadow){GA_TRY(l->SetCastShadows(true))}
  float v=(float)r.real();GA_TRY(l->SetIntensity(v))double cr=r.real(),cg=r.real(),cb=r.real();
  l->SetColor(Quantity_Color(cr,cg,cb,Quantity_TOC_RGB));lights.push_back(l);ga_flag(s->Add(l));ga_flag(s->Add(l));}
 GA_TRY(s->Add(occ::handle<Graphic3d_CLight>()))
 ga_int((int)s->Revision());sv_light_set_state(s);ga_int((int)s->UpdateRevision());
 int removed=r.integer(),toggled=r.integer();
 if(removed>0){ga_flag(s->Remove(lights[removed-1]));ga_flag(s->Remove(lights[removed-1]));ga_flag(s->Contains(lights[removed-1]));}
 for(int i=1;i<=s->Extent();++i)ga_int(s->Value(i)->Type());GA_TRY(s->Value(0))
 if(toggled>0)lights[toggled-1]->SetEnabled(!lights[toggled-1]->IsEnabled());
 sv_light_set_state(s);ga_end();}
