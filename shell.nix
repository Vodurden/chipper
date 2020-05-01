let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
  unstable = import sources.nixpkgs-unstable {};
in

pkgs.mkShell {
  buildInputs = with pkgs; [ cargo rustfmt rls rustracer unstable.rust-analyzer ];
}
