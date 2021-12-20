#!/bin/bash

export SSLKEYLOGFILE=./tmp/ssl_key_log
export QLOGDIR=./tmp/qlog
export LOGS=./tmp/logs
export TESTCASE=handshake
export DOWNLOADS=./downloads
export REQUESTS="https://localhost:443/index.html https://127.0.0.1:443/asdASGaASD https://[::1]:433/priv.key"