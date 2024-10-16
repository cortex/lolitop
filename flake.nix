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

          libxkbcommon
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libxcb
          pkgs.glfw

        ];
        desktopItem = pkgs.makeDesktopItem {
          name = "se.frikod.lolitop";
          exec = "lolitop";
          icon = "lolitop.svg";
          comment = "Eye-candy system monitor .";
          desktopName = "lolitop";
          genericName = "CPU Usage Visualizer";
          categories = [ "Utility" ];
        };
      in
      {
        packages.default = naersk-lib.buildPackage rec {
          src = ./.;
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postInstall = ''
            wrapProgram "$out/bin/lolitop" --prefix LD_LIBRARY_PATH : "${libPath}"
            
            mkdir -p $out/share/icons/hicolor/scalable/apps
            mkdir -p $out/share/applications
            
            cp ${self}/assets/icon.svg $out/share/icons/hicolor/scalable/apps/lolitop.svg
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
            libGL
            wayland
            vulkan-loader


            libxkbcommon
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libxcb
            pkgs.glfw
          ];
          LD_LIBRARY_PATH = libPath;
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
      });
}
