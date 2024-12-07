{ pkgs ? import <nixpkgs> { } }:
with pkgs;
mkShell {
  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    openssl
    lua
    duckdb
    xorg.libX11
    xorg.libXft
    xorg.libXi
    xorg.libxcb
    xorg.libXcursor
    libxkbcommon

  libGL
  libxkbcommon
  wayland
  xorg.libX11
  xorg.libXcursor
  xorg.libXi
  xorg.libXrandr
  ];
LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
  libGL
  libxkbcommon
  wayland
  xorg.libX11
  xorg.libXcursor
  xorg.libXi
  xorg.libXrandr
];
#  shellHook = ''
#	export LD_LIBRARY_PATH="$NIX_LD_LIBRARY_PATH"
#'';
}
