#include <stdint.h>
#include <stdio.h>

uint32_t next_random(uint32_t previous) {
    return (uint32_t) (((uint64_t) previous * 48271) % 2147483647);
}

int main(void) {
    uint32_t seed = 1234;
    uint32_t random = next_random(seed);
    for (int i = 0; i < 10; i++) {
        printf("%d - %u\n", i, random);
        random = next_random(random);
    }

    return 0;
}
