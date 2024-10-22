#!/bin/bash
cd /opt/omni-gpu-agent/
nohup ./omni-gpu-agent  > /var/log/omni-gpu-agent.log 2>&1 &
echo $! > /opt/omni-gpu-agent/pid
