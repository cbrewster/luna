{ pkgs }: {
	deps = [
		pkgs.nodejs-16_x
        pkgs.nodePackages.typescript-language-server
        pkgs.nodePackages.yarn
        pkgs.replitPackages.jest
        pkgs.cargo
        pkgs.rustc
        pkgs.rust-analyzer
        pkgs.rustfmt
        pkgs.python3
	];
}