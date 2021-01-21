with import <nixpkgs> {};
mkShell {
  buildInputs = [
    pkg-config
    pkgs.libusb
  ];
  LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib64:$LD_LIBRARY_PATH";
}
