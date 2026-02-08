{ pkgs }:

pkgs.writeShellScriptBin "pg_log" ''
  set -euo pipefail
  tail -n 200 -f "$PWD/.pg/logfile"
''
