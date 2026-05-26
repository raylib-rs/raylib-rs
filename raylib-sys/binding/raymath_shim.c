/* Emit external (out-of-line) symbols for raymath's otherwise-inline functions,
   so raylib-rs can call raylib's own math through FFI. See raymath.h (RAYMATH_IMPLEMENTATION). */
#define RAYMATH_IMPLEMENTATION
#include "../raylib/src/raymath.h"
