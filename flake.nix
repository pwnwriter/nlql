{
  description = "nlql - Talk to your database in plain English";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system (import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      }));

      mkPackage = system: pkgs:
        let
          inherit (pkgs) lib stdenv;
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "nlql";
          version = "0.1.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          meta = with lib; {
            description = "Talk to your database in plain English using AI";
            homepage = "https://github.com/pwnwriter/nlql";
            license = licenses.mit;
            mainProgram = "nlql";
          };
        };
    in
    {
      packages = forAllSystems (system: pkgs: {
        default = mkPackage system pkgs;
        nlql = mkPackage system pkgs;
      });

      apps = forAllSystems (system: pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/nlql";
        };
      });

      devShells = forAllSystems (system: pkgs: {
        default = import ./nix/shell.nix { inherit pkgs; };
      });
    };
}
