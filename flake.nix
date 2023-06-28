{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, fenix, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system: {
      packages.default =
        let
          toolchain = fenix.packages.${system}.minimal.toolchain;
          pkgs = nixpkgs.legacyPackages.${system};
        in

        (pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        }).buildRustPackage {
          pname = "git-commands";
          version = "0.2.0";

          src = ./.;
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.openssl] ++ (if pkgs.stdenv.isDarwin then [pkgs.darwin.apple_sdk.frameworks.Security ] else []);

          cargoLock.lockFile = ./Cargo.lock;
        };
    });
}
