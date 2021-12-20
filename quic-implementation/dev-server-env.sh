#!/bin/bash

sudo export SSLKEYLOGFILE=./tmp/ssl_key_log
sudo export QLOGDIR=./tmp/qlog
sudo export LOGS=./tmp/logs
sudo export TESTCASE=handshake
sudo export WWW=./www
sudo export CERTS=./certs
sudo export IP=127.0.0.1
sudo export PORT=443