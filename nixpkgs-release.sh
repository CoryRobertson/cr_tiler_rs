#! /usr/bin/env nix-shell
#! nix-shell --pure -i bash -p git bash cargo rustc libGL xorg.libX11 xorg.libXi alsa-lib wayland

cargo build -r