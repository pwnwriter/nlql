{ pkgs }:

pkgs.writeShellScriptBin "pg_status" ''
  set -euo pipefail

  export PGDATA="$PWD/.pg"

  if pg_ctl -D "$PGDATA" status >/dev/null 2>&1; then
    echo "Postgres is running."
    exit 0
  else
    echo "Postgres is NOT running."
    exit 1
  fi
''
