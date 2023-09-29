#!/bin/bash

cargo build --release && \
    cat ../relays.json | target/release/relay-ranker
