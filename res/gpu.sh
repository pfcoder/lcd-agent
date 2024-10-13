#!/bin/bash

# Get GPU information using nvidia-smi
gpu_info=$(nvidia-smi --query-gpu=index,name,power.draw,temperature.gpu --format=csv,noheader,nounits)

# Initialize JSON output
json_output="["

# Process each line of GPU information
while IFS=, read -r index name power temperature; do
    # Trim leading and trailing whitespace
    index=$(echo "$index" | xargs)
    name=$(echo "$name" | xargs)
    power=$(echo "$power" | xargs)
    temperature=$(echo "$temperature" | xargs)
    
    json_output+=$(jq -n \
        --arg index "$index" \
        --arg name "$name" \
        --arg power "$power" \
        --arg temperature "$temperature" \
        '{index: $index, name: $name, power: $power, temperature: $temperature}')
    json_output+=","
done <<< "$gpu_info"

# Remove trailing comma and close JSON array
json_output="${json_output%,}]"

# Output JSON
echo "$json_output" | jq .
