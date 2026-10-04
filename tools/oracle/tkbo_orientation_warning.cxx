#include "tkbo_seam_fixture.hxx"
#include <BOPTools_AlgoTools.hxx>
#include <Message_Report.hxx>
#include <TopoDS_AlertWithShape.hxx>
int main(){for(int mode=0;mode<3;mode++)for(int present=0;present<2;present++){
 auto fixture=pcurve_fixture(0);TopoDS_Shape split=fixture.face,original=fixture.face;
 if(mode==1){TopoDS_Compound compound;BRep_Builder().MakeCompound(compound);split=compound;original=compound;}
 if(mode==2){TopoDS_Face face;BRep_Builder().MakeFace(face,BRep_Tool::Surface(fixture.face),1e-7);split=face;original=pcurve_fixture(0).face;}
 Handle(IntTools_Context) context=new IntTools_Context();Handle(Message_Report) report;if(present)report=new Message_Report();
 bool result=BOPTools_AlgoTools::IsSplitToReverseWithWarn(split,original,context,report);
 BOPTools_AlgoTools::IsSplitToReverseWithWarn(split,original,context,report);
 std::cout<<mode<<' '<<present<<' '<<result<<' '<<(report.IsNull()?0:report->GetAlerts(Message_Warning).Size());
 if(!report.IsNull())for(auto const& alert:report->GetAlerts(Message_Warning)){
  auto shapeAlert=occ::down_cast<TopoDS_AlertWithShape>(alert);auto shape=shapeAlert->GetShape();int count=0;for(TopoDS_Iterator it(shape);it.More();it.Next())count++;std::cout<<' '<<count<<' '<<alert->GetMessageKey();
 }
 std::cout<<'\n';
}}
