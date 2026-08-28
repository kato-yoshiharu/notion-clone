# notion-clone

## Overview

This is a [Notion](https://www.notion.so/) clone.

[Demo](https://notion-clone-frontend.kato-yoshiharu.workers.dev)

## Technology Stacks

- **GraphQL**: For schema and API protocol.
- **Rust, Axum**: For backend development.
- **TypeScript, Next.js**: For frontend development.

## Setting up and running locally

### backend

To run backend server locally, and run

```sh
cd backend
# execute only once
makers init
makers dev
```

The server listens on port 8080. To check that it is running, send the `healthCheck` GraphQL query:

```sh
curl -X POST http://localhost:8080/graphql \
  -H 'Content-Type: application/json' \
  -d '{"query":"{ healthCheck }"}'
# => {"data":{"healthCheck":"OK"}}
```

### frontend

Requires Node 20.x and pnpm 3+. To run frontend server locally, and run

```sh
cd frontend
# execute only once
pnpm install
pnpm run dev
```

## License

This is MIT licensed.
