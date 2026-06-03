{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  packages = with pkgs; [
    rustup
    glfw
    cmake
    clang
    wayland
    libx11
    xorg.libXrandr
    xorg.libXinerama
    xorg.libXcursor
    xorg.libXi
    libGL
    # Web support (uncomment to enable) -- Untested - @JamesKEbert
    # emscripten
  ];

  LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
    libGL
  ];
  LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
}

