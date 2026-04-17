import { useState, useCallback } from 'react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Plus, X, ChevronDown, ChevronUp } from 'lucide-react'
import type { FilterExpression } from '@/lib/api'

interface FilterPanelProps {
  fields: Array<{ name: string; label: string; type: string }>
  filter: FilterExpression | null
  onChange: (filter: FilterExpression | null) => void
}

const OPERATORS = [
  { value: 'eq', label: 'equals' },
  { value: 'ne', label: 'not equals' },
  { value: 'gt', label: 'greater than' },
  { value: 'gte', label: 'greater or equal' },
  { value: 'lt', label: 'less than' },
  { value: 'lte', label: 'less or equal' },
  { value: 'contains', label: 'contains' },
  { value: 'starts_with', label: 'starts with' },
  { value: 'ends_with', label: 'ends with' },
  { value: 'is_null', label: 'is empty' },
  { value: 'is_not_null', label: 'is not empty' },
  { value: 'in', label: 'in list' },
  { value: 'not_in', label: 'not in list' },
  { value: 'between', label: 'between' },
]

function FilterConditionRow({
  condition,
  fields,
  onChange,
  onRemove,
}: {
  condition: Extract<FilterExpression, { type: 'condition' }>
  fields: FilterPanelProps['fields']
  onChange: (c: Extract<FilterExpression, { type: 'condition' }>) => void
  onRemove: () => void
}) {
  const selectedField = fields.find(f => f.name === condition.field)
  const needsValue = !['is_null', 'is_not_null'].includes(condition.operator)

  return (
    <div className="flex items-center gap-2 flex-wrap">
      <select
        value={condition.field}
        onChange={(e) => onChange({ ...condition, field: e.target.value })}
        className="h-9 rounded-md border border-input bg-background px-3 text-sm"
      >
        <option value="">Select field...</option>
        {fields.map(f => (
          <option key={f.name} value={f.name}>{f.label}</option>
        ))}
      </select>

      <select
        value={condition.operator}
        onChange={(e) => onChange({ ...condition, operator: e.target.value })}
        className="h-9 rounded-md border border-input bg-background px-3 text-sm"
      >
        {OPERATORS.map(op => (
          <option key={op.value} value={op.value}>{op.label}</option>
        ))}
      </select>

      {needsValue && (
        <Input
          value={typeof condition.value === 'string' ? condition.value : JSON.stringify(condition.value)}
          onChange={(e) => onChange({ ...condition, value: e.target.value })}
          placeholder="Value..."
          className="h-9 w-48"
        />
      )}

      <Button variant="ghost" size="icon" onClick={onRemove} className="h-9 w-9 shrink-0">
        <X className="h-4 w-4" />
      </Button>
    </div>
  )
}

function FilterGroup({
  group,
  fields,
  depth,
  onChange,
  onRemove,
}: {
  group: Extract<FilterExpression, { type: 'and' | 'or' }>
  fields: FilterPanelProps['fields']
  depth: number
  onChange: (g: Extract<FilterExpression, { type: 'and' | 'or' }>) => void
  onRemove: () => void
}) {
  const toggleType = () => {
    const newType = group.type === 'and' ? 'or' : 'and'
    onChange({ ...group, type: newType })
  }

  const addCondition = () => {
    onChange({
      ...group,
      conditions: [
        ...group.conditions,
        { type: 'condition', field: '', operator: 'eq', value: '' },
      ],
    })
  }

  const addGroup = () => {
    onChange({
      ...group,
      conditions: [
        ...group.conditions,
        { type: 'and', conditions: [{ type: 'condition', field: '', operator: 'eq', value: '' }] },
      ],
    })
  }

  const updateChild = (idx: number, child: FilterExpression) => {
    const newConditions = [...group.conditions]
    newConditions[idx] = child
    onChange({ ...group, conditions: newConditions })
  }

  const removeChild = (idx: number) => {
    const newConditions = group.conditions.filter((_, i) => i !== idx)
    if (newConditions.length === 0) {
      onRemove()
    } else {
      onChange({ ...group, conditions: newConditions })
    }
  }

  const borderClass = depth % 2 === 0 ? 'border-l-blue-300 bg-blue-50/30' : 'border-l-green-300 bg-green-50/30'

  return (
    <div className={`border-l-2 ${borderClass} pl-3 py-2 space-y-2`}>
      <div className="flex items-center gap-2">
        <button
          onClick={toggleType}
          className="text-xs font-bold px-2 py-1 rounded bg-primary/10 text-primary hover:bg-primary/20 transition-colors"
        >
          {group.type.toUpperCase()}
        </button>
        <span className="text-xs text-muted-foreground">
          {group.conditions.length} condition{group.conditions.length !== 1 ? 's' : ''}
        </span>
        <div className="flex-1" />
        {depth > 0 && (
          <Button variant="ghost" size="icon" onClick={onRemove} className="h-7 w-7">
            <X className="h-3 w-3" />
          </Button>
        )}
      </div>

      {group.conditions.map((child, idx) => {
        if (child.type === 'condition') {
          return (
            <FilterConditionRow
              key={idx}
              condition={child}
              fields={fields}
              onChange={(c) => updateChild(idx, c)}
              onRemove={() => removeChild(idx)}
            />
          )
        }
        return (
          <FilterGroup
            key={idx}
            group={child as Extract<FilterExpression, { type: 'and' | 'or' }>}
            fields={fields}
            depth={depth + 1}
            onChange={(g) => updateChild(idx, g)}
            onRemove={() => removeChild(idx)}
          />
        )
      })}

      <div className="flex gap-2">
        <Button variant="outline" size="sm" onClick={addCondition} className="h-7 text-xs">
          <Plus className="h-3 w-3 mr-1" /> Condition
        </Button>
        <Button variant="outline" size="sm" onClick={addGroup} className="h-7 text-xs">
          <Plus className="h-3 w-3 mr-1" /> Group
        </Button>
      </div>
    </div>
  )
}

export function FilterPanel({ fields, filter, onChange }: FilterPanelProps) {
  const [isOpen, setIsOpen] = useState(false)

  const countConditions = (f: FilterExpression | null): number => {
    if (!f) return 0
    if (f.type === 'condition') return 1
    return f.conditions.reduce((acc, c) => acc + countConditions(c), 0)
  }

  const activeCount = countConditions(filter)

  const startFilter = () => {
    onChange({
      type: 'and',
      conditions: [{ type: 'condition', field: '', operator: 'eq', value: '' }],
    })
  }

  const clearFilter = () => {
    onChange(null)
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={() => setIsOpen(!isOpen)}
          className="gap-1"
        >
          {isOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
          Filters
          {activeCount > 0 && (
            <Badge variant="secondary" className="ml-1 h-5 px-1.5 text-[10px]">
              {activeCount}
            </Badge>
          )}
        </Button>
        {activeCount > 0 && (
          <Button variant="ghost" size="sm" onClick={clearFilter} className="text-xs text-muted-foreground">
            Clear all
          </Button>
        )}
      </div>

      {isOpen && (
        <div className="rounded-lg border bg-card p-4">
          {filter && filter.type !== 'condition' ? (
            <FilterGroup
              group={filter as Extract<FilterExpression, { type: 'and' | 'or' }>}
              fields={fields}
              depth={0}
              onChange={(g) => onChange(g)}
              onRemove={clearFilter}
            />
          ) : filter && filter.type === 'condition' ? (
            // Single condition at root — wrap in AND
            <FilterGroup
              group={{ type: 'and', conditions: [filter] }}
              fields={fields}
              depth={0}
              onChange={(g) => onChange(g)}
              onRemove={clearFilter}
            />
          ) : (
            <Button variant="outline" size="sm" onClick={startFilter}>
              <Plus className="h-4 w-4 mr-1" /> Add Filter
            </Button>
          )}
        </div>
      )}
    </div>
  )
}
