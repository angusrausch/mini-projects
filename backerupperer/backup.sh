#!/bin/bash

set -e

# Define colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

DRY=""

show_usage() {
  echo -e "${BLUE}Usage: $0 [OPTIONS]${NC}"
  echo ""
  echo -e "${BLUE}Options:${NC}"
  echo -e "  --dry          ${YELLOW}Dry run - files will not be copied${NC}"
  echo -e "  -h, --help     ${CYAN}Show this help message${NC}"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry)
      DRY="--dry-run"
      echo -e "${YELLOW}Running in dry-run mode${NC}"
      shift
      ;;
    -h|--help)
      show_usage
      exit 0
      ;;
    *)
      echo -e "${RED}Unknown option: $1${NC}"
      show_usage
      exit 1
      ;;
  esac
done

# Prepare logging
RUN_DIR=$(dirname $(realpath $0))
CONFIG_FILE="backup.yaml"
LOG_DIR=$(yq '.log_dir' "$CONFIG_FILE")
mkdir -p "${LOG_DIR}"
LOG_FILE="${LOG_DIR}/backup_$(date +"%Y-%m-%d_%H-%M-%S").log"

# Redirect all output to tee (stdout + log file)
exec > >(tee -a "$LOG_FILE") 2>&1

echo -e "${YELLOW}Loading configuration from ${CYAN}$CONFIG_FILE${NC}...${NC}"
if ! command -v yq &> /dev/null; then
  echo -e "${RED}yq is required but not installed. Install with: brew install yq${NC}"
  exit 1
fi

echo -e "${GREEN}Backerupperer started: $(date +"%Y-%m-%d %H:%M:%S")${NC}"

NUM_DIRS=$(yq '.directories | length' "$CONFIG_FILE")

for ((i=0; i<NUM_DIRS; i++)); do
  DIR_PATH=$(yq ".directories[$i].path" "$CONFIG_FILE")
  BACKUP_LOCATION=$(yq ".directories[$i].backup_location" "$CONFIG_FILE")
  FILETYPE=$(yq ".directories[$i].file_type" "$CONFIG_FILE")
  TYPE=$(yq ".directories[$i].type" "$CONFIG_FILE")
  LIMIT=$(yq ".directories[$i].limit" "$CONFIG_FILE")
  KEYWORDS_COUNT=$(yq ".directories[$i].keywords | length" "$CONFIG_FILE")
  
  echo -e "${CYAN}\nProcessing directory: ${MAGENTA}$DIR_PATH${NC}"

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
      echo -e "${YELLOW}Selecting newest $LIMIT files${NC}"
    fi
    echo -e "${BLUE}Files to backup:${NC}\n$MATCHED_FILES"
    rsync -avzhr --delete --delay-updates ${DRY} ${MATCHED_FILES} ${BACKUP_LOCATION}
    echo -e "${GREEN}Backup completed for directory $DIR_PATH${NC}"
  else
    KEYWORDS=($(yq ".directories[$i].keywords[]" "$CONFIG_FILE"))
    echo -e "${GREEN}Loaded config for directory $DIR_PATH:${NC}"
    echo -e "  Backup location: ${CYAN}$BACKUP_LOCATION${NC}"
    echo -e "  Keywords: ${MAGENTA}${KEYWORDS[*]}${NC}"
    echo -e "  File type: ${CYAN}$FILETYPE${NC}"
    echo -e "  Type: ${CYAN}$TYPE${NC}"
    echo -e "  Limit: ${CYAN}$LIMIT${NC}"

    EXT_PATTERN="$(echo "$FILETYPE" | sed 's/\./\\./g')\$"

    echo -e "${YELLOW}Searching for files by keywords...${NC}"
    for KEYWORD in "${KEYWORDS[@]}"; do
      echo -e "${MAGENTA}Keyword: $KEYWORD${NC}"
      MATCHED_FILES=$(find "$DIR_PATH" -type f | grep "$KEYWORD" | grep -E "$EXT_PATTERN")
      if [[ "$TYPE" == "newest" ]]; then
        MATCHED_FILES=$(ls -t $MATCHED_FILES 2>/dev/null | head -n "$LIMIT")
        echo -e "${YELLOW}Selecting newest $LIMIT files for keyword $KEYWORD${NC}"
      fi
      echo -e "${BLUE}Files to backup for keyword $KEYWORD:${NC}\n$MATCHED_FILES"
      rsync -avzhr --delete --delay-updates ${DRY} ${MATCHED_FILES} ${BACKUP_LOCATION}
      echo -e "${GREEN}Backup completed for keyword $KEYWORD${NC}"
    done
  fi
done

echo -e "${GREEN}\nAll backups completed successfully at $(date +"%Y-%m-%d %H:%M:%S")${NC}"
echo -e "${CYAN}Log file: ${YELLOW}$LOG_FILE${NC}"
