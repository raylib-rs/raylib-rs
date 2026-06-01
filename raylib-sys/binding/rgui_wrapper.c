#include "../raylib/src/raylib.h"

// Route raygui's allocator macros through raylib's RL_MALLOC/RL_CALLOC/RL_FREE.
// Both default to libc malloc/calloc/free when no custom allocator is configured,
// so binary behavior is identical today. The reason this matters: when a downstream
// user redefines RL_MALLOC (raylib's allocator hook), raygui's mallocs now follow
// suit, which means our `MemFree` calls on `char**` returned from `GuiLoadIcons`
// stay allocator-correct under custom allocators. Without this define, raygui
// would always use libc malloc/free even when raylib is reconfigured.
#define RAYGUI_MALLOC(sz)     RL_MALLOC(sz)
#define RAYGUI_CALLOC(n, sz)  RL_CALLOC(n, sz)
#define RAYGUI_FREE(p)        RL_FREE(p)

#define RAYGUI_IMPLEMENTATION
#define RAYGUI_SUPPORT_ICONS
#define RLGL_IMPLEMENTATION
#define RLGL_SUPPORT_TRACELOG
#include "raygui.h"
#undef RAYGUI_IMPLEMENTATION
