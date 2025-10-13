{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
  };

  outputs = {nixpkgs, ...}: let
    eachSupportedSystem = nixpkgs.lib.genAttrs supportedSystems;
    supportedSystems = [
      "x86_64-linux"
    ];

    mkDevShells = system: let
      inherit (pkgs) lib;
      pkgs = import nixpkgs {inherit system;};
      nativeBuildInputs = with pkgs; [
        pkg-config
      ];
      buildInputs = with pkgs; [
        udev
        alsa-lib
        vulkan-loader
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr # To use Bevy's x11 feature
        libxkbcommon
        wayland # To use Bevy's wayland feature
      ];
    in {
      default = pkgs.mkShell {
        inherit nativeBuildInputs buildInputs;
        strictDeps = true;
        LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
      };
    };
  in {
    devShells = eachSupportedSystem mkDevShells;
  };
}
