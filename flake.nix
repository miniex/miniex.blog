{
  description = "miniex.blog — Rust (Axum) blog with Tailwind CSS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.stable.latest.default;
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "blog";
            version = "0.1.0";

            src = pkgs.lib.cleanSourceWith {
              src = ./.;
              filter = path: type:
                let
                  baseName = builtins.baseNameOf path;
                in
                baseName != "node_modules"
                && baseName != "target"
                && baseName != "data"
                && baseName != ".git"
                && baseName != "result"
                && baseName != "flake.nix"
                && baseName != "flake.lock"
                && (pkgs.lib.cleanSourceFilter path type);
            };

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [
              pkg-config
              bun
              makeWrapper
            ];

            buildInputs = with pkgs; [
              openssl
              sqlite
            ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
              pkgs.libiconv
            ];

            preBuild = ''
              # Install node dependencies and build Tailwind CSS
              export HOME=$(mktemp -d)
              bun install --frozen-lockfile
              bunx tailwindcss \
                -i ./assets/styles/tailwind.input.css \
                -o ./assets/styles/tailwind.output.css \
                --minify
            '';

            postInstall = ''
              # Copy runtime assets alongside the binary
              mkdir -p $out/share/blog
              cp -r assets $out/share/blog/
              cp -r templates $out/share/blog/
              cp -r contents $out/share/blog/

              # Wrap the binary so it can find its runtime files
              wrapProgram $out/bin/blog \
                --run 'cd ${placeholder "out"}/share/blog'
            '';

            meta = with pkgs.lib; {
              description = "miniex.blog — personal blog built with Axum";
              license = with licenses; [ mit asl20 ];
              mainProgram = "blog";
            };
          };
        });

      devShells = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustToolchain
              pkg-config
              openssl
              sqlite
              bun
              nodejs
            ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
              pkgs.libiconv
            ];

            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

            shellHook = ''
              echo "miniex.blog dev shell"
              echo "  rust : $(rustc --version)"
              echo "  cargo: $(cargo --version)"
              echo "  bun  : $(bun --version)"
            '';
          };
        });
    };
}
