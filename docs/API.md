# Atlas ERP API Documentation

## Base URL
`http://localhost:8080/api/v1`

## Authentication
Atlas uses JWT for authentication. Include the token in the `Authorization` header:
`Authorization: Bearer <your-token>`

## Core Endpoints

### Entities
- `GET /{entity}`: List records for an entity.
- `POST /{entity}`: Create a new record.
- `GET /{entity}/{id}`: Get a specific record.
- `PUT /{entity}/{id}`: Update a specific record.
- `DELETE /{entity}/{id}`: Delete a specific record.

### Workflow
- `POST /{entity}/{id}/{action}`: Execute a workflow action (e.g., `approve`, `reject`).

### Admin
- `POST /api/admin/schema`: Create or update an entity schema.
