import { useQuery, useQueryClient } from '@tanstack/react-query'
import { getAdminConfig, getEntitySchema, createEntity, deleteEntity, type EntityDefinition, type FieldDefinition, type FieldTypeUnion } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Settings, Trash2, RefreshCw, Database, Plus, Eye, Edit, ChevronDown, ChevronUp, X } from 'lucide-react'
import { useState } from 'react'
import { Link } from 'react-router-dom'

export function AdminPage() {
  const qc = useQueryClient()
  const [createOpen, setCreateOpen] = useState(false)
  const [expandedEntity, setExpandedEntity] = useState<string | null>(null)

  const { data: config, isLoading, refetch, isFetching } = useQuery({
    queryKey: ['admin-config'],
    queryFn: getAdminConfig,
  })

  const handleDelete = async (entity: string) => {
    if (!confirm(`Delete entity "${entity}"? This will remove the schema definition. The data table will be preserved.`)) return
    try {
      await deleteEntity(entity, false)
      refetch()
    } catch (err) {
      alert(`Failed to delete: ${err}`)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Admin</h1>
          <p className="text-muted-foreground">Manage entity schemas, workflows, and system configuration</p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => refetch()} disabled={isFetching}>
            <RefreshCw className={`mr-2 h-4 w-4 ${isFetching ? 'animate-spin' : ''}`} /> Refresh
          </Button>
          <Button onClick={() => setCreateOpen(true)}>
            <Plus className="mr-2 h-4 w-4" /> New Entity
          </Button>
        </div>
      </div>

      {/* System info */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Schema Version</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{config?.version ?? '—'}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Entity Count</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{config?.entities?.length ?? 0}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Database className="h-4 w-4" /> Infrastructure
            </CardTitle>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground">
            <p>PostgreSQL 16 + NATS</p>
            <p className="text-xs">Hot-reload enabled</p>
          </CardContent>
        </Card>
      </div>

      {/* Entity list */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" /> Entity Definitions
          </CardTitle>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="space-y-3">
              {[1, 2, 3].map((i) => (
                <div key={i} className="h-16 animate-pulse rounded bg-muted" />
              ))}
            </div>
          ) : (
            <div className="space-y-2">
              {(config?.entities ?? []).map((entity) => (
                <EntityRow
                  key={entity}
                  entity={entity}
                  expanded={expandedEntity === entity}
                  onToggle={() => setExpandedEntity(expandedEntity === entity ? null : entity)}
                  onDelete={() => handleDelete(entity)}
                />
              ))}
              {config?.entities?.length === 0 && (
                <div className="py-12 text-center">
                  <Database className="mx-auto h-12 w-12 text-muted-foreground/40 mb-4" />
                  <p className="text-muted-foreground mb-2">No entities defined yet.</p>
                  <Button variant="outline" onClick={() => setCreateOpen(true)}>
                    <Plus className="mr-2 h-4 w-4" /> Create your first entity
                  </Button>
                </div>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Create entity dialog */}
      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent className="max-w-3xl max-h-[85vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle>Create Entity</DialogTitle>
          </DialogHeader>
          <CreateEntityForm
            onDone={() => {
              setCreateOpen(false)
              refetch()
            }}
          />
        </DialogContent>
      </Dialog>
    </div>
  )
}

function EntityRow({
  entity,
  expanded,
  onToggle,
  onDelete,
}: {
  entity: string
  expanded: boolean
  onToggle: () => void
  onDelete: () => void
}) {
  const { data: schema } = useQuery({
    queryKey: ['schema', entity],
    queryFn: () => getEntitySchema(entity),
  })

  return (
    <div className="rounded-lg border">
      <div className="flex items-center justify-between p-4">
        <div className="flex items-center gap-4">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-lg font-medium">
            {schema?.icon ? schema.icon.charAt(0).toUpperCase() : entity.charAt(0).toUpperCase()}
          </div>
          <div>
            <p className="font-medium">{schema?.label ?? entity}</p>
            <p className="text-xs text-muted-foreground">
              {entity} · {schema?.fields?.length ?? '?'} fields
              {schema?.tableName && ` · ${schema.tableName}`}
            </p>
          </div>
          <div className="flex gap-1">
            {schema?.workflow && <Badge variant="info">workflow</Badge>}
            {schema?.isAuditEnabled && <Badge variant="secondary">audit</Badge>}
            {schema?.isSoftDelete && <Badge variant="outline">soft-delete</Badge>}
          </div>
        </div>
        <div className="flex gap-2">
          <Link to={`/${entity}`}>
            <Button variant="ghost" size="icon" title="View records">
              <Eye className="h-4 w-4" />
            </Button>
          </Link>
          <Button variant="ghost" size="icon" onClick={onToggle} title={expanded ? 'Collapse' : 'Expand'}>
            {expanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
          </Button>
          <Button variant="ghost" size="icon" onClick={onDelete} title="Delete entity">
            <Trash2 className="h-4 w-4 text-destructive" />
          </Button>
        </div>
      </div>

      {/* Expanded field details */}
      {expanded && schema && (
        <div className="border-t px-4 py-3 bg-muted/20">
          <p className="text-sm font-medium mb-2">Fields ({schema.fields.length})</p>
          <div className="grid gap-1.5 sm:grid-cols-2 lg:grid-cols-3">
            {schema.fields.map((field) => (
              <div key={field.name} className="flex items-center gap-2 text-xs rounded border bg-background px-2 py-1.5">
                <span className="font-mono font-medium">{field.name}</span>
                <Badge variant="outline" className="text-[9px] px-1 py-0">{field.fieldType.type}</Badge>
                {field.isRequired && <span className="text-destructive text-[9px]">REQ</span>}
                {field.isUnique && <span className="text-blue-500 text-[9px]">UQ</span>}
                <span className="text-muted-foreground truncate">{field.label}</span>
              </div>
            ))}
          </div>
          {schema.workflow && (
            <>
              <p className="text-sm font-medium mt-3 mb-2">Workflow: {schema.workflow.name}</p>
              <div className="flex flex-wrap gap-1">
                {schema.workflow.states.map((s) => (
                  <Badge key={s.name} variant={s.stateType === 'initial' ? 'success' : s.stateType === 'final' ? 'destructive' : 'secondary'}>
                    {s.name.replace(/_/g, ' ')}
                  </Badge>
                ))}
              </div>
              <div className="mt-2 flex flex-wrap gap-1">
                {schema.workflow.transitions.map((t) => (
                  <span key={t.name} className="text-[10px] text-muted-foreground">
                    {t.fromState.replace(/_/g, ' ')} → {t.toState.replace(/_/g, ' ')} ({t.action})
                  </span>
                ))}
              </div>
            </>
          )}
        </div>
      )}
    </div>
  )
}

// ── Create Entity Form ──────────────────────────────────────

function CreateEntityForm({ onDone }: { onDone: () => void }) {
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState('')

  // Entity metadata
  const [name, setName] = useState('')
  const [label, setLabel] = useState('')
  const [pluralLabel, setPluralLabel] = useState('')
  const [description, setDescription] = useState('')

  // Fields
  const [fields, setFields] = useState<FieldDefinition[]>([])
  const [newFieldName, setNewFieldName] = useState('')
  const [newFieldType, setNewFieldType] = useState<string>('string')
  const [newFieldRequired, setNewFieldRequired] = useState(false)

  // Workflow
  const [addWorkflow, setAddWorkflow] = useState(false)
  const [workflowStates, setWorkflowStates] = useState('draft,submitted,approved')
  const [workflowTransitions, setWorkflowTransitions] = useState('draft→submitted:submit,submitted→approved:approve')

  const fieldTypes = [
    'string', 'integer', 'decimal', 'boolean', 'date', 'date_time',
    'enum', 'reference', 'email', 'url', 'rich_text', 'json',
  ]

  const addField = () => {
    if (!newFieldName.trim()) return
    const ft = buildFieldType(newFieldType)
    const field: FieldDefinition = {
      name: newFieldName.trim().toLowerCase().replace(/\s+/g, '_'),
      label: newFieldName.trim().replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase()),
      fieldType: ft,
      isRequired: newFieldRequired,
      isUnique: false,
      isReadOnly: false,
      isSearchable: true,
      displayOrder: fields.length,
      validations: [],
      visibility: { condition: undefined, roles: [], hidden: false },
    }
    setFields([...fields, field])
    setNewFieldName('')
    setNewFieldRequired(false)
  }

  const removeField = (idx: number) => {
    setFields(fields.filter((_, i) => i !== idx))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setSaving(true)
    setError('')

    try {
      const entityDef: EntityDefinition = {
        name: name.trim().toLowerCase().replace(/\s+/g, '_'),
        label: label.trim() || name.trim(),
        pluralLabel: pluralLabel.trim() || (label.trim() || name.trim()) + 's',
        description: description.trim() || undefined,
        fields,
        indexes: [],
        isAuditEnabled: true,
        isSoftDelete: true,
        metadata: {},
      }

      if (addWorkflow) {
        const states = workflowStates.split(',').map((s) => s.trim()).filter(Boolean)
        const transitionDefs = workflowTransitions.split(',').map((t) => {
          const match = t.trim().match(/^(\w+)→(\w+):(\w+)$/)
          if (!match) return null
          return {
            name: match[3],
            fromState: match[1],
            toState: match[2],
            action: match[3],
            actionLabel: match[3].replace(/_/g, ' '),
            guards: [],
            requiredRoles: [] as string[],
            entryActions: [],
            metadata: {},
          }
        }).filter(Boolean) as EntityDefinition['workflow'] extends undefined ? never : NonNullable<EntityDefinition['workflow']>['transitions']

        if (states.length > 0) {
          entityDef.workflow = {
            name: `${entityDef.name}_workflow`,
            initialState: states[0],
            states: states.map((s, i) => ({
              name: s,
              label: s.replace(/_/g, ' '),
              stateType: i === 0 ? 'initial' as const : i === states.length - 1 ? 'final' as const : 'working' as const,
              entryActions: [],
              exitActions: [],
              metadata: {},
            })),
            transitions: transitionDefs,
            isActive: true,
          }
        }
      }

      await createEntity(entityDef)
      onDone()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setSaving(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {error && (
        <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
          {error}
        </div>
      )}

      {/* Entity metadata */}
      <div className="space-y-4">
        <h3 className="font-medium text-sm text-muted-foreground uppercase tracking-wide">Entity Details</h3>
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-1.5">
            <Label>Entity Name *</Label>
            <Input
              placeholder="e.g. expense_reports"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
            />
            <p className="text-xs text-muted-foreground">Lowercase, underscores. Used as API endpoint.</p>
          </div>
          <div className="space-y-1.5">
            <Label>Label *</Label>
            <Input
              placeholder="e.g. Expense Report"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label>Plural Label</Label>
            <Input
              placeholder="e.g. Expense Reports"
              value={pluralLabel}
              onChange={(e) => setPluralLabel(e.target.value)}
            />
          </div>
          <div className="space-y-1.5">
            <Label>Description</Label>
            <Input
              placeholder="Optional description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </div>
        </div>
      </div>

      {/* Fields */}
      <div className="space-y-4">
        <h3 className="font-medium text-sm text-muted-foreground uppercase tracking-wide">Fields</h3>

        {/* Existing fields */}
        {fields.length > 0 && (
          <div className="space-y-2">
            {fields.map((field, idx) => (
              <div key={idx} className="flex items-center gap-2 rounded border bg-muted/30 px-3 py-2">
                <span className="font-mono text-sm font-medium">{field.name}</span>
                <Badge variant="outline" className="text-[10px]">{field.fieldType.type}</Badge>
                {field.isRequired && <Badge variant="secondary" className="text-[10px]">required</Badge>}
                <span className="text-sm text-muted-foreground flex-1">{field.label}</span>
                <button type="button" onClick={() => removeField(idx)} className="text-muted-foreground hover:text-destructive">
                  <X className="h-4 w-4" />
                </button>
              </div>
            ))}
          </div>
        )}

        {/* Add field */}
        <div className="flex items-end gap-2">
          <div className="flex-1 space-y-1.5">
            <Label className="text-xs">Field Name</Label>
            <Input
              placeholder="e.g. title"
              value={newFieldName}
              onChange={(e) => setNewFieldName(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), addField())}
            />
          </div>
          <div className="w-36 space-y-1.5">
            <Label className="text-xs">Type</Label>
            <select
              className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
              value={newFieldType}
              onChange={(e) => setNewFieldType(e.target.value)}
            >
              {fieldTypes.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </div>
          <label className="flex items-center gap-1.5 pb-2">
            <input
              type="checkbox"
              checked={newFieldRequired}
              onChange={(e) => setNewFieldRequired(e.target.checked)}
              className="h-4 w-4"
            />
            <span className="text-xs">Required</span>
          </label>
          <Button type="button" variant="outline" size="sm" onClick={addField} disabled={!newFieldName.trim()}>
            <Plus className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Workflow */}
      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <input
            type="checkbox"
            id="add-workflow"
            checked={addWorkflow}
            onChange={(e) => setAddWorkflow(e.target.checked)}
            className="h-4 w-4"
          />
          <Label htmlFor="add-workflow">Add Workflow</Label>
        </div>
        {addWorkflow && (
          <div className="space-y-3 border-l-2 border-primary/20 pl-4">
            <div className="space-y-1.5">
              <Label className="text-xs">States (comma-separated)</Label>
              <Input
                placeholder="draft,submitted,approved,rejected"
                value={workflowStates}
                onChange={(e) => setWorkflowStates(e.target.value)}
              />
            </div>
            <div className="space-y-1.5">
              <Label className="text-xs">Transitions (from→to:action, comma-separated)</Label>
              <Textarea
                placeholder="draft→submitted:submit,submitted→approved:approve,submitted→rejected:reject"
                value={workflowTransitions}
                onChange={(e) => setWorkflowTransitions(e.target.value)}
                rows={3}
              />
              <p className="text-xs text-muted-foreground">Format: from_state→to_state:action_name</p>
            </div>
          </div>
        )}
      </div>

      {/* Submit */}
      <div className="flex justify-end gap-2 pt-4 border-t">
        <Button type="button" variant="outline" onClick={onDone}>Cancel</Button>
        <Button type="submit" disabled={saving || !name.trim()}>
          {saving ? 'Creating…' : 'Create Entity'}
        </Button>
      </div>
    </form>
  )
}

function buildFieldType(type: string): FieldTypeUnion {
  switch (type) {
    case 'string': return { type: 'string', maxLength: 255 }
    case 'integer': return { type: 'integer' }
    case 'decimal': return { type: 'decimal', precision: 12, scale: 2 }
    case 'boolean': return { type: 'boolean' }
    case 'date': return { type: 'date' }
    case 'date_time': return { type: 'date_time' }
    case 'email': return { type: 'email' }
    case 'url': return { type: 'url' }
    case 'rich_text': return { type: 'rich_text' }
    case 'json': return { type: 'json' }
    case 'enum': return { type: 'enum', values: ['option1', 'option2'] }
    case 'reference': return { type: 'reference', entity: 'users' }
    default: return { type: 'string', maxLength: 255 }
  }
}
