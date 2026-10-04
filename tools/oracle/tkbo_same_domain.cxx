#include <BOPDS_DS.hxx>
#include <iostream>
void output(const BOPDS_DS& ds, int stage)
{
  for (int index : {-4, 1, 2, 3, 4, 5}) {
    int representative = 987;
    const bool mapped = ds.HasShapeSD(index, representative);
    std::cout << stage << ' ' << index << ' ' << mapped << ' '
              << representative << ' ' << ds.GetSameDomainIndex(index) << '\n';
  }
}
int main()
{
  BOPDS_DS ds;
  ds.AddShapeSD(1, 1);
  output(ds, 0);
  ds.AddShapeSD(1, 2);
  ds.AddShapeSD(2, 3);
  ds.AddShapeSD(3, 4);
  ds.AddShapeSD(-4, 1);
  output(ds, 1);
  ds.AddShapeSD(1, 1);
  ds.AddShapeSD(2, 5);
  output(ds, 2);
}
