{ pkgs }:

let
  env = import ./env.nix { };

  pg_up = import ./pg_up.nix { inherit pkgs; };
  pg_down = import ./pg_down.nix { inherit pkgs; };
  pg_status = import ./pg_status.nix { inherit pkgs; };
  pg_log = import ./pg_log.nix { inherit pkgs; };
in
pkgs.mkShell {
  packages = with pkgs; [
    postgresql
    sqlite
    pkg-config

    pg_up
    pg_down
    pg_status
    pg_log
  ];

  shellHook = ''
    ${env}

    echo "Dev shell ready."
    echo "Commands:"
    echo "  pg_up      - init + start postgres"
    echo "  pg_down    - stop postgres"
    echo "  pg_status  - status"
    echo "  pg_log     - tail logs"
  '';
}
