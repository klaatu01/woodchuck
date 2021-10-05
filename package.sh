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

cd $destination
zip -r extensions.zip extensions &&
mv extensions.zip ../
cd ..
rm -r $destination
