# shorty

A simple URL shortener app.

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
