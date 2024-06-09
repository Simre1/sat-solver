#!/bin/bash


# Write the header to the output file
echo "file,simple,dpll,cdcl"

# Process the input file and append to the output file
awk '{
    # Extract file name and remove the .in extension
    split($1, file_parts, "/");
    sub(".in", "", file_parts[4]);
    file = file_parts[4];

    # Extract algorithm and time
    algorithm = $2;
    time = $3;

    # Store time for each algorithm in associative array
    times[file,algorithm] = time;

    # If all times for a file are collected, print them
    if (times[file,"simple"] != "" && times[file,"dpll"] != "" && times[file,"cdcl"] != "") {
        printf "%s,%s,%s,%s\n", file, times[file,"simple"], times[file,"dpll"], times[file,"cdcl"];
    }
}'
