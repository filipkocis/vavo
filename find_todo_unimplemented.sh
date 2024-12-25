#!/bin/bash

# Script to find occurrences of "TODO" and "UNIMPLEMENTED" in a codebase

# Initialize counters
todo_count=0
unimplemented_count=0

# Find and display occurrences of "TODO"
echo "Searching for 'TODO' in the codebase..."
grep -rniw './src/' -e 'TODO' --include=\*.* | tee temp_todo_results.txt
todo_count=$(wc -l < temp_todo_results.txt)
echo ""

# Find and display occurrences of "UNIMPLEMENTED"
echo "Searching for 'UNIMPLEMENTED' in the codebase..."
grep -rniw './src/' -e 'UNIMPLEMENTED' --include=\*.* | tee temp_unimplemented_results.txt
unimplemented_count=$(wc -l < temp_unimplemented_results.txt)
echo ""

# Display counts
echo "Summary:"
echo "TODO count: $todo_count"
echo "UNIMPLEMENTED count: $unimplemented_count"

# Clean up temporary files
rm -f temp_todo_results.txt temp_unimplemented_results.txt
