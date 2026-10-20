#!/bin/bash
set -e

URL=https://github.com/SingleStepTests/65x02/archive/refs/heads/main.tar.gz
REPO_PATH="65x02-main/6502/v1"
TARGET_DIR="./tests/json/"

echo "Downloading SingleStepTests (https://github.com/SingleStepTests/65x02)..."

if [ ! -d "$TARGET_DIR" ]; then
    echo "Directory '$TARGET_DIR' does not exist, creating..."
    mkdir "$TARGET_DIR"
elif [ ! -z "$( ls -A "$TARGET_DIR" )" ]; then
    echo "Directory '$TARGET_DIR' is not empty."
    exit 1
fi

cd "$TARGET_DIR"
curl -L "$URL" | tar -xz --strip-components=3 "$REPO_PATH"

CURL_STATUS="$?"
if [ "$CURL_STATUS" -eq 0 ]; then
    echo "Successfully downloaded tests in: '$TARGET_DIR'."
    exit 0
else
    echo "Failed to download tests."
    echo "Curl exited with status: '$CURL_STATUS'."
    exit 2
fi
