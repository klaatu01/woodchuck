#!/bin/bash
help_string="requires destination. build.sh loggly|custom x86_64|arm64"
if [ -d $1 ]; then
  echo $help_string
  exit 1
fi
if [ -d $2 ]; then
  echo $help_string
  exit 1
fi

woodchuck_name="woodchuck_$1_$2"

mkdir extensions

case $2 in
  "x86_64")
    docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --features "$1" --release && 
    cp target/x86_64-unknown-linux-musl/release/woodchuck extensions/$woodchuck_name
    ;;
  "arm64")
    docker run --rm -it -v "$(pwd)":/home/rust/src messense/rust-musl-cross:aarch64-musl cargo build --features "$1 $2" --release && 
    cp target/aarch64-unknown-linux-musl/release/woodchuck extensions/$woodchuck_name
    ;;
esac

zip -r extensions.zip extensions
rm -rf extensions
