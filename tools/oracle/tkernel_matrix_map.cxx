#include <NCollection_Mat3.hxx>
#include <NCollection_Mat4.hxx>

#include <iostream>

void probe_mat3()
{
  double a3[9] = {0, 1, 2, 3, 4, 5, 6, 7, 8};
  NCollection_Mat3<double>& m3 = NCollection_Mat3<double>::Map(a3);
  std::cout << "mat3.alias " << (&m3 == reinterpret_cast<void*>(a3)) << '\n';
  std::cout << "mat3.before " << m3.GetValue(1, 2) << '\n';
  m3.SetValue(1, 2, 41.0);
  std::cout << "mat3.array_after_set " << a3[7] << '\n';
  a3[4] = 52.0;
  const NCollection_Mat3<double>& c3 = NCollection_Mat3<double>::Map(a3);
  std::cout << "mat3.const_after_array " << c3.GetValue(1, 1) << '\n';
}

void probe_mat4()
{
  double a4[16] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15};
  NCollection_Mat4<double>& m4 = NCollection_Mat4<double>::Map(a4);
  std::cout << "mat4.alias " << (&m4 == reinterpret_cast<void*>(a4)) << '\n';
  std::cout << "mat4.before " << m4.GetValue(1, 2) << '\n';
  m4.SetValue(1, 2, 63.0);
  std::cout << "mat4.array_after_set " << a4[9] << '\n';
  a4[14] = 74.0;
  const NCollection_Mat4<double>& c4 = NCollection_Mat4<double>::Map(a4);
  std::cout << "mat4.const_after_array " << c4.GetValue(2, 3) << '\n';
}

int main()
{
  probe_mat3();
  probe_mat4();
  return 0;
}
