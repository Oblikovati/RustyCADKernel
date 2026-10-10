// AIS timeline callbacks and frozen timer parity against OCCT 3d097a0.
#include <AIS_Animation.hxx>
struct AnimationProbeAtom : AIS_Animation {
  AnimationProbeAtom(const char* name) : AIS_Animation(name) {}
  void update(const AIS_AnimationProgress& position) override {
    ga_text(Name().ToCString()); ga_real(position.Pts); ga_real(position.LocalPts);
    ga_real(position.LocalNormalized); ga_flag(IsStopped());
  }
};
void v3_ais_animation(std::istream& input) {
  GaReader reader{input};
  occ::handle<AIS_Animation> root = new AnimationProbeAtom("root");
  occ::handle<AIS_Animation> child = new AnimationProbeAtom("child");
  root->SetStartPts(reader.real()); root->SetOwnDuration(reader.real());
  child->SetStartPts(reader.real()); child->SetOwnDuration(reader.real());
  root->Add(child); root->Add(child);
  ga_int(root->Children().Length()); ga_real(root->Duration());
  root->Start(false);
  for(int index=0; index<3; ++index) { ga_flag(root->Update(reader.real())); }
  root->Pause(); ga_flag(root->IsStopped()); ga_flag(child->IsStopped());
  root->Clear(); ga_real(root->Duration()); root->UpdateTotalDuration();
  ga_real(root->Duration()); ga_end();
}
void v3_ais_animation_timer(std::istream& input) {
  GaReader reader{input};
  occ::handle<AIS_Animation> root = new AnimationProbeAtom("timer");
  root->SetOwnDuration(reader.real());
  double pts=reader.real(), speed=reader.real();
  root->StartTimer(pts,speed,true,true);
  ga_real(root->ElapsedTime()); ga_real(root->Timer()->PlaybackSpeed());
  ga_flag(root->Timer()->IsStarted()); ga_flag(root->IsStopped());
  ga_real(root->UpdateTimer()); root->Stop(); ga_real(root->ElapsedTime());
  ga_end();
}
void v3_ais_animation_edits(std::istream& input) {
  GaReader reader{input};
  occ::handle<AIS_Animation> root=new AIS_Animation("root");
  occ::handle<AIS_Animation> first=new AIS_Animation("same");
  occ::handle<AIS_Animation> second=new AIS_Animation("same");
  first->SetOwnDuration(reader.real()); second->SetOwnDuration(reader.real());
  root->Add(first); root->Add(second);
  ga_flag(root->Find("same")==first); ga_flag(root->Find("absent").IsNull());
  ga_flag(root->Replace(first,second)); ga_int(root->Children().Length());
  ga_flag(root->Remove(second)); ga_real(root->Duration());
  occ::handle<AIS_Animation> copy=new AIS_Animation("copy"); copy->CopyFrom(root);
  ga_text(copy->Name().ToCString()); ga_real(copy->Duration());
  copy->CopyFrom(copy); ga_int(copy->Children().Length()); ga_real(copy->Duration());
  GA_TRY(root->Add(occ::handle<AIS_Animation>())); ga_end();
}
