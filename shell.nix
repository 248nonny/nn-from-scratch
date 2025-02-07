{ pkgs ? import <nixpkgs> {} }:

with pkgs;

mkShell {
  nativeBuildInputs = with xorg; [
    libxcb
    libXcursor
    libXrandr
    libXi
    pkg-config
    libxkbcommon
  ] ++ [
    python3
    libGL
    libGLU
  ];
  buildInputs = [
    cargo
    rustc
    xorg.libX11
    wayland
    libxkbcommon
  ];

  shellHook = ''
      export LD_LIBRARY_PATH=/run/opengl-driver/lib/:${lib.makeLibraryPath ([libGL libGLU libxkbcommon])}
  '';
}
