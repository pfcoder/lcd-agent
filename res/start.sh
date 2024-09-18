#!/bin/bash

nohup /opt/omni-agent/lcd-agent  > /var/log/omni-agent.log 2>&1 &
echo $! > /opt/omni-agent/pid