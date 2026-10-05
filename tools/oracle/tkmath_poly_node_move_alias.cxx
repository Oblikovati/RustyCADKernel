#include <Poly_ArrayOfNodes.hxx>
#include <Poly_ArrayOfUVNodes.hxx>

#include <iostream>

static void reportNodeMove()
{
  Poly_ArrayOfNodes aSource(2);
  aSource.SetValue(0, gp_Pnt(1.0, 2.0, 3.0));
  Poly_ArrayOfNodes aTarget;
  aTarget.Move(aSource);
  std::cout << aSource.Size() << ' ' << aTarget.Size() << ' ' << aSource.IsDeletable()
            << ' ' << aTarget.IsDeletable() << '\n';
  aTarget.SetValue(0, gp_Pnt(4.0, 5.0, 6.0));
  const gp_Pnt aFromSource = aSource.Value(0);
  std::cout << aFromSource.X() << ' ' << aFromSource.Y() << ' ' << aFromSource.Z() << '\n';
  aSource.SetValue(1, gp_Pnt(7.0, 8.0, 9.0));
  const gp_Pnt aFromTarget = aTarget.Value(1);
  std::cout << aFromTarget.X() << ' ' << aFromTarget.Y() << ' ' << aFromTarget.Z() << '\n';
}

static void reportUvMove()
{
  Poly_ArrayOfUVNodes aSource(2);
  aSource.SetValue(0, gp_Pnt2d(1.0, 2.0));
  Poly_ArrayOfUVNodes aTarget;
  aTarget.Move(aSource);
  std::cout << aSource.Size() << ' ' << aTarget.Size() << ' ' << aSource.IsDeletable()
            << ' ' << aTarget.IsDeletable() << '\n';
  aTarget.SetValue(0, gp_Pnt2d(4.0, 5.0));
  const gp_Pnt2d aFromSource = aSource.Value(0);
  std::cout << aFromSource.X() << ' ' << aFromSource.Y() << '\n';
  aSource.SetValue(1, gp_Pnt2d(7.0, 8.0));
  const gp_Pnt2d aFromTarget = aTarget.Value(1);
  std::cout << aFromTarget.X() << ' ' << aFromTarget.Y() << '\n';
}

int main()
{
  reportNodeMove();
  reportUvMove();
  return 0;
}
