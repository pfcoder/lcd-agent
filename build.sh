#!/bin/bash

cargo build --release
cp target/release/omni-gpu-agent ./res/omni-gpu-agent/
tar -cvzf ./res/omni-gpu-agent/machine.tgz ./res/machine