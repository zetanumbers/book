{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShellNoCC {
  packages = [
    pkgs.mdbook
    pkgs.mdbook-footnote
  ];
}
