# Example:
[![CircleCI](https://circleci.com/gh/klaatu01/woodchuck.svg?style=svg)](https://circleci.com/gh/klaatu01/woodchuck)

# Woodchuck

_How many Logs would a Woodchuck chuck if a would chuck could chuck Logs?_

AWS Lambda External Extension for Parsing/Forwarding CloudWatch Logs to Loggly.

## Installation MacOS - optional

Dependencies:
 - [docker](https://github.com/docker/cli)
 - [aws-cli](https://github.com/aws/aws-cli)
 
```bash
./build.sh && ./deploy.sh
```
_Builds inside of a docker mirror of the AWS Lambda provided runtime as outlined [here](https://github.com/awslabs/aws-lambda-rust-runtime)_

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
* [ ] More...

## Usage

Add the layer to your lambda and set environment variables for LOGGLY_TOKEN & LOGGLY_TAG:

_I will insert the publicly accessable ARN of the Layer once v1 is ready for release_

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License
[MIT](https://choosealicense.com/licenses/mit/)
