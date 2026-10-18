{
  description = "NovaLight — photo library and non-destructive raw developer, in pure Rust";

  inputs = {
    # NovaLight needs edition 2024 and rustc ≥ 1.90, so the flake tracks unstable. Consumers on a
    # release channel can retarget it: `inputs.novalight.inputs.nixpkgs.follows = "nixpkgs";`
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # Optional build input of the repository: the CJK fonts (BIZ UDPGothic, BIZ UDMincho, Noto Sans CJK SC,
    # Shippori Mincho) that `crates/engine/build.rs` embeds when `NOVAFOODS_FONTS_DIR` is set. Official
    # releases always build with them; without them everything works but Japanese text has no
    # glyphs. Pinned to the revision the release workflow uses — bump deliberately
    # (`nix flake update novafoods-fonts`); craftrules standards/fonts.md.
    novafoods-fonts = {
      url = "github:EuLucaDev/novafoods-fonts/3d9ab3a132ed0eff457e424226a435a901da30d3";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      novafoods-fonts,
    }:
    let
      inherit (nixpkgs) lib;

      # No x86_64-darwin: the pinned nixpkgs (26.11, "nixos-unstable") dropped support for it. On
      # an Intel Mac, retarget nixpkgs at a release that still has it, or install the .dmg from the
      # GitHub release.
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      # The AppStream metadata carries a release date. The flake source's own commit date is
      # reproducible, unlike "now"; a tarball or dirty tree without one falls back to the epoch.
      buildDate =
        let
          d = self.lastModifiedDate or "";
        in
        if builtins.stringLength d >= 8 then
          "${builtins.substring 0 4 d}-${builtins.substring 4 2 d}-${builtins.substring 6 2 d}"
        else
          "1970-01-01";

      packageArgs = {
        inherit novafoods-fonts buildDate;
      };

      forAllSystems = f: lib.genAttrs systems (system: f (import nixpkgs { inherit system; }) system);
    in
    {
      packages = forAllSystems (
        pkgs: _:
        let
          novalight = pkgs.callPackage ./nix/package.nix packageArgs;
        in
        {
          inherit novalight;
          default = novalight;
        }
      );

      # `nixpkgs.overlays = [ inputs.novalight.overlays.default ]` → `pkgs.novalight` everywhere.
      overlays.default = final: _prev: {
        novalight = final.callPackage ./nix/package.nix packageArgs;
      };

      devShells = forAllSystems (
        pkgs: system:
        let
          novalight = self.packages.${system}.novalight;
        in
        {
          default = pkgs.mkShell (
            {
              # The package's own native inputs, so `cargo build` in the shell needs no extra setup.
              inputsFrom = [ novalight ];
              packages = with pkgs; [
                cargo
                rustc
                clippy
                rustfmt
                rust-analyzer
              ];
              # `cargo run` dlopens the Vulkan loader and libxkbcommon like the packaged binary.
              LD_LIBRARY_PATH = lib.makeLibraryPath novalight.passthru.runtimeLibs;
              RUST_BACKTRACE = "1";
            }
            // lib.optionalAttrs (novafoods-fonts != null) {
              # CJK glyphs in `cargo run` too; build.rs reads this as NOVAFOODS_FONTS_DIR.
              NOVAFOODS_FONTS_DIR = toString novafoods-fonts;
            }
          );
        }
      );

      checks = forAllSystems (
        pkgs: system: {
          novalight = self.packages.${system}.novalight;

          # `nix fmt` formats the flake; this keeps it formatted.
          formatting = pkgs.runCommand "novalight-nix-formatting" { nativeBuildInputs = [ pkgs.nixfmt ]; } ''
            nixfmt --check ${./flake.nix} ${./nix}
            touch $out
          '';
        }
      );

      formatter = forAllSystems (pkgs: _: pkgs.nixfmt);
    };
}
