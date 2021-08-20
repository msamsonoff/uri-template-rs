#! /bin/sh
set -ex
exec docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin sh -c 'cargo build && cargo tarpaulin --out Html'
