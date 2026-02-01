# NEURAL-NOTES
[![CI Axum](https://github.com/lyh4215/neural-notes/actions/workflows/ci_axum.yml/badge.svg)](https://github.com/lyh4215/neural-notes/actions/workflows/ci_axum.yml)
[![Cloudflare Pages](https://img.shields.io/badge/Cloudflare%20Pages-Deployed-orange?logo=cloudflare)](https://neural-notes.pages.dev)

This is monorepo of `neural-notes` app.



![example](.github/docs/examples2.gif)


## Structure
![pipeline](.github/docs/pipeline.jpeg)

### Write-back Cache as an Architectural Trade-off
[axum-redis-cache](https://github.com/lyh4215/axum-redis-cache)
A write-back cache layer was introduced in front of the database using Redis and evaluated within this product, a personal document web application.

In practice, this approach resulted in measurable performance improvements:
- Significantly reduced read and write latency
- Removed database contention from the synchronous request path
- Enabled fast responses without additional `latency-hiding` techniques

### Limitations

The limitations of this approach are as follows:

- Application state becomes difficult to reason about if the Redis cache layer is unavailable or inconsistent
- Strong durability guarantees cannot be provided without additional recovery mechanisms
- Therefore, this model is unsuitable for domains where data must be durably preserved (e.g. document applications)

In summary, a write-back cache is effective for high-throughput, latency-sensitive workloads, but requires careful domain selection and should be applied only with careful consideration of the domain.


## Quick Start for dev
1. install just &  docker

2. in your shell
    ```bash
    git clone git@github.com:lyh4215/neural-notes.git
    cd neural-notes
    just env
    just dev
    ```
3. go to http://localhost:5173

## Backend - axum
- use [axum-redis-cache](https://github.com/lyh4215/axum-redis-cache)



## Frontend - react


## Embed api - FastAPI
- use embedding for *related post recommanding system*
