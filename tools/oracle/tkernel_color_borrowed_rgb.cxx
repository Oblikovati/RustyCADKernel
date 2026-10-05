#include <Quantity_Color.hxx>
#include <iostream>

static void borrowed_components(const Quantity_Color& color)
{
  const auto& first = color.Rgb();
  const auto& second = color.Rgb();
  std::cout << (&first == &second) << ' ' << first.x() << ' ' << first.y()
            << ' ' << first.z() << '\n';
}

int main()
{
  borrowed_components(Quantity_Color());
  borrowed_components(Quantity_Color(0.25, 0.5, 0.75, Quantity_TOC_RGB));
}
