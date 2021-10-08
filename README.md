# Example:
[![CircleCI](https://circleci.com/gh/klaatu01/woodchuck.svg?style=svg)](https://circleci.com/gh/klaatu01/woodchuck)

# Woodchuck

_How many Logs would a Woodchuck chuck if a would chuck could chuck Logs?_

AWS Lambda External Extension for Forwarding CloudWatch Logs to Supported Logging Platforms.

## Support

Currently Supported Runtimes:
* [x] nodejs12.x
* [x] nodejs10.x
* [x] python2.7
* [x] python3.7
* [x] dotnetcore3.1
* [x] dotnetcore2.1
* [ ] go1.x
* [ ] java11
* [ ] java8.al2
* [ ] java8
* [ ] ruby2.7
* [ ] ruby2.5

Currently Supported Log Destinations:
* [x] Loggly
* [x] Logzio

## Serverless Framework

If you are using the Serverless Framework checkout the official [plugin](github.com/klaatu01/serverless-plugin-woodchuck).

## Prebuilt Layers

Provided you are in using any of the following AWS Regions:
  - eu-west-1
  - eu-west-2
  - eu-central-1
  - us-east-1
  - us-east-2
  - us-west-2

There are some premade layers avaliable for each Destination and Architecture:
  - arn:aws:lambda:<region>:856198688143:layer:woodchuck_loggly_x86_64
  - arn:aws:lambda:<region>:856198688143:layer:woodchuck_logzio_x86_64
  - arn:aws:lambda:<region>:856198688143:layer:woodchuck_loggly_arm64
  - arn:aws:lambda:<region>:856198688143:layer:woodchuck_logzio_arm64

## Building from source

If your organisation does not want to use one of the premade layers. Woodchuck can be built from source for your desired `destination (loggly|logzio)` and `architecture (x86_64|arm64)`

### Dependencies
 - [docker](https://github.com/docker/cli): To compile the plugin on the target Architecture.
 - [aws-cli](https://github.com/aws/aws-cli): To deploy the layer to your AWS Account.
 
### Building:
```bash
./build.sh <loggly|logzio> <x86_64|arm64>
```

### Publishing:
```bash
./publish.sh <loggly|logzio> <x86_64|arm64> <region>
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[MIT](https://choosealicense.com/licenses/mit/)
