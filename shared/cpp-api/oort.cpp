#include "oort.h"

extern "C" {
    uint64_t SYSTEM_STATE[128];
    uint8_t ENVIRONMENT[1024];
    uint8_t PANIC_BUFFER[1024];
}

void std::__libcpp_verbose_abort(char const* format, ...) {
    // TODO write to panic buffer
    std::abort();
}

int main() {
    return 0;
}

void tick();

__attribute__((export_name("tick")))
void sys_tick() {
    tick();
}
