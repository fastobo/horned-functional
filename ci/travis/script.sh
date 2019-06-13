#!/bin/sh -e

. $(dirname $0)/functions.sh

# --- Test -------------------------------------------------------------------

log Testing code in debug mode
cargo test

# --- Test with coverage -----------------------------------------------------

log Measuring code coverage in debug mode
cargo tarpaulin -v --out Xml --ciserver travis-ci
