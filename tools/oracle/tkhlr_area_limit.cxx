#include <HLRBRep_AreaLimit.hxx>
#include <iostream>

static void print(const char* name, const occ::handle<HLRBRep_AreaLimit>& limit)
{
  std::cout << name << ' ' << limit->Vertex().Index() << ' '
            << limit->Vertex().Parameter() << ' ' << limit->IsBoundary() << ' '
            << limit->IsInterference() << ' ' << limit->StateBefore() << ' '
            << limit->StateAfter() << ' ' << limit->EdgeBefore() << ' '
            << limit->EdgeAfter() << ' ' << limit->Previous().IsNull() << ' '
            << limit->Next().IsNull() << '\n';
}

int main()
{
  HLRAlgo_Intersection vertex(TopAbs_REVERSED, 1, 2, 7, 0.25, 0.125f, TopAbs_ON);
  occ::handle<HLRBRep_AreaLimit> limit = new HLRBRep_AreaLimit(
    vertex, true, false, TopAbs_IN, TopAbs_OUT, TopAbs_ON, TopAbs_UNKNOWN);
  print("initial", limit);
  vertex.Index(99);
  occ::handle<HLRBRep_AreaLimit> neighbor = new HLRBRep_AreaLimit(
    vertex, false, true, TopAbs_UNKNOWN, TopAbs_ON, TopAbs_OUT, TopAbs_IN);
  print("neighbor", neighbor);
  limit->Previous(neighbor);
  limit->Next(limit);
  limit->StateBefore(TopAbs_UNKNOWN);
  limit->StateAfter(TopAbs_ON);
  limit->EdgeBefore(TopAbs_OUT);
  limit->EdgeAfter(TopAbs_IN);
  print("changed", limit);
  std::cout << "links " << (limit->Previous() == neighbor) << ' '
            << (limit->Next() == limit) << ' ' << neighbor->Next().IsNull() << '\n';
  limit->Clear();
  limit->Clear();
  print("cleared", limit);
}
