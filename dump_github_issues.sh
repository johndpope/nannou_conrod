#!/bin/bash

# dump_github_issues.sh - Dump all GitHub issues to a concatenated text file
# Usage: ./dump_github_issues.sh [output_file]

set -e

# Output file (default: github_issues_dump.txt)
OUTPUT_FILE="${1:-github_issues_dump.txt}"

# Colors for terminal output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Fetching GitHub issues...${NC}"

# Get all open issues (increase limit if needed)
ISSUE_NUMBERS=$(gh issue list --state all --limit 100 --json number --jq '.[].number' | sort -n)

# Clear or create output file
> "$OUTPUT_FILE"

# Header
echo "GitHub Issues Dump - RDT Project" >> "$OUTPUT_FILE"
echo "Generated on: $(date)" >> "$OUTPUT_FILE"
echo "Repository: $(gh repo view --json nameWithOwner --jq .nameWithOwner)" >> "$OUTPUT_FILE"
echo "==========================================" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Counter
COUNT=0

# Process each issue
for ISSUE_NUM in $ISSUE_NUMBERS; do
    echo -e "${GREEN}Processing issue #$ISSUE_NUM...${NC}"
    
    # Add separator
    echo "==========================================" >> "$OUTPUT_FILE"
    echo "ISSUE #$ISSUE_NUM" >> "$OUTPUT_FILE"
    echo "==========================================" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    # Get issue details with body and comments
    gh issue view $ISSUE_NUM >> "$OUTPUT_FILE"
    
    # Add extra spacing between issues
    echo "" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    
    COUNT=$((COUNT + 1))
done

echo -e "${GREEN}✓ Successfully dumped $COUNT issues to $OUTPUT_FILE${NC}"
echo -e "${BLUE}File size: $(ls -lh "$OUTPUT_FILE" | awk '{print $5}')${NC}"