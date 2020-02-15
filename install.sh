#!/bin/bash

if [ "$EUID" -ne 0 ]
  then echo "This script needs root privileges"
  exit
fi

echo "Building binary..."
crystal build src/azula.cr --release
echo "Copying binary"
cp azula /usr/bin/azula
echo "Copying stdlib"
mkdir -p /usr/lib/azula
cp -r src/azula/std /usr/lib/azula/sources
echo "Removing binary"
rm azula