let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
  unstable = import sources.nixpkgs-unstable {};
in

pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo rustfmt rls rustracer unstable.rust-analyzer

    # Dependencies for ggez:
    alsaLib # libasound2
    udev # libudev
    pkgconfig

    x11
    gnome3.zenity # for tinyfilepicker
  ];

  APPEND_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
    libGL
    xorg.libX11
    xlibs.libXcursor
    xlibs.libXi
    xlibs.libXrandr
  ];

  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$APPEND_LIBRARY_PATH"
  '';

}
