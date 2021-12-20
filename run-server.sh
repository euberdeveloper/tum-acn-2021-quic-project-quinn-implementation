#!/bin/bash

SCRIPTDIR=`dirname "$(readlink -f "$0")"`

${SCRIPTDIR}/quic-implementation/target/release/server > ${LOGS}/log.txt 2>&1

retVal=$?
if [ $retVal -eq 127 ]; then
    echo "exited with code 127"
fi
