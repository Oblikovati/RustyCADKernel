#include <Poly_MakeLoops.hxx>
#include <Standard_Failure.hxx>
#include <Standard_OutOfRange.hxx>

#include <iostream>

class ProbeLoops : public Poly_MakeLoops2D
{
public:
  class ProbeHelper : public Helper
  {
  public:
    const ListOfLink& GetAdjacentLinks(int) const override { return links; }
    bool GetFirstTangent(const Link&, gp_Dir2d&) const override { return false; }
    bool GetLastTangent(const Link&, gp_Dir2d&) const override { return false; }
    ListOfLink links;
  };

  explicit ProbeLoops(const Helper* helper)
      : Poly_MakeLoops2D(true, helper, occ::handle<NCollection_BaseAllocator>())
  {
  }

  Link getLinkForProbe(int index) const { return getLink(index); }
};

class ProbeLoops3D final : public Poly_MakeLoops3D
{
public:
  class ProbeHelper : public Helper
  {
  public:
    const ListOfLink& GetAdjacentLinks(int) const override { return links; }
    bool GetFirstTangent(const Link&, gp_Dir&) const override { return false; }
    bool GetLastTangent(const Link&, gp_Dir&) const override { return false; }
    bool GetNormal(int, gp_Dir&) const override { return false; }
    ListOfLink links;
  };

  explicit ProbeLoops3D(const Helper* helper)
      : Poly_MakeLoops3D(helper, occ::handle<NCollection_BaseAllocator>())
  {
  }

  int chooseForProbe(const NCollection_List<int>& candidates) const
  {
    return chooseLeftWay(1, 1, candidates);
  }
};

class ProbeLoops2D final : public ProbeLoops
{
public:
  using ProbeLoops::ProbeLoops;
  int chooseForProbe(const NCollection_List<int>& candidates) const
  {
    return chooseLeftWay(1, 1, candidates);
  }
};

template <class Choice>
void printEmptyChoice(const char* name, Choice choice)
{
  try
  {
    (void)choice();
    std::cout << name << " returned\n";
  }
  catch (const Standard_NoSuchObject& exception)
  {
    std::cout << name << ' ' << exception.what() << '\n';
  }
}

int main()
{
  ProbeLoops::ProbeHelper helper;
  ProbeLoops loops(&helper);
  loops.AddLink({7, 9});
  for (int index : {1, -1})
  {
    const Poly_MakeLoops::Link link = loops.getLinkForProbe(index);
    std::cout << index << ' ' << link.node1 << ' ' << link.node2 << ' ' << link.flags << '\n';
  }
  for (int index : {0, 2, -2})
  {
    try
    {
      (void)loops.getLinkForProbe(index);
      std::cout << index << " returned\n";
    }
    catch (const Standard_OutOfRange&)
    {
      std::cout << index << " Standard_OutOfRange\n";
    }
  }

  ProbeLoops nullHelper(nullptr);
  nullHelper.AddLink({11, 12});
  const auto stored = nullHelper.FindLink({11, 12});
  std::cout << "null-helper-storage " << stored.node1 << ' ' << stored.node2 << '\n';

  NCollection_List<int> empty;
  ProbeLoops2D loops2d(&helper);
  loops2d.AddLink({7, 9});
  printEmptyChoice("2d", [&] { return loops2d.chooseForProbe(empty); });
  ProbeLoops3D::ProbeHelper helper3d;
  ProbeLoops3D loops3d(&helper3d);
  printEmptyChoice("3d", [&] { return loops3d.chooseForProbe(empty); });
}
