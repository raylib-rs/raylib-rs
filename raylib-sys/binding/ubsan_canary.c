/* UBSAN wire-up canary — TEMPORARY, deleted in Task 5.
 *
 * Performs a deliberate INT_MAX + 1 signed overflow so the sanitizers
 * workflow can prove `-Clink-arg=-fsanitize=undefined` is actually
 * linking libubsan. See docs/superpowers/specs/2026-05-30-ubsan-through-ffi-design.md §4.3.
 *
 * Both operands are `volatile` to defeat compile-time folding (gcc would
 * otherwise compute the overflow at compile time and emit no runtime
 * check, even with -fsanitize=undefined).
 */
#include <limits.h>

void rlrust_ubsan_canary(void) {
    volatile int x = INT_MAX;
    volatile int one = 1;
    volatile int y = x + one;
    (void)y;
}
