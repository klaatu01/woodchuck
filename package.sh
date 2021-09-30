#!/bin/bash

help_string="Requires: package.sh loggly|custom x86_64|arm64"
if [ -d $1 ]; then
  echo $help_string
  exit 1
fi
if [ -d $2 ]; then
  echo $help_string
  exit 1
fi

destination="destination_$1_$2"

zip -r extensions.zip $destination/extensions &&
rm -r $destination
