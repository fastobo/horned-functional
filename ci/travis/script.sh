#!/bin/sh -e

. $(dirname $0)/functions.sh

# --- Test with coverage -----------------------------------------------------

log Measuring code coverage through Tarpaulin with default features
cargo tarpaulin -v --out Xml --ciserver travis-ci
