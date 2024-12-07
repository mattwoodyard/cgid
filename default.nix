let pkgs = import <nixpkgs> {}; in

pkgs.rustPlatform.buildRustPackage {
  pname = "monorepo";
  version = "0.0.1";
  src = ./.;
  buildInputs = with pkgs; [
	openssl
  ];

     nativeBuildInputs = with pkgs; [
        pkg-config
    ];

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

}
