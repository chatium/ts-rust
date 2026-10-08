#!/usr/bin/env bash
# zbook side of the build-time benchmark: holds the host lock, runs one buildbench-plan.sh
# session there and fetches its results to target/continuation-r97-goport/buildspeed/runs/<host>-<session>/.
# The setup session also copies this worktree's source (not target/ or .git) to the host.
# usage: buildbench-remote.sh <host> <session>
# The last line is DONE or FAIL rc=N. Run sessions over 10 minutes under systemd-run --user --collect.
set -uo pipefail
HOST=${1:?host} S=${2:?session}
REPO=/home/theo/Code/sandbox/ts-rust
WT=$(cd "$(dirname "$0")/../.." && pwd)
OUT=$REPO/target/continuation-r97-goport/buildspeed/runs/$HOST-$S
case $HOST in dbook-lan) lock=/tmp/goport-remote-dbook.lock ;; *) lock=/tmp/goport-remote-$HOST.lock ;; esac
case $HOST in dbook-lan) envs='export RUSTUP_HOME=/home/dbook/.rustup CARGO_HOME=/home/dbook/.cargo;' ;; *) envs= ;; esac
exec 7> "$lock"
flock 7
echo "lock $lock taken $(date -Is)"
rc=0
# Same LAN route and host key as remote.sh.
ssh_opts=()
case $HOST in
  mini-743d) ssh_opts=(-e "ssh -o HostName=mini-743d.local -o HostKeyAlias=$(ssh -G mini-743d 2> /dev/null | awk '$1 == "hostname" { print $2 }')") ;;
  mini-abf9) echo "mini-abf9 is Theo's machine since 2026-10-04. Do not use it." >&2; exit 2 ;;
esac
if [[ $S == setup ]]; then
  "$REPO/scripts/goport/remote.sh" sync-scripts "$HOST" > /dev/null || rc=$?
  rsync -a --delete --mkpath "${ssh_opts[@]}" --exclude=/target/ --exclude=/.git "$WT/" "$HOST:$WT/" || rc=$?
fi
# The bench tools and the cargo wrapper can change between sessions (source files are not copied again).
rsync -a "${ssh_opts[@]}" "$WT/scripts/goport/" "$HOST:$WT/scripts/goport/" || rc=$?
rsync -a "${ssh_opts[@]}" "$WT/scripts/run-cargo-capped.sh" "$HOST:$WT/scripts/run-cargo-capped.sh" || rc=$?
envs+=" export BENCH_COMMIT=$(git -C "$WT" rev-parse HEAD);"
"$REPO/scripts/goport/remote.sh" run "$HOST" "$envs $WT/scripts/goport/buildbench-plan.sh $S $OUT" || rc=$?
"$REPO/scripts/goport/remote.sh" fetch "$HOST" "$OUT" > /dev/null || rc=$?
echo "lock released $(date -Is)"
((rc == 0)) && echo DONE || echo "FAIL rc=$rc"
exit $rc
