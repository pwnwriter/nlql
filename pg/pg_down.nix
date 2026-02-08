{ pkgs }:

pkgs.writeShellScriptBin "pg_down" ''
  set -euo pipefail

  export PGDATA="$PWD/.pg"

  if pg_ctl -D "$PGDATA" status >/dev/null 2>&1; then
    echo "Stopping Postgres..."
    pg_ctl -D "$PGDATA" stop -m fast
    echo "Postgres stopped."
  else
    echo "Postgres not running."
  fi
''
