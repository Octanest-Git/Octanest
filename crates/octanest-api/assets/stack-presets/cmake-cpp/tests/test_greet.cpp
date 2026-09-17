#include <cassert>
#include <iostream>
#include "greet.hpp"

int main() {
  assert(greet("Octanest") == "Hello, Octanest!");
  std::cout << "ok\n";
  return 0;
}
