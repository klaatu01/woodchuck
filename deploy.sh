#!/bin/bash
help_string="requires destination. build.sh loggly|custom x86_64|arm64 region"
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
RESULT=$(aws lambda publish-layer-version --layer-name $woodchuck_name --zip-file "fileb://extensions.zip" --region $3)
VERSION=$(echo $RESULT | jq '.Version')
aws lambda add-layer-version-permission --layer-name $woodchuck_name --region $3 --statement-id "public-layer" --version-number $VERSION --action lambda:GetLayerVersion --principal '*'
