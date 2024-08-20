{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {

  name = "backend";
  RUSTC_VERSION = "stable";

  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
    export DATABASE_URL="postgres://backend:37O*2p78>Itg~Fh69}y~£V|@localhost/backend"
    '';

  packages = with pkgs; [
    rustup
    clang
    diesel-cli
    dbeaver-bin
  ];
}


