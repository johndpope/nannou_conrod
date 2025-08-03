#!/bin/bash

# dump_github_issues_formatted.sh - Advanced GitHub issues dumper with formatting
# Usage: ./dump_github_issues_formatted.sh [options]

set -e

# Default values
OUTPUT_FILE="github_issues_dump.txt"
FORMAT="text"  # text, markdown, or json
INCLUDE_COMMENTS=true
STATE="all"  # all, open, closed

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

# Help function
show_help() {
    echo "GitHub Issues Dumper"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -o, --output FILE      Output file (default: github_issues_dump.txt)"
    echo "  -f, --format FORMAT    Output format: text, markdown, json (default: text)"
    echo "  -s, --state STATE      Issue state: all, open, closed (default: all)"
    echo "  --no-comments          Exclude comments"
    echo "  -h, --help            Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 -o issues.md -f markdown"
    echo "  $0 --state open --no-comments"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        -f|--format)
            FORMAT="$2"
            shift 2
            ;;
        -s|--state)
            STATE="$2"
            shift 2
            ;;
        --no-comments)
            INCLUDE_COMMENTS=false
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Adjust file extension based on format
if [[ "$FORMAT" == "markdown" && "$OUTPUT_FILE" == "github_issues_dump.txt" ]]; then
    OUTPUT_FILE="github_issues_dump.md"
elif [[ "$FORMAT" == "json" && "$OUTPUT_FILE" == "github_issues_dump.txt" ]]; then
    OUTPUT_FILE="github_issues_dump.json"
fi

echo -e "${BLUE}Fetching GitHub issues (state: $STATE)...${NC}"

# Function to format issue as text
format_issue_text() {
    local issue_json="$1"
    local issue_num=$(echo "$issue_json" | jq -r '.number')
    local title=$(echo "$issue_json" | jq -r '.title')
    local state=$(echo "$issue_json" | jq -r '.state')
    local author=$(echo "$issue_json" | jq -r '.author.login')
    local created=$(echo "$issue_json" | jq -r '.createdAt')
    local labels=$(echo "$issue_json" | jq -r '.labels[].name' | tr '\n' ', ' | sed 's/,$//')
    local body=$(echo "$issue_json" | jq -r '.body // "No description"')
    
    echo "==========================================
ISSUE #$issue_num: $title
==========================================
State: $state
Author: $author
Created: $created
Labels: $labels

Description:
$body"
    
    if [[ "$INCLUDE_COMMENTS" == "true" ]]; then
        echo -e "\nComments:"
        echo "----------"
        
        # Get comments
        gh issue view $issue_num --comments --json comments | \
        jq -r '.comments[] | "
Author: \(.author.login)
Date: \(.createdAt)
\(.body)
---"'
    fi
}

# Function to format issue as markdown
format_issue_markdown() {
    local issue_json="$1"
    local issue_num=$(echo "$issue_json" | jq -r '.number')
    local title=$(echo "$issue_json" | jq -r '.title')
    local state=$(echo "$issue_json" | jq -r '.state')
    local author=$(echo "$issue_json" | jq -r '.author.login')
    local created=$(echo "$issue_json" | jq -r '.createdAt')
    local labels=$(echo "$issue_json" | jq -r '.labels[].name' | tr '\n' ', ' | sed 's/,$//')
    local body=$(echo "$issue_json" | jq -r '.body // "No description"')
    
    echo "## Issue #$issue_num: $title

**State:** $state  
**Author:** @$author  
**Created:** $created  
**Labels:** $labels

### Description

$body"
    
    if [[ "$INCLUDE_COMMENTS" == "true" ]]; then
        echo -e "\n### Comments\n"
        
        # Get comments
        gh issue view $issue_num --comments --json comments | \
        jq -r '.comments[] | "
**@\(.author.login)** - _\(.createdAt)_

\(.body)

---"'
    fi
}

# Main processing
case "$FORMAT" in
    text)
        # Clear or create output file
        > "$OUTPUT_FILE"
        
        # Header
        echo "GitHub Issues Dump - RDT Project
Generated on: $(date)
Repository: $(gh repo view --json nameWithOwner --jq .nameWithOwner)
State Filter: $STATE
==========================================
" > "$OUTPUT_FILE"
        
        # Get all issues
        ISSUES=$(gh issue list --state "$STATE" --limit 100 --json number,title,state,author,createdAt,labels,body)
        COUNT=$(echo "$ISSUES" | jq 'length')
        
        echo "$ISSUES" | jq -c '.[]' | while read -r issue; do
            issue_num=$(echo "$issue" | jq -r '.number')
            echo -e "${GREEN}Processing issue #$issue_num...${NC}"
            
            format_issue_text "$issue" >> "$OUTPUT_FILE"
            echo -e "\n\n" >> "$OUTPUT_FILE"
        done
        ;;
        
    markdown)
        # Header
        echo "# GitHub Issues Dump - RDT Project

**Generated on:** $(date)  
**Repository:** $(gh repo view --json nameWithOwner --jq .nameWithOwner)  
**State Filter:** $STATE

---
" > "$OUTPUT_FILE"
        
        # Table of contents
        echo "## Table of Contents
" >> "$OUTPUT_FILE"
        
        ISSUES=$(gh issue list --state "$STATE" --limit 100 --json number,title)
        echo "$ISSUES" | jq -r '.[] | "- [Issue #\(.number): \(.title)](#issue-\(.number)-\(.title | ascii_downcase | gsub("[^a-z0-9]+"; "-")))"' >> "$OUTPUT_FILE"
        
        echo -e "\n---\n" >> "$OUTPUT_FILE"
        
        # Get full issue data
        ISSUES=$(gh issue list --state "$STATE" --limit 100 --json number,title,state,author,createdAt,labels,body)
        COUNT=$(echo "$ISSUES" | jq 'length')
        
        echo "$ISSUES" | jq -c '.[]' | while read -r issue; do
            issue_num=$(echo "$issue" | jq -r '.number')
            echo -e "${GREEN}Processing issue #$issue_num...${NC}"
            
            format_issue_markdown "$issue" >> "$OUTPUT_FILE"
            echo -e "\n---\n" >> "$OUTPUT_FILE"
        done
        ;;
        
    json)
        # Get all issues with full details
        echo -e "${YELLOW}Generating JSON output...${NC}"
        
        if [[ "$INCLUDE_COMMENTS" == "true" ]]; then
            # Get issues with comments
            gh issue list --state "$STATE" --limit 100 --json number | \
            jq -r '.[] | .number' | \
            xargs -I {} gh issue view {} --json number,title,state,author,createdAt,labels,body,comments | \
            jq -s '.' > "$OUTPUT_FILE"
        else
            # Get issues without comments
            gh issue list --state "$STATE" --limit 100 --json number,title,state,author,createdAt,labels,body > "$OUTPUT_FILE"
        fi
        
        COUNT=$(jq 'length' "$OUTPUT_FILE")
        ;;
        
    *)
        echo -e "${RED}Invalid format: $FORMAT${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}✓ Successfully dumped $COUNT issues to $OUTPUT_FILE${NC}"
echo -e "${BLUE}File size: $(ls -lh "$OUTPUT_FILE" | awk '{print $5}')${NC}"

# Optional: Open the file
echo -e "${YELLOW}Open the file? (y/n)${NC}"
read -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    case "$FORMAT" in
        text|markdown)
            ${EDITOR:-open} "$OUTPUT_FILE"
            ;;
        json)
            if command -v jq &> /dev/null; then
                jq . "$OUTPUT_FILE" | less
            else
                less "$OUTPUT_FILE"
            fi
            ;;
    esac
fi