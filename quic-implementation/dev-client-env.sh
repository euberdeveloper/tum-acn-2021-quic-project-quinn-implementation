#!/bin/bash

export SSLKEYLOGFILE=./tmp/ssl_key_log
export QLOGDIR=./tmp/qlog
export LOGS=./tmp/logs
export TESTCASE=handshake
export DOWNLOADS=./downloads
export REQUESTS="https://localhost:4433/index.html https://127.0.0.2:1337/asdASGaASD https://[::1]:23344/priv.key"