{ pkgs }:

pkgs.writeShellScriptBin "pg_up" ''
  set -euo pipefail

  export PGDATA="$PWD/.pg"
  export PGHOST="$PGDATA"
  export PGPORT=5432

  mkdir -p "$PGDATA"

  if [ ! -f "$PGDATA/PG_VERSION" ]; then
    echo "Initializing database..."
    initdb -D "$PGDATA" --auth=trust >/dev/null
  fi

  if pg_ctl -D "$PGDATA" status >/dev/null 2>&1; then
    echo "Postgres already running."
    exit 0
  fi

  echo "Starting Postgres..."
  pg_ctl -D "$PGDATA" -l "$PGDATA/logfile" start \
    -o "-c listen_addresses= -k $PGDATA" \
    -w -t 10

  echo "Postgres started!"
''
