# AWS Lambda Server Adapter

A tool to run web servers on AWS Lambda

It allows us to start server before events come in and we keep it running in the background as long as events are coming in. This reduces cold starts and improves performance.

Useful for applications such as NextJS, Graphql or REST servers, Express, Apollo, etc.

## Features

- Run web applications on AWS Lambda
- Supports Amazon API Gateway Rest API and Http API endpoints, Lambda Function URLs, and Application Load Balancer
- Supports Lambda managed runtimes, custom runtimes and docker OCI images
- Supports any web frameworks and languages, no new code dependency to include
- Automatic encode binary response
- Enables graceful shutdown
- Supports response payload compression
- Supports response streaming

## Usage

AWS Lambda Server Adapter work with native runtimes. It is added in a form of Layer.

### Lambda functions packaged as Zip package for AWS managed runtimes

<!-- @TODO: Publish layer arns for use. -->

## Readiness Check

Once new Lambda is started, it will initialize the runtime via Runtime API. We start server and wait for it to respond to our HTTP calls, once it does, we mark the Lambda as ready and start processing events.

In case server does not start - command fails, HTTP response is not received, etc. - we will log the error and Lambda will die after 10sec.

## Configurations

<!-- @TODO: Document options -->

## Graceful Shutdown

<!-- @TODO: Implement passing the SIGTERM to handler -->

## Acknowledgement

This is a direct fork of [AWS Lambda Web Adapter](https://github.com/awslabs/aws-lambda-web-adapter) which was re-written and simplified.
