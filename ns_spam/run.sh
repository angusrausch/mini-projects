#!/bin/bash

DIR="$(dirname "$(realpath "$0")")"

#Compile
${DIR}/build.sh
if [ $? -eq 0 ]; then
        #Run
        ${DIR}/target/ns-spam $*
    else
        exit 1
    fi

