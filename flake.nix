{
  description = "CDK Flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, pre-commit-hooks, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        lib = pkgs.lib;
        stdenv = pkgs.stdenv;
        isDarwin = stdenv.isDarwin;
        libsDarwin = with pkgs; lib.optionals isDarwin [
          # Additional darwin specific inputs can be set here
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        # Dependencies
        pkgs = import nixpkgs {
          inherit system overlays;
        };



        # Toolchains
        # latest stable
        stable_toolchain = pkgs.rust-bin.stable."1.86.0".default.override {
          targets = [ "wasm32-unknown-unknown" ]; # wasm
          extensions = [ "rustfmt" "clippy" "rust-analyzer" ];
        };

        # MSRV stable
        msrv_toolchain = pkgs.rust-bin.stable."1.75.0".default.override {
          targets = [ "wasm32-unknown-unknown" ]; # wasm
        };

        # Nightly used for formatting
        nightly_toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rustfmt" "clippy" "rust-analyzer" "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ]; # wasm
        });

        # Common inputs
        envVars = { };
        buildInputs = with pkgs; [
          # Add additional build inputs here
          git
          pkg-config
          curl
          just
          nixpkgs-fmt
          typos
          cargo-outdated

          # Needed for github ci
          libz
        ] ++ libsDarwin;


        nativeBuildInputs = [
          #Add additional build inputs here
        ] ++ lib.optionals isDarwin [
          # Additional darwin specific native inputs can be set here
        ];
      in
      {
        checks = {
          # Pre-commit checks
          pre-commit-check =
            let
              # this is a hack based on https://github.com/cachix/pre-commit-hooks.nix/issues/126
              # we want to use our own rust stuff from oxalica's overlay
              _rust = pkgs.rust-bin.stable.latest.default;
              rust = pkgs.buildEnv {
                name = _rust.name;
                inherit (_rust) meta;
                buildInputs = [ pkgs.makeWrapper ];
                paths = [ _rust ];
                pathsToLink = [ "/" "/bin" ];
                postBuild = ''
                  for i in $out/bin/*; do
                    wrapProgram "$i" --prefix PATH : "$out/bin"
                  done
                '';
              };
            in
            pre-commit-hooks.lib.${system}.run {
              src = ./.;
              hooks = {
                rustfmt = {
                  enable = true;
                  entry = lib.mkForce "${rust}/bin/cargo-fmt fmt --all -- --config format_code_in_doc_comments=true --check --color always";
                };
                nixpkgs-fmt.enable = true;
                typos.enable = true;
                commitizen.enable = true; # conventional commits
              };
            };
        };

        devShells =
          let
            # pre-commit-checks
            _shellHook = (self.checks.${system}.pre-commit-check.shellHook or "");

            # devShells
            msrv = pkgs.mkShell ({
              shellHook = "
              ${_shellHook}
              cargo update


              cargo update -p zerofrom --precise 0.1.5

              cargo update -p url --precise 2.5.2


              ";
              buildInputs = buildInputs ++ [ msrv_toolchain ];
              inherit nativeBuildInputs;
            } // envVars);

            stable = pkgs.mkShell ({
              shellHook = ''${_shellHook}'';
              buildInputs = buildInputs ++ [ stable_toolchain ];
              inherit nativeBuildInputs;
            } // envVars);


            nightly = pkgs.mkShell ({
              shellHook = ''
                ${_shellHook}
                # Needed for github ci
                export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath [
                  pkgs.zlib
                  ]}:$LD_LIBRARY_PATH
                export RUST_SRC_PATH=${nightly_toolchain}/lib/rustlib/src/rust/library
              '';
              buildInputs = buildInputs ++ [ nightly_toolchain ];
              inherit nativeBuildInputs;
            } // envVars);

          in
          {
            inherit msrv stable nightly;
            default = stable;
          };
      }
    );
}
