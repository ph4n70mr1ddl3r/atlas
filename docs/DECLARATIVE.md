# Declarative Patterns in Atlas

## Defining Entities

Entities are defined using a JSON or YAML schema that specifies fields, types, and relationships.

Example:
```json
{
  "name": "purchase_orders",
  "label": "Purchase Order",
  "fields": [
    {
      "name": "po_number",
      "label": "PO Number",
      "field_type": { "type": "string" },
      "is_required": true,
      "is_unique": true
    }
  ]
}
```

## Defining Workflows

Workflows are state machines that define how an entity moves through different statuses.

Example:
```json
{
  "name": "po_approval",
  "initial_state": "draft",
  "states": [
    { "name": "draft", "label": "Draft", "state_type": "initial" },
    { "name": "approved", "label": "Approved", "state_type": "final" }
  ],
  "transitions": [
    {
      "name": "approve",
      "from_state": "draft",
      "to_state": "approved",
      "action": "approve"
    }
  ]
}
```
