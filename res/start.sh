#!/bin/bash
cd /opt/omni-agent/
nohup ./lcd-agent  > /var/log/omni-agent.log 2>&1 &
echo $! > /opt/omni-agent/pid
