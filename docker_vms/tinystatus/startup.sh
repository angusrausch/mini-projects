#!/bin/bash

set -e

# Define colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

BUILD=false
START=false
STOP=false
DIR=$(dirname $(realpath $0))

source $DIR/.env

show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --build	Build Docker Image"
    echo "  --start	Start Docker Container"
    echo "  --stop	Stop Docker Container"
}

if [[ $# -eq 0 ]]; then
  show_usage
  exit 1
fi

while [[ $# -gt 0 ]]; do
	case "$1" in
	    --build)
		BUILD=true
		shift
		;;
	    --start)
		START=true
		shift
		;;
	    --stop)
		STOP=true
		shift
		;;
	    *)
		echo "Unknown option: $1"
		show_usage
		exit 1
		;;
	esac
done

echo "$VLAN_NAME"

if ! docker network inspect $VLAN_NAME >/dev/null 2>&1; then
    echo -e "${YELLOW}Creating Docker Network...${NC}"
    docker network create -d macvlan \
  	--subnet=$SUBNET \
  	--gateway=$GATEWAY \
  	-o parent=$PARENT_DEVICE \
	--aux-address "host=${AUX_ADDRESS}" \
  	$VLAN_NAME
    echo -e "${GREEN}Created Network${NC}"
fi 

if $STOP; then
    echo -e "${YELLOW}Stopping contianer...${NC}"
    docker rm -f tinystatus >/dev/null 2>&1
    if [[ $? -ne 0 ]]; then
	echo -e "${RED}Failed to stop containers${NC}"
	exit 1
    fi
    echo -e "${BLUE}Container stopped${NC}"
    exit 0
fi

if $BUILD; then
    echo -e "\n${YELLOW}Builidng container...${NC}"
    docker build -t tinystatus $DIR
    if [[ $? -ne 0 ]]; then
	echo -e "${RED}Failed to build container${NC}"
	exit 1
    fi
    echo -e "${GREEN}Container built${NC}"
fi
	
if $START; then
    if docker ps | egrep "tinystatus" > /dev/null 2>&1; then
		echo -e "${YELLOW}Old container found. Shutting it down...${NC}"
			docker rm -f tinystatus > /dev/null 2>&1
			if [[ $? -ne 0 ]]; then
			echo -e "${RED}Failed to stop old container${NC}"
			exit 1
		fi
		echo -e "${GREEN}Old container stopped${NC}"
    fi
    echo -e "\n${YELLOW}Starting container...${CYAN}"

	docker run \
		--name=tinystatus \
		--network $VLAN_NAME \
		--ip $IP \
		--mac-address $MAC \
		--cap-add=NET_ADMIN \
		--cap-add=NET_RAW \
		--cap-add=SYS_NICE \
		--cap-add=NET_BROADCAST \
		-v ${DIR}/keepalived.conf:/etc/keepalived/keepalived.conf \
		-v ${DIR}/ssl/:/etc/nginx/ssl/ \
		-v ${DIR}/tinystatus.conf:/etc/nginx/http.d/tinystatus.conf \
		-v ${DIR}/checks.yaml:/usr/src/app/checks.yaml \
		-v ${DIR}/incidents.md:/usr/src/app/incidents.md \
		-v ${DIR}/assets:/usr/src/app/assets \
		--restart unless-stopped \
		-d \
		tinystatus

    if [[ $? -ne 0 ]]; then
	echo -e "${RED}Failed to start container${NC}"
	exit 1
    fi
    echo -e "${BLUE}Container started${NC}"
fi

exit 0
