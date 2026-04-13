# Atlas ERP - Declarative Enterprise Resource Planning System

## Overview

Atlas is a fully declarative, microservices-based ERP system built entirely in Rust. Unlike traditional ERP systems where business logic is hardcoded, Atlas treats **everything as data** - data models, business processes, forms, reports, validation rules, and workflows are all defined declaratively and can be modified at runtime without service restarts.

**Inspired by**: Oracle Fusion, SAP S/4HANA, Odoo, and modern low-code platforms
**Built with**: Rust (Axum for HTTP, Leptos for frontend), PostgreSQL, NATS for event streaming

---

## Architecture Principles

### 1. Everything is Declarative
```yaml
# Example: Defining a complete "Purchase Order" entity
entity: purchase_orders
fields:
  - name: po_number
    type: string
    required: true
    unique: true
  - name: supplier_id
    type: reference
    entity: suppliers
    required: true
  - name: lines
    type: one_to_many
    entity: purchase_order_lines
  - name: total_amount
    type: computed
    formula: SUM(lines.amount)
  - name: status
    type: enum
    values: [draft, submitted, approved, rejected, closed]
workflow:
  initial_state: draft
  transitions:
    - from: draft
      to: submitted
      action: submit
      guard: "validate.required(supplier_id) AND validate.not_empty(lines)"
    - from: submitted
      to: approved
      action: approve
      role: purchase_manager
    - from: submitted
      to: rejected
      action: reject
      role: purchase_manager
```

### 2. Hot-Reload Architecture
- Configuration stored in PostgreSQL, not files
- Changes propagate via event bus to all services
- Services subscribe to relevant config changes
- No deployment required for business logic changes

### 3. Microservices Domain Separation
```
┌─────────────────────────────────────────────────────────────────┐
│                         API Gateway                              │
│                    (Authentication, Routing)                      │
└─────────────────────────────────────────────────────────────────┘
         │              │              │              │
    ┌────┴────┐   ┌────┴────┐   ┌────┴────┐   ┌────┴────┐
    │   HCM   │   │Financials│  │   SCM   │   │   CRM   │
    │         │   │         │  │         │   │         │
    └─────────┘   └─────────┘  └─────────┘   └─────────┘
         │              │              │              │
    ┌────┴────────────────────────────────────────────┴────┐
    │                    Core Engine                          │
    │  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐  │
    │  │ Schema   │  │ Workflow │  │ Configuration Store  │  │
    │  │ Engine   │  │ Engine   │  │ (Hot-Reload)         │  │
    │  └──────────┘  └──────────┘  └──────────────────────┘  │
    └────────────────────────────────────────────────────────┘
         │
    ┌────┴────┐
    │PostgreSQL│ (Primary data + Configuration)
    └─────────┘
         │
    ┌────┴────┐
    │  NATS   │ (Event Bus for inter-service communication)
    └─────────┘
```

---

## Domain Modules

### Core Engine (atlas-core)
- **Schema Engine**: Dynamic entity definitions, fields, relationships
- **Workflow Engine**: State machines, transitions, guards, actions
- **Validation Engine**: Declarative rules, cross-field validation
- **Formula Engine**: Computed fields, aggregations, expressions
- **Security Engine**: Row-level security, field-level access
- **Audit Engine**: Complete change tracking, time-travel queries

### HCM - Human Capital Management (atlas-hcm)
- Organizations & Positions
- Employees & Contractors
- Payroll & Benefits
- Time & Attendance
- Performance Management

### Financials (atlas-financials)
- Chart of Accounts
- General Ledger
- Accounts Payable
- Accounts Receivable
- Fixed Assets
- Cost Management
- Budgeting & Planning

### SCM - Supply Chain Management (atlas-scm)
- Products & Inventory
- Suppliers & Sourcing
- Purchase Orders
- Sales Orders
- Warehouse Management
- Demand Planning

### CRM - Customer Relationship Management (atlas-crm)
- Customers & Contacts
- Leads & Opportunities
- Sales Pipeline
- Marketing Campaigns
- Service Cases

### Project Management (atlas-projects)
- Projects & Tasks
- Resource Allocation
- Timesheets
- Project Billing

---

## Data Model (Declarative Schema)

### Core Primitives
```rust
// Every type is defined in the configuration store
enum FieldType {
    String { max_length: Option<usize>, pattern: Option<String> },
    Integer { min: Option<i64>, max: Option<i64> },
    Decimal { precision: u8, scale: u8 },
    Boolean,
    Date,
    DateTime,
    Reference { entity: EntityId, field: FieldId },
    OneToMany { entity: EntityId, foreign_key: FieldId },
    OneToOne { entity: EntityId, foreign_key: FieldId },
    Enum { values: Vec<String> },
    Computed { formula: String, return_type: FieldType },
    Attachment,
    Currency { code: String },
    RichText,
    Json,
}
```

### Relationships
- Entities can reference other entities
- Cascade rules defined declaratively
- Soft deletes with audit trail
- Multi-tenancy via organization_id

---

## Business Process Engine

### Workflow Definition
```yaml
workflow: purchase_order_approval
entity: purchase_orders
initial_state: draft

states:
  - name: draft
    type: initial
    entry_actions:
      - set_field: { status: draft }
      - send_notification: { template: po_draft_created }
  - name: submitted
    type: working
    entry_actions:
      - send_notification: { template: po_submitted, to: approvers }
  - name: approved
    type: final
    entry_actions:
      - set_field: { status: approved }
      - send_notification: { template: po_approved }
      - invoke_action: create_supplier_invoice
  - name: rejected
    type: final

transitions:
  - name: submit
    from: draft
    to: submitted
    action: submit
    guard:
      - validate.not_empty: supplier_id
      - validate.not_empty: lines
      - validate.greater_than: total_amount, 0
  - name: approve
    from: submitted
    to: approved
    action: approve
    required_role: purchase_manager
  - name: reject
    from: submitted
    to: rejected
    action: reject
    required_role: purchase_manager
    entry_actions:
      - send_notification: { template: po_rejected, reason_required: true }
```

---

## API Design

### RESTful Endpoints (Generated from Schema)
```
GET    /api/v1/{entity}                    # List with filtering, pagination
POST   /api/v1/{entity}                     # Create
GET    /api/v1/{entity}/{id}                # Read
PUT    /api/v1/{entity}/{id}                # Update
DELETE /api/v1/{entity}/{id}                # Soft delete
POST   /api/v1/{entity}/{id}/{action}       # Workflow action (approve, reject, etc.)

# Workflow state machine
GET    /api/v1/{entity}/{id}/history        # Audit trail
GET    /api/v1/{entity}/{id}/transitions    # Available actions

# Schema introspection
GET    /api/v1/schema/{entity}              # Entity definition
GET    /api/v1/schema/{entity}/form         # Form configuration
GET    /api/v1/schema/{entity}/list         # List view configuration
```

### WebSocket Events
```json
{
  "type": "config_changed",
  "entity": "purchase_orders",
  "version": 42,
  "changes": ["workflow.transitions"]
}
```

---

## Project Structure

```
atlas/
├── Cargo.toml                    # Workspace definition
├── SPEC.md
├── docker-compose.yml
│
├── crates/
│   ├── atlas-core/               # Core declarative engine
│   │   ├── src/
│   │   │   ├── schema/           # Dynamic schema engine
│   │   │   ├── workflow/         # State machine engine
│   │   │   ├── validation/       # Rule engine
│   │   │   ├── formula/          # Expression evaluator
│   │   │   ├── security/         # Access control
│   │   │   └── audit/            # Change tracking
│   │   └── Cargo.toml
│   │
│   ├── atlas-macros/             # Derive macros for entities
│   │   └── Cargo.toml
│   │
│   ├── atlas-shared/             # Shared types, proto, utils
│   │   ├── src/
│   │   │   ├── proto/            # Protobuf definitions
│   │   │   ├── errors.rs
│   │   │   ├── types.rs
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   │
│   ├── atlas-gateway/            # API Gateway & Auth
│   │   └── Cargo.toml
│   │
│   ├── atlas-hcm/                # Human Capital Management
│   ├── atlas-financials/          # Financial modules
│   ├── atlas-scm/                # Supply Chain
│   ├── atlas-crm/                # Customer Relations
│   └── atlas-projects/           # Project Management
│
├── frontend/
│   ├── Cargo.toml                # Leptos workspace
│   ├── apps/
│   │   ├── dashboard/            # Main dashboard app
│   │   └── admin/                # System administration
│   └── packages/
│       ├── ui/                   # Component library
│       └── schema-client/        # Type-safe schema client
│
├── migrations/                   # SQL migrations
│
└── scripts/
    ├── seed-config.sh            # Load initial declarative config
    └── test-workflows.sh
```

---

## Technology Stack

### Backend (Rust)
- **Web Framework**: Axum 0.7
- **ORM**: SQLx with compile-time query checking
- **Serialization**: Serde with JSON Schema generation
- **Validation**:Validator
- **Auth**: JWT + OAuth2 (Keycloak integration ready)
- **Event Bus**: NATS (async-nats)
- **Config**: Custom hot-reload system (no env vars needed)
- **Async Runtime**: Tokio

### Frontend (Rust + Leptos)
- **Framework**: Leptos 0.6 (WASM)
- **Routing**: Leptos Router
- **Styling**: Tailwind CSS via nightly feature
- **State**: Leptos signals + context
- **API Client**: Generated from OpenAPI spec

### Database
- **Primary**: PostgreSQL 16
- **Extensions**: uuid-ossp, pg_trgm, pgrowlocks

---

## Configuration Store Schema

```sql
-- Core configuration tables (stored in special _atlas schema)

CREATE TABLE _atlas.entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    label VARCHAR(200),
    plural_label VARCHAR(200),
    table_name VARCHAR(100),
    is_audit_enabled BOOLEAN DEFAULT true,
    is_soft_delete BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE _atlas.entity_fields (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID REFERENCES _atlas.entities(id),
    name VARCHAR(100) NOT NULL,
    label VARCHAR(200),
    field_type VARCHAR(50) NOT NULL,
    type_config JSONB,
    is_required BOOLEAN DEFAULT false,
    is_unique BOOLEAN DEFAULT false,
    is_read_only BOOLEAN DEFAULT false,
    default_value JSONB,
    help_text TEXT,
    display_order INT,
    UNIQUE(entity_id, name)
);

CREATE TABLE _atlas.workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id UUID REFERENCES _atlas.entities(id),
    name VARCHAR(100) NOT NULL,
    definition JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    version INT DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT now()
);

CREATE TABLE _atlas.config_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_name VARCHAR(100),
    version INT NOT NULL,
    config JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now(),
    created_by UUID
);

CREATE TABLE _atlas.audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100),
    entity_id UUID,
    action VARCHAR(20),
    old_data JSONB,
    new_data JSONB,
    changed_by UUID,
    changed_at TIMESTAMPTZ DEFAULT now(),
    session_id UUID
);
```

---

## Implementation Phases

### Phase 1: Foundation (Core Engine)
- [x] Project scaffolding & workspace setup
- [x] Schema engine (entity CRUD, field types)
- [x] Dynamic query builder
- [x] Hot-reload configuration system
- [x] Basic authentication
- [x] Audit logging

### Phase 2: Workflow Engine
- [x] State machine implementation
- [x] Transition guards & actions
- [x] Event publishing for state changes
- [x] Workflow history tracking

### Phase 3: Validation & Computed Fields
- [x] Rule engine with declarative rules
- [x] Formula parser & evaluator
- [x] Cross-field validation
- [x] Dynamic field dependencies

### Phase 4: Domain Modules
- [x] HCM core entities
- [x] Financials core entities
- [x] SCM core entities
- [x] CRM core entities

### Phase 5: Frontend
- [x] Leptos app scaffold
- [x] Dynamic form generator
- [x] List view builder
- [x] Dashboard framework

### Phase 6: Integration
- [x] NATS event bus integration
- [x] Inter-service communication
- [x] Report generation
- [x] Import/Export

---

## Security Model

### Row-Level Security
```yaml
security_policy: employees_by_org
entity: employees
rules:
  - condition: "user.organization_id == this.organization_id"
    actions: [read, update]
  - condition: "user.role == 'HR_ADMIN'"
    actions: [read, update, create, delete]
```

### Field-Level Security
```yaml
field: employees.salary
read_roles: [HR_ADMIN, PAYROLL_ADMIN]
write_roles: [PAYROLL_ADMIN]
```

---

## Getting Started

```bash
# Start the infrastructure
docker-compose up -d postgres nats

# Run the core gateway
cargo run -p atlas-gateway

# Initialize declarative schema
psql -h localhost -U atlas -d atlas < migrations/001_init.sql

# Load base configuration
./scripts/seed-config.sh

# Start frontend dev server
cd frontend && cargo run --app dashboard
```

---

## Example: Creating a New Business Object

```bash
# POST to the admin API to create a new entity
curl -X POST http://localhost:8080/api/admin/schema \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "entity": {
      "name": "expense_reports",
      "label": "Expense Report",
      "plural_label": "Expense Reports"
    },
    "fields": [
      {"name": "title", "type": "string", "required": true},
      {"name": "amount", "type": "decimal", "precision": 12, "scale": 2},
      {"name": "submitted_by", "type": "reference", "entity": "employees"},
      {"name": "status", "type": "enum", "values": ["draft", "submitted", "approved", "rejected"]}
    ],
    "workflow": {
      "initial_state": "draft",
      "transitions": [
        {"from": "draft", "to": "submitted", "action": "submit"},
        {"from": "submitted", "to": "approved", "action": "approve", "role": "finance_manager"},
        {"from": "submitted", "to": "rejected", "action": "reject", "role": "finance_manager"}
      ]
    }
  }'

# Now the new entity is immediately available
curl http://localhost:8080/api/v1/expense_reports
# Returns: {"data": [], "meta": {...}}

# No restart needed - the schema engine picked up the change
```

---

## Design Decisions

1. **Why Rust?**: Memory safety without GC, excellent performance, modern type system
2. **Why declarative?**: Business needs change constantly; code changes require releases
3. **Why microservices?**: Domain isolation, independent scaling, team autonomy
4. **Why PostgreSQL?**: ACID compliance, JSONB for flexible config, mature ecosystem
5. **Why Leptos?**: Full Rust stack, fine-grained reactivity, no JavaScript runtime needed

---

## License

MIT OR Apache-2.0
