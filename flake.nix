{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk/master";
    rust-overlay.url = "github:oxalica/rust-overlay";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    utils,
    naersk,
    rust-overlay,
    ...
  }:
    utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        naersk-lib = pkgs.callPackage naersk {};

        runtimeDeps = with pkgs; [
          hidapi
        ];

        buildDeps = with pkgs; [
          pkg-config
        ];
      in {
        packages = {
          default = naersk-lib.buildPackage {
            nativeBuildInputs = buildDeps;
            buildInputs = runtimeDeps;

            src = ./.;

            postInstall = ''
              install -Dm444 -t "$out/lib/udev/rules.d" *.rules
            '';
          };
        };
        devShell = pkgs.mkShell rec {
          nativeBuildInputs = buildDeps;

          buildInputs = with pkgs;
            [
              (rust-bin.stable.latest.default.override {
                extensions = ["rust-src"];
              })
              rust-analyzer-unwrapped
            ]
            ++ runtimeDeps;

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (buildInputs ++ nativeBuildInputs);
        };
      }
    );
}
