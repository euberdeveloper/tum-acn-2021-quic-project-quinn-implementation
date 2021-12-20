#!/bin/bash

SCRIPTDIR=`dirname "$(readlink -f "$0")"`

cd ${SCRIPTDIR}/quic-implementation
cargo build --bins --release

# zip -R artifact.zip quic-implementation run-client.sh run-server.sh setup-env.sh