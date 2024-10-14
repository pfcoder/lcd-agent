#!/bin/bash

# Check input
if [ $# -ne 1 ]; then
    echo "Usage: $0 [API TOKEN]"
    exit 1
fi

API_TOKEN=$1
RUN_DIR="/opt/omni-gpu-agent"
EXE_NAME="omni-gpu-agent"

# Create the run directory if it doesn't exist
mkdir -p "$RUN_DIR" || { echo "Failed to create directory $RUN_DIR"; exit 1; }

# Copy necessary files
cp start.sh "$RUN_DIR/" || { echo "Failed to copy start.sh"; exit 1; }
cp stop.sh "$RUN_DIR/" || { echo "Failed to copy stop.sh"; exit 1; }
cp "$EXE_NAME.service" /etc/systemd/system/ || { echo "Failed to copy $EXE_NAME.service"; exit 1; }
cp "release/$EXE_NAME" "$RUN_DIR/" || { echo "Failed to copy $EXE_NAME"; exit 1; }

# Make scripts executable
chmod a+x "$RUN_DIR/start.sh" || { echo "Failed to make start.sh executable"; exit 1; }
chmod a+x "$RUN_DIR/stop.sh" || { echo "Failed to make stop.sh executable"; exit 1; }

# Create environment variable file
echo "AGENT_TOKEN=$API_TOKEN" > "$RUN_DIR/.env" || { echo "Failed to create .env file"; exit 1; }

# Reload systemd, enable and start the service
systemctl daemon-reload || { echo "Failed to reload systemd"; exit 1; }
systemctl enable "$EXE_NAME" || { echo "Failed to enable $EXE_NAME service"; exit 1; }
systemctl start "$EXE_NAME" || { echo "Failed to start $EXE_NAME service"; exit 1; }

echo "Deployment completed successfully."