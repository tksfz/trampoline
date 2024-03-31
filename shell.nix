# shell.nix
let
  pkgs = import <nixpkgs> {};
in
  pkgs.mkShell {
    packages = [
      pkgs.cargo
      pkgs.rustc

      pkgs.rust-analyzer
      pkgs.rustfmt

      # If the dependencies need system libs, you usually need pkg-config + the lib
      pkgs.pkg-config
      pkgs.openssl

      pkgs.protobuf

      pkgs.minikube
      pkgs.kubernetes-helm
      pkgs.k9s

      pkgs.vscode
    ];

    env = {
      RUST_BACKTRACE = "full";
    };

  }



