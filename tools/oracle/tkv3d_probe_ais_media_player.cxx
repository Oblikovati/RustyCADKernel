// AIS_MediaPlayer without media input: construction state, frame sizing, presentation,
// selection and closing in a context, the twin of
// crates/tkv3d/examples/v3_probe/ais_media_player.rs. A request is
// `v3_ais_media_player X Y W H Count Op...`.
#include <AIS_InteractiveContext.hxx>
#include <AIS_MediaPlayer.hxx>
#include <Graphic3d_TransformPers.hxx>

class V3ProbeMediaPlayer : public AIS_MediaPlayer {
public:
  bool size(const NCollection_Vec2<int>& c,const NCollection_Vec2<int>& s){return updateSize(c,s);}
  NCollection_Vec2<int> frameSize() const{return myFrameSize;}
  NCollection_Vec2<int> frameLeft() const{return myFrameBottomLeft;}
  bool accepts(int m) const{return AcceptDisplayMode(m);}
};
static void v3_media_state(const occ::handle<V3ProbeMediaPlayer>& p){
  ga_int(p->ZLayer());ga_flag(p->IsInfinite());ga_flag(!p->TransformPersistence().IsNull());
  if(!p->TransformPersistence().IsNull()){ga_int(p->TransformPersistence()->Mode());ga_int(p->TransformPersistence()->Corner2d());}
  for(int m=-1;m<=1;++m)ga_flag(p->accepts(m));ga_flag(p->PlayerContext().IsNull());ga_real(p->Duration());
  ga_int(p->frameSize().x());ga_int(p->frameSize().y());ga_int(p->frameLeft().x());ga_int(p->frameLeft().y());ga_int(p->Presentations().Size());
}
void v3_ais_media_player(std::istream& input){
  GaReader r{input};const int x=r.integer(),y=r.integer(),w=r.integer(),h=r.integer(),count=r.integer();
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<AIS_InteractiveContext> ctx=new AIS_InteractiveContext(viewer);
  occ::handle<V3ProbeMediaPlayer> p=new V3ProbeMediaPlayer();
  v3_media_state(p);
  for(int i=0;i<count;++i){
    switch(r.integer()){
      case 0:GA_TRY(ga_flag(p->size(NCollection_Vec2<int>(x,y),NCollection_Vec2<int>(w,h))))break;
      case 1:GA_TRY(ga_flag(p->PresentFrame(NCollection_Vec2<int>(x,y),NCollection_Vec2<int>(w,h))))break;
      case 2:ctx->Display(p,false);break;
      case 3:p->SetClosePlayer();break;
      case 4:{occ::handle<SelectMgr_Selection> s=new SelectMgr_Selection(0);static_cast<SelectMgr_SelectableObject*>(p.get())->ComputeSelection(s,0);
        ga_int(s->Entities().Size());for(const auto& e:s->Entities()){ga_int(e->BaseSensitive()->NbSubElements());ga_int(e->BaseSensitive()->OwnerId()->Priority());}break;}
      case 5:p->PlayPause();break;
      case 6:p->OpenInput("",false);break;
    }
    ga_int(ctx->DisplayStatus(p));v3_media_state(p);
  }
  v3_display_trace(trace);ga_end();
}
