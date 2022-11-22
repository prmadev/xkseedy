{
  description = "xkseedy";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    crane,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };

      inherit (pkgs) lib stdenv;

      craneLib = crane.lib.${system};
      src = craneLib.cleanCargoSource ./.;

      # If one needs to customize the build environment mostly only needed for
      # macos dependencies or frameworks.
      buildInputs =
        [
          pkgs.pkg-config
          pkgs.openssl
        ]
        ++ lib.optionals stdenv.isDarwin (lib.attrVals ["libiconv"] pkgs);

      # Build *just* the cargo dependencies, so we can reuse
      # all of that work (e.g. via cachix) when running in CI
      cargoArtifacts = craneLib.buildDepsOnly {
        inherit src buildInputs;
      };

      # Build the actual crate itself, reusing the dependency
      # artifacts from above.
      xkseedy = craneLib.buildPackage {
        inherit cargoArtifacts src buildInputs;
      };
    in {
      checks =
        {
          # Build the crate as part of `nix flake check` for convenience
          inherit xkseedy;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          # my-crate-clippy = craneLib.cargoClippy {
          #   inherit cargoArtifacts src buildInputs;
          #   cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          # };

          # nixme-doc = craneLib.cargoDoc {
          #   inherit cargoArtifacts src;
          # };

          # Check formatting
          # nixme-fmt = craneLib.cargoFmt {
          #   inherit src;
          # };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `my-crate` if you do not want
          # the tests to run twice
          # nixme-nextest = craneLib.cargoNextest {
          #   inherit cargoArtifacts src buildInputs;
          #   partitions = 1;
          #   partitionType = "count";
          # };
        }
        // lib.optionalAttrs (system == "x86_64-linux") {
          # NB: cargo-tarpaulin only supports x86_64 systems
          # Check code coverage (note: this will not upload coverage anywhere)
          # nixme-coverage = craneLib.cargoTarpaulin {
          #   inherit cargoArtifacts src;
          # };
        };

      packages.xkseedy = xkseedy;

      apps.nixme = flake-utils.lib.mkApp {
        drv = xkseedy;
      };

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks;

        # Extra inputs can be added here
        nativeBuildInputs = with pkgs; [
          cargo
          openssl
          pkg-config
          rustc
        ];
      };
    });
}