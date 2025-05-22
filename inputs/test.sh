#!/bin/bash

# exit when any command fails
set -e

ALL=
for file in ./inputs/*.dot; do
  NAME=/tmp/out_$RANDOM.svg
  ALL="$NAME $ALL"
  cargo run --bin layout $file -o $NAME $1 $2 $3
  NAME=/tmp/out_$RANDOM.svg
  ALL="$NAME $ALL"
  dot -Tsvg $file -o $NAME
done
echo $ALL | xargs firefox &
