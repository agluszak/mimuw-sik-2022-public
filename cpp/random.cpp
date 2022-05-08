#include <iostream>
#include <random>
#include <cstdint>

int main() {
    std::uint32_t seed = 1234;

    std::minstd_rand random(seed);
    for (int i = 0; i < 10; i++) {
        std::cout << i << " - " << random() << std::endl;
    }

    return 0;
}
