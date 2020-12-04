#!/bin/bash
if [ -d $1 ]; then
  echo "requires layer name"
  exit 1
fi
if [ -d $2 ]; then
  echo "requires region"
  exit 1
fi
aws lambda publish-layer-version --layer-name $1 --zip-file "fileb://extensions.zip" --region $2
