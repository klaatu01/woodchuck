#!/bin/bash
help_string="requires destination. build.sh loggly|logzio x86_64|arm64 region"
if [ -d $1 ]; then
  echo $help_string
  exit 1
fi
if [ -d $2 ]; then
  echo $help_string
  exit 1
fi
if [ -d $3 ]; then
  echo $help_string
  exit 1
fi


woodchuck_name="woodchuck_$1_$2"
aws lambda publish-layer-version --layer-name $woodchuck_name --zip-file "fileb://extensions.zip" --region $3 --compatible-architectures $2
