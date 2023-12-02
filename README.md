# worker-vless-rs

**This is an experimental project, and nothing is guaranteed.**

## Requirements

- a cloudflare account
- a domain under that account
- git, nodejs, npm, rust ...

## Setup

clone this repo

```shell
$ git clone https://github.com/oiioooiio/worker-vless-rs
```

change `account_id`, `routes` and `UUID` in wrangler.toml

deploy to cloudflare

```shell
$ npm install
$ npm run publish
```