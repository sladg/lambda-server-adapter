# AWS Lambda Server Adapter

A tool to run web servers on AWS Lambda

It allows us to start server before events come in and we keep it running in the background as long as events are coming in. This reduces cold starts and improves performance.

Useful for applications such as NextJS, Graphql or REST servers, Express, Apollo, etc.

## Features

- Starts server before events come in and keeps it running in the background as long as events are coming in.
- Translates AWS events into HTTP requests.
- Translates HTTP responses into AWS events.
- Simple plug-and-play as layer.
- Small size, no dependencies, as fast as your server.

## Usage

AWS Lambda Server Adapter work with native runtimes. It is added in a form of Layer.

[NextJS](/examples/Next.md)

### Lambda functions packaged as Zip package for AWS managed runtimes

<!-- @TODO: Publish layer arns for use. -->

### Handler configuration

<!-- @TODO: Document how extensions are resolved (node, python) from handler file -->

## Readiness Check

Once new Lambda is started, it will initialize the runtime via Runtime API. We start server and wait for it to respond to our HTTP calls, once it does, we mark the Lambda as ready and start processing events.

In case server does not start - command fails, HTTP response is not received, etc. - we will log the error and Lambda will die after 10sec.

## Configurations

<!-- @TODO: Document options -->

## Graceful Shutdown

<!-- @TODO: Implement passing the SIGTERM to handler -->

## Acknowledgement

This is a direct fork of [AWS Lambda Web Adapter](https://github.com/awslabs/aws-lambda-web-adapter) which was re-written and simplified. It's based on AWS's [lambda_http](https://docs.rs/lambda_http/latest/lambda_http/).
