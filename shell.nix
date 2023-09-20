let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    packages = [
      pkgs.git
      pkgs.bash
      pkgs.cargo
      pkgs.rustc
      pkgs.libGL
      pkgs.xorg.libX11
      pkgs.xorg.libXi
      pkgs.alsa-lib
      pkgs.wayland
    ];
  }