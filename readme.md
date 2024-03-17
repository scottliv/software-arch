## Overview

My application gets the latest popular images and from them uses generative AI to create similar images. Its purpose is mainly for fun and to see if there are any interesting comparisons between the human and computer created images. To accomplish this it uses 2 third party apis, Unsplash for image collection and OpenAi for generation. The application is built using rust, developed as a number of different services that are their own packages but share common dependencies using a cargo workspace. The frontend of the project is built using htmx and was kept really simple as my main focus was on service architecture. It uses postgresql as a data store, which was chosen as the data models matched well to a relational database in that the 2 main models have a 1 to 1 relation. It uses sea-orm to manage the DB in app, which provides a nice API for migrations and record querying. The migrations are part of the Database service and are automatically run when a container is spun up. This works well for this project, but would probably need to be looked at if it were running multiple instances. The project also leverages postgres as a message queue using [pgmq](https://github.com/tembo-io/pgmq), which provides a message queue with a similar api to aws SQS which I selected as it has a nice rust sdk and was easy to integrate into the project. For image storage it uses a S3 bucket.

### System Requirements

- Docker
- Rust Tool chain

### Environment Variables

To connect to 3rd party APIs the following variables must be set in a `.env` in the project root

S3_SECRET_KEY
S3_ACCESS_KEY
UNSPLASH_ACCESS_KEY
OPEN_AI_ACCESS_KEY

### Commands

Scripts can be found in `Makefile.toml` (they require `cargo make` to be installed)

`cargo make` will build all services

`docker compose up` will run all build services

### Data Collector

The data collector service is a service that runs a cron job scheduled to run every day to fetch the most recent images from the unsplash API. It is developed in rust and uses a number of packages to aid in scheduling and web requests. It writes to a postgres database that is shared between services. It also enqueues a message to a postgres message queue after successfully saving an image.

### Data Analyzer

The data analyzer component of the project is the image generator. It reads from a postgres message queue and on each incoming message uses the inspiration image to generate a new image. It writes the generated image to a shared postgres db and uploads the image to a S3 bucket. When the queue is empty, it sleeps for a bit.

### Testing

Unit and Integration tests can be found in the image collection, server and the database services. I utilize a docker-container library to fully test the database integration and operations on the image collection service. I haven’t completed 100% test coverage, but that would be a nice future improvement.

### CI/CD

The integration and deployment logic lives in `.github/workflows/ec2deploy.yaml` which runs the test suite, builds the service images on docker hub and loads and starts them on an aws ec2 instance.

### Production

The production environment is all in AWS, with everything running in a container on an EC2 instance and an S3 bucket for image storage. This was a relatively simple way to get up and running, but could probably be improved

### Metrics & Monitoring

This project uses prometheus to gather metrics and grafana for visualizing, both are included in docker-compose.yml. The project makes use of the `tracing` package for logging which logs to `stdout` a future improvement would be better capturing of production logs using a 3rd party service to have better structured and searchable logs.

### Local Development

I am currently using the production docker compose to run everything in a container locally for development. If I were to continue this project (or follow a similar architecture in the future) I would put more effort into the local dev experience as it requires the container to be rebuilt on each code change, which is not ideal but wasn’t painful enough for me to fix before the end of the project. I think I would set up the services (db and message queue) in a `docker-compose-dev.yml` and then run everything else locally as needed in order to leverage tools like cargo-watch to recompile on save which would drastically shorten the dev feedback loop.
