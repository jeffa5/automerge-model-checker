#!/usr/bin/env bash

cargo build --release

results_dir=results
rm -rf $results_dir
mkdir -p $results_dir

function info() {
  echo "=========== $@ ============="
}

function run() {
  bin=$1
  shift
  filename="${bin}_$@"
  filename="${filename// /_}"
  out="$results_dir/$filename"
  cmd="cargo run -q --release --bin $bin -- $@ | tee $out"
  info $cmd
  eval $cmd
}

info counter
run amc-counter check-iterative
run amc-counter check-iterative --counter-type
run amc-counter check-iterative --initial-change
run amc-counter check-iterative --initial-change --counter-type

info moves
run amc-moves check-iterative

info todos
run amc-todo check-iterative
run amc-todo check-iterative --random-ids
run amc-todo check-iterative --initial-change
run amc-todo check-iterative --initial-change --random-ids

info automerge
run amc-automerge check-iterative
