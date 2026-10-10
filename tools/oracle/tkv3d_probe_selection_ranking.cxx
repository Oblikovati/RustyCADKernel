// Native adaptive-tolerance traces and candidate comparisons.
#include <SelectMgr_ToleranceMap.hxx>
#include <SelectMgr_SortCriterion.hxx>
void v3_tolerance_state(const SelectMgr_ToleranceMap& map) {
  ga_int(map.Tolerance());ga_int(map.CustomTolerance());ga_flag(map.IsCustomTolSet());
}
void v3_tolerance_case(std::istream& input) {
  GaReader r{input};SelectMgr_ToleranceMap map;v3_tolerance_state(map);int count=r.integer();
  for(int i=0;i<count;++i){int operation=r.integer(),tolerance=r.integer();
    if(operation==0)map.Add(tolerance);if(operation==1){GA_TRY(map.Decrement(tolerance))}
    if(operation==2)map.SetCustomTolerance(tolerance);if(operation==3)map.ResetDefaults();v3_tolerance_state(map);}
  ga_end();
}
SelectMgr_SortCriterion v3_criterion_read(GaReader& r) {
  SelectMgr_SortCriterion c;float x=static_cast<float>(r.real()),y=static_cast<float>(r.real()),z=static_cast<float>(r.real());
  c.Normal=NCollection_Vec3<float>(x,y,z);c.Depth=r.real();c.MinDist=r.real();c.Tolerance=r.real();
  c.SelectionPriority=r.integer();c.DisplayPriority=r.integer();c.ZLayerPosition=r.integer();c.NbOwnerMatches=r.integer();c.IsPreferPriority=r.flag();return c;
}
void v3_criterion_state(const SelectMgr_SortCriterion& c) {
  ga_flag(c.Entity.IsNull());ga_pnt(c.Point);ga_real(c.Normal.x());ga_real(c.Normal.y());ga_real(c.Normal.z());
  ga_real(c.Depth);ga_real(c.MinDist);ga_real(c.Tolerance);ga_int(c.SelectionPriority);ga_int(c.DisplayPriority);ga_int(c.ZLayerPosition);ga_int(c.NbOwnerMatches);ga_flag(c.IsPreferPriority);
}
void v3_criterion_case(std::istream& input) {
  GaReader r{input};SelectMgr_SortCriterion a,b;if(r.flag()){a=v3_criterion_read(r);b=v3_criterion_read(r);}
  v3_criterion_state(a);v3_criterion_state(b);GA_TRY(ga_flag(a.IsCloserDepth(b)))GA_TRY(ga_flag(b.IsCloserDepth(a)))
  ga_flag(a.IsHigherPriority(b));ga_flag(b.IsHigherPriority(a));ga_end();
}
