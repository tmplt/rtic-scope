with import <nixpkgs> {};
mkShell {
  buildInputs = [
    pkg-config
    pkgs.libusb

    # Latest tagged release is from 2017, which lacks some scripts I need.
    (pkgs.openocd.overrideAttrs (old: {
      src = fetchgit {
        url = "https://git.code.sf.net/p/openocd/code";
        rev = "7c88e76a76588fa0e3ab645adfc46e8baff6a3e4";
        sha256 = "0qli4zyqc8hvbpkhwscsfphk14sdaa1zxav4dqpvj21kgqxnbjr8";
        fetchSubmodules = false; # available in nixpkgs
      };

      # no longer applies
      patches = [];
      postPatch = ''
          ${gnused}/bin/sed -i "s/\''${libtoolize}/libtoolize/g" ./bootstrap
          ${gnused}/bin/sed -i '7,14d' ./bootstrap
        '';

      buildInputs = old.buildInputs ++ [ automake autoconf m4 libtool tcl jimtcl ];

      preConfigure = ''
        ./bootstrap nosubmodule
      '';
      configureFlags = old.configureFlags ++ [
        "--disable-internal-jimtcl"
        "--disable-internal-libjaylink"
      ];
      buildPhase = ''
        make CFLAGS="-g -O0"
      '';
    }))
  ];
  LD_LIBRARY_PATH="${stdenv.cc.cc.lib}/lib64:$LD_LIBRARY_PATH";
}
