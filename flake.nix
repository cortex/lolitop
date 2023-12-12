{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
                libPath = with pkgs; lib.makeLibraryPath [
          libGL
          libxkbcommon
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];

      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [ 
            cargo 
            rustc 
            rustfmt 
            pre-commit 
            rustPackages.clippy 
            rust-analyzer wayland
xorg.libX11
            libxkbcommon
            libGL
          wayland
          ];
            LD_LIBRARY_PATH = libPath;
          # LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      });
}
