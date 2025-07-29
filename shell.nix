# default.nix
with import <nixpkgs> {};
  stdenv.mkDerivation {
    name = "dev-environment"; # Probably put a more meaningful name here
    buildInputs = [gcc gnumake openssl pkg-config libopus yt-dlp];
  }
