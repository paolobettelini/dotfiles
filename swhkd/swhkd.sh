#!/usr/bin/env bash

# Start the swhkd deamon
#
# $HOME/.config/swhkd/swhkd.sh

killall swhks

swhks & pkexec swhkd