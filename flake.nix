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
          libxkbcommon
          wayland
          vulkan-loader
        ];
        desktopItem = pkgs.makeDesktopItem {
          name = "xtop";
          exec = "xtop";
          icon = "xtop.svg";
          comment = "Eye-candy system monitor .";
          desktopName = "XTop";
          genericName = "CPU Usage Visualizer";
          categories = [ "Utility" ];
        };
      in
      {
        packages.default = naersk-lib.buildPackage rec {
          src = ./.;
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postInstall = ''
            wrapProgram "$out/bin/xtop" --prefix LD_LIBRARY_PATH : "${libPath}"
            
            mkdir -p $out/share/icons
            mkdir -p $out/share/applications
            
            cp ${self}/assets/icon.svg $out/share/icons/xtop.svg
            cp ${desktopItem}/share/applications/${desktopItem.name} \
              $out/share/applications
          '';
        };
        devShell = with pkgs; mkShell {
          buildInputs = [
            cargo
            rustc
            rustfmt
            pre-commit
            rustPackages.clippy
            rust-analyzer
            wayland
            xorg.libX11
            libxkbcommon
            libGL
            wayland
            vulkan-loader
          ];
          LD_LIBRARY_PATH = libPath;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      });
}
