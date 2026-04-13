# Atlas ERP - Declarative Enterprise Resource Planning

A fully declarative, microservices-based ERP system built entirely in Rust. Inspired by Oracle Fusion, Atlas treats **everything as data** - data models, business processes, forms, reports, validation rules, and workflows are all defined declaratively and can be modified at runtime without service restarts.

## Key Features

### 100% Declarative Architecture
- **Dynamic Data Models**: Define entities and fields via JSON/YAML configuration
- **Workflow Engine**: State machines with guards and actions defined declaratively
- **Validation Engine**: Field-level and cross-field validation rules
- **Formula Engine**: Computed fields with expressions
- **Security Engine**: Row-level and field-level security policies

### Hot-Reload Configuration
No restarts required for:
- Adding new entities
- Modifying workflows
- Changing validation rules
- Updating security policies
- Creating new forms or reports

### Microservices Architecture
- **atlas-core**: Schema engine, workflow engine, validation, formula, security, audit
- **atlas-gateway**: API gateway with authentication
- **atlas-hcm**: Human Capital Management
- **atlas-financials**: Financial modules
- **atlas-scm**: Supply Chain Management
- **atlas-crm**: Customer Relationship Management
- **atlas-projects**: Project Management

## Tech Stack

- **Backend**: Rust with Axum 0.7
- **Frontend**: Leptos (WASM) - Full Rust stack
- **Database**: PostgreSQL 16
- **Event Bus**: NATS
- **ORM**: SQLx with compile-time query checking

## Quick Start

### Prerequisites
- Rust 1.76+
- Docker & Docker Compose
- PostgreSQL 16
- NATS

### Run Infrastructure
```bash
docker-compose up -d postgres nats
```

### Seed Configuration
```bash
./scripts/seed-config.sh
```

### Run the Gateway
```bash
cargo run -p atlas-gateway
```

The API will be available at `http://localhost:8080`

## API Examples

### Create a New Entity
```bash
curl -X POST http://localhost:8080/api/admin/schema \
  -H "Content-Type: application/json" \
  -d '{
    "definition": {
      "name": "expense_reports",
      "label": "Expense Report",
      "plural_label": "Expense Reports",
      "fields": [
        {"name": "title", "label": "Title", "field_type": {"type": "string"}, "required": true},
        {"name": "amount", "label": "Amount", "field_type": {"type": "currency", "code": "USD"}}
      ],
      "workflow": {
        "initial_state": "draft",
        "states": [
          {"name": "draft", "label": "Draft", "state_type": "initial"},
          {"name": "approved", "label": "Approved", "state_type": "final"}
        ],
        "transitions": [
          {"from": "draft", "to": "approved", "action": "approve", "role": "finance_manager"}
        ]
      }
    },
    "create_table": true
  }'
```

### Query Records
```bash
curl http://localhost:8080/api/v1/employees
```

### Execute Workflow Action
```bash
curl -X POST http://localhost:8080/api/v1/purchase_orders/123/approve \
  -H "Content-Type: application/json" \
  -d '{"comment": "Looks good, approved"}'
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         API Gateway                              │
└─────────────────────────────────────────────────────────────────┘
         │              │              │              │
    ┌────┴────┐   ┌────┴────┐   ┌────┴────┐   ┌────┴────┐
    │   HCM   │   │Financials│  │   SCM   │   │   CRM   │
    └─────────┘   └─────────┘  └─────────┘   └─────────┘
         │
    ┌────┴────────────────────────────────────────────┐
    │                    Core Engine                   │
    │  ┌──────────┐  ┌──────────┐  ┌────────────────┐  │
    │  │ Schema   │  │ Workflow │  │ Configuration  │  │
    │  │ Engine   │  │ Engine   │  │ (Hot-Reload)   │  │
    │  └──────────┘  └──────────┘  └────────────────┘  │
    └──────────────────────────────────────────────────┘
         │
    ┌────┴────┐
    │PostgreSQL│
    └─────────┘
```

## Project Structure

```
atlas/
├── Cargo.toml                    # Workspace
├── SPEC.md                       # Detailed specification
├── docker-compose.yml            # Infrastructure
├── crates/
│   ├── atlas-core/               # Declarative engine
│   ├── atlas-shared/              # Shared types
│   ├── atlas-gateway/             # API server
│   ├── atlas-hcm/                 # Human Capital
│   ├── atlas-financials/           # Financial
│   ├── atlas-scm/                 # Supply Chain
│   ├── atlas-crm/                 # CRM
│   └── atlas-projects/            # Projects
├── frontend/                     # Leptos frontend
├── migrations/                   # Database migrations
└── scripts/                      # Utility scripts
```

## Documentation

- [SPEC.md](SPEC.md) - Complete system specification
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - Detailed architecture
- [API.md](docs/API.md) - API documentation
- [DECLARATIVE.md](docs/DECLARATIVE.md) - Declarative patterns guide

## License

MIT OR Apache-2.0
