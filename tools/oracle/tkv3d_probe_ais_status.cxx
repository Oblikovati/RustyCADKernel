// Independent context status, ordered modes and retained drawer identity.
#include <AIS_GlobalStatus.hxx>
static void v3_ais_status_state(const AIS_GlobalStatus& status,const std::vector<occ::handle<Prs3d_Drawer>>& styles){
  ga_int(status.DisplayMode());ga_flag(status.IsHilighted());ga_flag(status.IsSubIntensityOn());
  int style=0;for(int i=0;i<3;++i)if(styles[i]==status.HilightStyle())style=i;ga_int(style);
  ga_int(status.SelectionModes().Size());for(int mode:status.SelectionModes())ga_int(mode);
}
void v3_ais_status(std::istream& input){
  GaReader r{input};AIS_GlobalStatus status;std::vector<occ::handle<Prs3d_Drawer>> styles={nullptr,new Prs3d_Drawer(),new Prs3d_Drawer()};
  v3_ais_status_state(status,styles);int count=r.integer();
  for(int i=0;i<count;++i){int op=r.integer(),value=r.integer();switch(op){
    case 0:status.SetDisplayMode(value);break;case 1:status.SetHilightStatus(value!=0);break;
    case 2:status.SetSubIntensity(value!=0);break;case 3:status.SetHilightStyle(styles[value]);break;
    case 4:ga_flag(status.AddSelectionMode(value));break;case 5:ga_flag(status.RemoveSelectionMode(value));break;
    case 6:status.ClearSelectionModes();break;case 7:ga_flag(status.IsSModeIn(value));break;
  }v3_ais_status_state(status,styles);}ga_end();
}
