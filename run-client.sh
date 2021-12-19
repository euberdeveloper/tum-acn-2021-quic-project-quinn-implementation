#!/bin/bash

SCRIPTDIR=`dirname "$(readlink -f "$0")"`

cd ${SCRIPTDIR}/quic-implementation
cargo run