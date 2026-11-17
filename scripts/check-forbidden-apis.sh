#!/usr/bin/env bash
# Fails if production code outside the egress/test crates opens outbound sockets
# or spawns processes that could (ADR-0006, invariant I1/I10; security review
# finding 14). The deny.toml ban only covers HTTP client *crates*; this catches
# raw sockets in std/tokio. Test code (tests/, #[cfg(test)] modules in testkit)
# and crates/aegis-egress and crates/aegis-testkit are exempt.
set -euo pipefail
cd "$(dirname "$0")/.."

pattern='TcpStream::connect|UdpSocket|TcpSocket|std::process::Command|tokio::process|\.connect\(|ToSocketAddrs'
status=0
while IFS= read -r file; do
  case "$file" in
    crates/aegis-egress/*|crates/aegis-testkit/*|*/tests/*) continue ;;
  esac
  if matches=$(grep -nE "$pattern" "$file" | grep -v '^\s*//' || true); [ -n "$matches" ]; then
    echo "forbidden network/process API in $file:"
    echo "$matches"
    status=1
  fi
done < <(find crates -name '*.rs' -not -path '*/target/*')

if [ "$status" -ne 0 ]; then
  echo "Outbound sockets and process spawning belong only in aegis-egress (ADR-0006)." >&2
fi
exit "$status"
