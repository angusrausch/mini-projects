#!/bin/bash

set -e

# Define colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

DRY=""

show_usage() {
  echo "Usage: $0 [OPTIONS]"
  echo ""
  echo "Options:"
  echo "  --dry          Dry run - not files copied"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry)
      DRY="--dry-run"
      shift
      ;;
    -h|--help)
      show_usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      show_usage
      exit 1
      ;;
  esac
done

echo -e "${YELLOW}Loading configuration from backup.yaml...${NC}"
if ! command -v yq &> /dev/null; then
  echo -e "${RED}yq is required but not installed. Install with: brew install yq${NC}"
  exit 1
fi

CONFIG_FILE="backup.yaml"
NUM_DIRS=$(yq '.directories | length' "$CONFIG_FILE")

for ((i=0; i<NUM_DIRS; i++)); do
  DIR_PATH=$(yq ".directories[$i].path" "$CONFIG_FILE")
  BACKUP_LOCATION=$(yq ".directories[$i].backup_location" "$CONFIG_FILE")
  FILETYPE=$(yq ".directories[$i].file_type" "$CONFIG_FILE")
  TYPE=$(yq ".directories[$i].type" "$CONFIG_FILE")
  LIMIT=$(yq ".directories[$i].limit" "$CONFIG_FILE")
  KEYWORDS_COUNT=$(yq ".directories[$i].keywords | length" "$CONFIG_FILE")
  if [[ "$KEYWORDS_COUNT" -eq 0 ]]; then
    echo -e "${YELLOW}No keywords specified, copying all files from $DIR_PATH${NC}"
    if [[ -z "$FILETYPE" || "$FILETYPE" == "null" ]]; then
      MATCHED_FILES=$(find "$DIR_PATH" -type f)
    else
      EXT_PATTERN="$(echo "$FILETYPE" | sed 's/\./\\./g')\$"
      MATCHED_FILES=$(find "$DIR_PATH" -type f | grep -E "$EXT_PATTERN")
    fi
    if [[ "$TYPE" == "newest" ]]; then
      MATCHED_FILES=$(ls -t $MATCHED_FILES 2>/dev/null | head -n "$LIMIT")
    fi
    echo "$MATCHED_FILES"
    rsync -avzhr --delete --delay-updates ${DRY} ${MATCHED_FILES} ${BACKUP_LOCATION}
  else
    KEYWORDS=($(yq ".directories[$i].keywords[]" "$CONFIG_FILE"))
    echo -e "${GREEN}Loaded config for directory $DIR_PATH:${NC}"
    echo "  Backup location: $BACKUP_LOCATION"
    echo "  Keywords: ${KEYWORDS[*]}"
    echo "  File type: $FILETYPE"
    echo "  Type: $TYPE"
    echo "  Limit: $LIMIT"

    EXT_PATTERN="$(echo "$FILETYPE" | sed 's/\./\\./g')\$"

    echo -e "${YELLOW}Searching for files by keywords...${NC}"
    for KEYWORD in "${KEYWORDS[@]}"; do
      echo -e "${CYAN}Keyword: $KEYWORD${NC}"
      MATCHED_FILES=$(find "$DIR_PATH" -type f | grep "$KEYWORD" | grep -E "$EXT_PATTERN")
      if [[ "$TYPE" == "newest" ]]; then
        MATCHED_FILES=$(ls -t $MATCHED_FILES 2>/dev/null | head -n "$LIMIT")
      fi
      echo "$MATCHED_FILES"
      rsync -avzhr --delete --delay-updates ${DRY} ${MATCHED_FILES} ${BACKUP_LOCATION}
    done
  fi
done

