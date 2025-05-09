#include <iostream>
/// @rst
/// .. mydirective:: arg1 arg2
///         :option1: value1
///         :option2: value2
///    
///         This is the content of the directive.
/// @endrst
int main() {
    std::cout << "Hello, World!" << std::endl;
    return 0;
}