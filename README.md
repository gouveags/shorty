# shorty

A simple URL shortener app.

## Why I'm building this

This is my personal attempt to build a URL shortener while studying system design.

The idea is simple: take a very long URL and turn it into something short. Behind that tiny link, though, there is a lot of stuff worth learning: databases, caching, redirects, counters, concurrency, Docker, and how to make a service that does not fall apart after more than one person clicks the same link.

I'm doing this because system design can get very abstract very fast, and I don't want to just memorize boxes and arrows, although the Excalidraw drawings do look cool.

I want to build the thing, break the thing, fix the thing, and hopefully understand the thing.

Rust because I want to learn it properly.

Also because apparently I looked at system design, databases, Redis, Docker, async code, and thought: "yes, let's make this harder with lifetimes too."

Anyways...

## Goal

Build and deploy a real web app where users can create short links and be redirected reliably.

## Status

First implementation.

## Run

```bash
docker compose up --build
```

The API listens on `http://localhost:8080`.

## Test

```bash
cargo test
```

## API

Create a generated short URL:

```bash
curl -X POST http://localhost:8080/urls \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://example.com"}'
```

Create a custom short URL:

```bash
curl -X POST http://localhost:8080/urls \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://example.com","custom_code":"example"}'
```

Redirect:

```bash
curl -i http://localhost:8080/example
```

Stats:

```bash
curl http://localhost:8080/urls/example
```
