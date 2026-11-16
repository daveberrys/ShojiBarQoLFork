#!/bin/sh

if [ "$#" -eq 0 ]; then
    echo "usage: restart.sh PROGRAM [ARG ...]" >&2
    exit 2
fi

while :; do
    "$@"
    status=$?
    if [ "$status" -eq 0 ]; then
        exit 0
    fi

    echo "$@: process exited with status $status; restarting in 1 second" >&2
    sleep 1
done
