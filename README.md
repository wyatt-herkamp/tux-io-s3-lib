# TuxIO S3 Library

A Rust S3 Library with the goal of providing a client to S3 services or building S3 compatible service

## Why?
I am working on planning on building an S3 compatible service and wanted to modulize part of the code.

### Testing

Some tests are behind a feature flag `client-testing`. To run these tests you will need to enable this feature.
These tests require you to have a configuration to allow it to connect to an S3 instance.