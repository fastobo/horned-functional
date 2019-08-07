#!/bin/sh -e

. $(dirname $0)/functions.sh

# --- Test with coverage -----------------------------------------------------

log Measuring code coverage in debug mode
cargo tarpaulin -v --out Xml --ciserver travis-ci
