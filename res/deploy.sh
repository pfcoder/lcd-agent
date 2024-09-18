#!/bin/bash

#check input
if [ $# -ne 1 ]; then
    echo "Usage: $0 [API TOKEN]"
    exit 1
fi

API_TOKEN=$1

mkdir -p /opt/omni-agent
cp start.sh /opt/omni-agent/
cp stop.sh /opt/omni-agent/
cp lcd-agent.service /etc/systemd/system/
cp release/lcd-agent /opt/omni-agent/
chmod a+x /opt/omni-agent/start.sh
chmod a+x /opt/omni-agent/stop.sh

echo "AGENT_TOKEN=$API_TOKEN" > /opt/omni-agent/.env

systemctl daemon-reload
systemctl enable lcd-agent
systemctl start lcd-agent
