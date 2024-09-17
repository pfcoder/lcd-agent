#!/bin/bash

# Define the API endpoint
API_ENDPOINT="http://localhost/api/info"

# Use curl to send a GET request to the API endpoint
response=$(curl -s $API_ENDPOINT)
sn=$(echo $response | grep -Po '"sn":"\K[^"]+')
hash=$(echo $response | grep -Po '"hash":"\K[^"]+')

temp=$(sensors | grep -oP 'Core .: *\+\K[0-9.]+' | tr '\n' ' ')
avg=$(echo $temp | tr ' ' '\n' | awk '{sum+=$1} END {print sum/NR}')

hdds=$(smartctl -a /dev/sda | grep "Serial Number:" | awk '{print $3}')
hddt=$(smartctl -a /dev/sda | grep "Temperature_Celsius" | awk '{print $10}')

#cpu_usage=$(mpstat -P ALL 1 1 | awk '/Average/ && $2 ~ /[0-9]+/ {print $3}')

# Convert to comma-separated format
#cpu_usage_csv=$(echo "$cpu_usage" | paste -sd "," -)

total_cpu_usage=$(mpstat 1 1 | awk '/Average/ && $3 ~ /[0-9.]+/ {print $3}')

cpu_model=$(lscpu | grep "Model name:" | awk -F: '{print $2}' | xargs)

echo '{"sn": "'"$sn"'", "hash": "'"$hash"'", "temp_sys": "'"$avg"'", "hdd_sn": "'"$hdds"'", "temp_hdd":"'"$hddt"'", "cpu_occupy": "'"$total_cpu_usage"'", "cpu_model": "'"$cpu_model"'"}'