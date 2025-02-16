{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShellNoCC {
  packages = [
    pkgs.mdbook
    pkgs.mdbook-footnote
    pkgs.graphviz
    (pkgs.rust.packages.stable.rustPlatform.buildRustPackage rec {
      pname = "mdbook-graphviz";
      version = "0.2.1";

      src = pkgs.fetchFromGitHub {
        owner = "zetanumbers";
        repo = pname;
        rev = "ab472394625fa0d514108bdf885f88edd6fdc918";
        hash = "sha256-EKXNDoIBN4ocbLoXONM5oRm85wUphuEH7BWLgpd7wNs=";
      };

      useFetchCargoVendor = true;
      cargoHash = "sha256-8FjCaMHGbDjr2ZUuVEjiF4zNVdgrXsTZPLXJXhV9NGg=";

      buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [ pkgs.darwin.apple_sdk.frameworks.CoreServices ];

      doCheck = false;
      nativeCheckInputs = [ pkgs.graphviz ];
    })
  ];
}
