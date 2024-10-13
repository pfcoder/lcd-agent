#!/bin/bash

# Path to the log file
log_file="/opt/aleo_prover/prover.log"

# Get the last 100 lines of the log file
last_100_lines=$(tail -n 100 "$log_file")

# Initialize variables to store the last matching line
last_timestamp=""
gpu_info_list=()

# Process each line of the last 100 lines
while IFS= read -r line; do
    # Extract timestamp
    if [[ $line =~ ^\|[[:space:]]*([0-9T:-]+)[[:space:]]*\| ]]; then
        last_timestamp="${BASH_REMATCH[1]}"
        gpu_info_list=()  # Reset GPU info list when a new timestamp is found
    fi

    # Extract GPU information
    if [[ $line =~ ^\|[[:space:]]*gpu\[([0-9\*]+)\]:[[:space:]]*\(1m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*5m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*15m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*30m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*60m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*\)[[:space:]]*\| ]]; then
        gpu_info_list+=("$line")
    fi
done <<< "$last_100_lines"

# Check if we found any GPU information after the last timestamp
if [ -n "$last_timestamp" ] && [ ${#gpu_info_list[@]} -gt 0 ]; then
    # Initialize JSON output
    json_output="["

    # Process each GPU information line
    for gpu_info in "${gpu_info_list[@]}"; do
        if [[ $gpu_info =~ ^\|[[:space:]]*gpu\[([0-9\*]+)\]:[[:space:]]*\(1m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*5m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*15m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*30m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*60m[[:space:]]*-[[:space:]]*([0-9]+)[[:space:]]*\)[[:space:]]*\| ]]; then
            gpu_index="${BASH_REMATCH[1]}"
            one_min="${BASH_REMATCH[2]}"
            five_min="${BASH_REMATCH[3]}"
            fifteen_min="${BASH_REMATCH[4]}"
            thirty_min="${BASH_REMATCH[5]}"
            sixty_min="${BASH_REMATCH[6]}"

            # Append JSON object to the output
            json_output+=$(jq -nc \
                --arg timestamp "$last_timestamp" \
                --arg gpu_index "$gpu_index" \
                --arg one_min "$one_min" \
                --arg five_min "$five_min" \
                --arg fifteen_min "$fifteen_min" \
                --arg thirty_min "$thirty_min" \
                --arg sixty_min "$sixty_min" \
                '{timestamp: $timestamp, gpu_index: $gpu_index, one_min: $one_min, five_min: $five_min, fifteen_min: $fifteen_min, thirty_min: $thirty_min, sixty_min: $sixty_min}')
            json_output+=","
        fi
    done

    # Remove trailing comma and close JSON array
    json_output="${json_output%,}]"

    # Output JSON
    echo "$json_output"
else
    # echo empty json
    echo "[]"
fi