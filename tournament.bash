#!/bin/bash
# Script to run chess engine tournaments with any number of engines

# Check if at least one engine path is provided
if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <executable_path_1> [executable_path_2] [executable_path_3] ..."
    echo "At least one engine must be provided."
    exit 1
fi

# Detect the number of available CPU cores
if [ -f /proc/cpuinfo ]; then
    # Linux
    available_cpus=$(grep -c ^processor /proc/cpuinfo)
elif [ "$(uname)" == "Darwin" ]; then
    # macOS
    available_cpus=$(sysctl -n hw.ncpu)
else
    # Default to 1 if we can't detect
    available_cpus=1
fi

# Set concurrency to 75% of available CPUs, minimum 1
concurrency=$(( (available_cpus * 3) / 4 ))
[ $concurrency -lt 1 ] && concurrency=1

echo "Detected $available_cpus CPU cores, setting concurrency to $concurrency"

# Prepare engine parameters for fastchess
engine_params=""
engine_count=0

# Process each engine path
for engine_path in "$@"; do
    # Extract the executable name from the path
    engine_name=$(basename "$engine_path")
    engine_count=$((engine_count+1))
    
    # Check if this engine name has already been used
    # If there are duplicates, add numbers to make them unique
    duplicate_count=0
    for ((i=1; i<engine_count; i++)); do
        if [ "$engine_name" = "$(basename "${!i}")" ]; then
            duplicate_count=$((duplicate_count+1))
        fi
    done
    
    # If it's a duplicate, append a number
    if [ $duplicate_count -gt 0 ]; then
        engine_name="${engine_name}_$((duplicate_count+1))"
    fi
    
    # Add this engine to the parameters
    engine_params="$engine_params -engine cmd=\"$engine_path\" name=\"$engine_name\""
done

# Clean up any existing log files
rm -rf fastchess_log

# Build the full command
# Use eval to properly handle the double quotes in engine_params
eval "fastchess $engine_params \
      -openings file=Pohl.epd format=epd order=random \
      -each tc=10+0.1 -rounds 100 -repeat -concurrency $concurrency -log file=fastchess_log engine=true"

#!/bin/bash
# Script to run chess engine tournaments with any number of engines and optional core control

# Default: no core override
user_cores=""

# Check for --cores option
for arg in "$@"; do
    if [[ "$arg" =~ ^--cores=([0-9]+)$ ]]; then
        user_cores="${BASH_REMATCH[1]}"
        # Remove --cores option from arguments
        set -- "${@/--cores=$user_cores/}"
        break
    fi
done

# Check if at least one engine path is provided
if [ "$#" -lt 1 ]; then
    echo "Usage: $0 [--cores=N] <executable_path_1> [executable_path_2] ..."
    echo "At least one engine must be provided."
    exit 1
fi

# Detect the number of available CPU cores
if [ -f /proc/cpuinfo ]; then
    available_cpus=$(grep -c ^processor /proc/cpuinfo)
elif [ "$(uname)" == "Darwin" ]; then
    available_cpus=$(sysctl -n hw.ncpu)
else
    available_cpus=1
fi

# Set concurrency
if [ -n "$user_cores" ]; then
    concurrency=$user_cores
else
    concurrency=$(( (available_cpus * 3) / 4 ))
    [ $concurrency -lt 1 ] && concurrency=1
fi

echo "Detected $available_cpus CPU cores, using concurrency: $concurrency"

# Prepare engine parameters for fastchess
engine_params=""
engine_count=0

for engine_path in "$@"; do
    engine_name=$(basename "$engine_path")
    engine_count=$((engine_count+1))
    
    duplicate_count=0
    for ((i=1; i<engine_count; i++)); do
        if [ "$engine_name" = "$(basename "${!i}")" ]; then
            duplicate_count=$((duplicate_count+1))
        fi
    done
    
    if [ $duplicate_count -gt 0 ]; then
        engine_name="${engine_name}_$((duplicate_count+1))"
    fi
    
    engine_params="$engine_params -engine cmd=\"$engine_path\" name=\"$engine_name\""
done

rm -rf fastchess_log

eval "fastchess $engine_params \
      -openings file=Pohl.epd format=epd order=random \
      -each tc=10+0.1 -rounds 100 -repeat -concurrency $concurrency -log file=fastchess_log engine=true"

