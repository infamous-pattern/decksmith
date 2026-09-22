#!/usr/bin/env bash
set -euo pipefail
repo_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
if [[ -f "$repo_dir/release.json" ]]; then
  case "${1:-status}" in
    start|stop|status) exec systemctl --user "${1:-status}" decksmith.service ;;
    logs) exec journalctl --user -u decksmith.service -n 40 --no-pager ;;
    *) echo 'Use start, stop, status or logs.' >&2; exit 2 ;;
  esac
fi
case "${1:-status}" in
  start)
    if systemctl --user is-active --quiet decksmith.service; then
      echo "Decksmith is already running."
      exit 0
    fi
    if [[ ! -x "$repo_dir/target/debug/decksmithd" ]]; then
      echo "Build decksmithd with the hardware feature first." >&2
      exit 1
    fi
    systemctl --user reset-failed decksmith.service 2>/dev/null || true
    systemd-run --user --unit=decksmith --description='Decksmith device controls' \
      --property=Type=exec --property=Restart=no --property=TimeoutStopSec=5s \
      --working-directory="$repo_dir" \
      "$repo_dir/target/debug/decksmithd" --config "$repo_dir/config/audio.json" --exclusive --run
    ;;
  stop) systemctl --user stop decksmith.service ;;
  status) systemctl --user status --no-pager decksmith.service ;;
  logs) journalctl --user -u decksmith.service -n 40 --no-pager ;;
  *) echo "Usage: $0 start|stop|status|logs" >&2; exit 2 ;;
esac
