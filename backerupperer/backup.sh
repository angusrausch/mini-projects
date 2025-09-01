#!/bin/bash

set -e

# Define colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
ORANGE='\033[38;5;208m'
NC='\033[0m' # No Color

DRY=""
DELETE=false

show_usage() {
  echo -e "${BLUE}Usage: $0 [OPTIONS]${NC}"
  echo ""
  echo -e "${BLUE}Options:${NC}"
  echo -e "  --dry            ${YELLOW}Dry run - files will not be copied${NC}"
  echo -e "  --delete         ${YELLOW}Delete any files which match the keyword from the output directory and set rsync delete flag${NC}"
  echo -e "  --manual         ${YELLOW}Manual run. Shows rsync progress.${NC}"
  echo -e "  -h, --help       ${CYAN}Show this help message${NC}"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry)
      DRY="--dry-run"
      echo -e "${YELLOW}Running in dry-run mode${NC}"
      shift
      ;;
    --delete)
      DELETE=true
      shift
      ;;
    --manual)
      MANUAL=--progress
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
CONFIG_FILE="${RUN_DIR}/backup.yaml"
LOG_DIR=$(yq '.log_dir' "$CONFIG_FILE")
mkdir -p "${LOG_DIR}"
LOG_FILE="${LOG_DIR}/backup_$(date +"%Y-%m-%d_%H-%M-%S").log"

if $DELETE; then
  DELETE_FLAG=--delete
fi

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
    rsync -avzhr ${MANUAL} ${DELETE_FLAG} --delay-updates ${DRY} ${MATCHED_FILES} ${BACKUP_LOCATION}
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
      rsync -avzhr ${MANUAL} ${DELETE_FLAG} --delay-updates ${DRY} ${MATCHED_FILES} ${BACKUP_LOCATION}
      echo -e "${GREEN}Backup completed for keyword $KEYWORD${NC}"

      if [[ $DRY != true && $DELETE == true ]]; then
        echo -e "\n${YELLOW}Deleting previous matching files${NC}"

        if [[ "$BACKUP_LOCATION" =~ ^[^@]+@[^:]+:.+ ]]; then
          REMOTE_USER_HOST=$(echo "$BACKUP_LOCATION" | cut -d: -f1)
          REMOTE_PATH=$(echo "$BACKUP_LOCATION" | cut -d: -f2-)
          # List files to delete
          DELETED_FILES=$(ssh "$REMOTE_USER_HOST" "find \"$REMOTE_PATH\" -maxdepth 1 -type f -name \"*${KEYWORD}*\" ! -name \"$(basename "$MATCHED_FILES")\"")
          if [[ -n "$DELETED_FILES" ]]; then
            echo -e "${ORANGE}Files deleted remotely:${NC}"
            while IFS= read -r file; do
              [[ -n "$file" ]] && echo -e "${ORANGE}$file${NC}"
            done <<< "$DELETED_FILES"
            # Delete files
            ssh "$REMOTE_USER_HOST" "echo \"$DELETED_FILES\" | xargs -d '\n' rm -f --"
          else
            echo -e "${ORANGE}No files to delete remotely.${NC}"
          fi
        else
          # List files to delete
          DELETED_FILES=$(find "${BACKUP_LOCATION}" -maxdepth 1 -type f -name "*${KEYWORD}*" ! -name "$(basename "$MATCHED_FILES")")
          if [[ -n "$DELETED_FILES" ]]; then
            echo -e "${ORANGE}Files deleted locally:${NC}"
            while IFS= read -r file; do
              [[ -n "$file" ]] && echo -e "${ORANGE}$file${NC}"
              rm -f -- "$file"
            done <<< "$DELETED_FILES"
          else
            echo -e "${ORANGE}No files to delete locally.${NC}"
          fi
        fi
        echo -e "${GREEN}Files Successfully Deleted${NC}"
      fi

    done
  fi
done

echo -e "${GREEN}\nAll backups completed successfully at $(date +"%Y-%m-%d %H:%M:%S")${NC}"
echo -e "${CYAN}Log file: ${YELLOW}$LOG_FILE${NC}"
echo -e "${GREEN}\nAll backups completed successfully at $(date +"%Y-%m-%d %H:%M:%S")${NC}"
echo -e "${CYAN}Log file: ${YELLOW}$LOG_FILE${NC}"
