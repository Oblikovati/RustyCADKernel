#include <Contap_ThePathPointOfTheSearch.hxx>
#include <Contap_TheSegmentOfTheSearch.hxx>
void ct_search_point(const Contap_ThePathPointOfTheSearch& point){
 ga_pnt(point.Value());ga_real(point.Tolerance());ga_flag(point.IsNew());ga_flag(point.Arc().IsNull());ga_real(point.Parameter());
 GA_TRY(ga_flag(point.Vertex().IsNull()));
}
void ct_search_segment(const Contap_TheSegmentOfTheSearch& segment){
 ga_flag(segment.Curve().IsNull());ga_flag(segment.HasFirstPoint());ga_flag(segment.HasLastPoint());
 GA_TRY(ct_search_point(segment.FirstPoint()));GA_TRY(ct_search_point(segment.LastPoint()));
}
void ct_search_records_case(std::istream& input){
 GaReader r{input};GA_TRY(auto arc=ct_arc(r);gp_Pnt position=r.pnt();double tolerance=r.real(),parameter=r.real();
 occ::handle<Adaptor3d_HVertex> vertex=new Adaptor3d_HVertex(gp_Pnt2d(),TopAbs_FORWARD,tolerance);
 Contap_ThePathPointOfTheSearch point;ct_search_point(point);Contap_TheSegmentOfTheSearch segment;ct_search_segment(segment);
 segment.SetValue(arc);ct_search_segment(segment);
 point=Contap_ThePathPointOfTheSearch(position,tolerance,vertex,arc,parameter);ct_search_point(point);
 segment.SetLimitPoint(point,true);ct_search_segment(segment);
 point.SetValue(position,tolerance,arc,parameter+.5);ct_search_point(point);ct_search_segment(segment);
 segment.SetLimitPoint(point,false);ct_search_segment(segment);
 point=Contap_ThePathPointOfTheSearch(position,tolerance,arc,parameter-.5);ct_search_point(point);
 point.SetValue(position,tolerance,vertex,arc,parameter);ct_search_point(point);
 segment.SetValue(occ::handle<Adaptor2d_Curve2d>());ct_search_segment(segment));ga_end();
}
