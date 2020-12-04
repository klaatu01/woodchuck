#!/bin/bash
if [ -d $1 ]; then
  echo "requires destination. build.sh loggly|custom"
  exit 1
fi
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder'
rust-musl-builder cargo build --features $1 --release && 
mkdir extensions &&
cp target/x86_64-unknown-linux-musl/release/woodchuck extensions/woodchuck
zip -r extensions.zip extensions &&
rm -r extensions
