#include <Aspect_GenId.hxx>
#include <climits>
#include <iostream>
int main() {
  Aspect_GenId ids(1,INT_MAX);
  std::cout << ids.Available() << ' ' << ids.Next() << ' ' << ids.Next();
  ids.Free(1);
  std::cout << ' ' << ids.Next() << ' ' << ids.Next() << '\n';
}
