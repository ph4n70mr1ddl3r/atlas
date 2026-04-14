import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useParams, useNavigate, useSearchParams } from 'react-router-dom'
import { useMemo } from 'react'
import type { ColumnDef } from '@tanstack/react-table'
import {
  listRecords,
  getEntitySchema,
  createRecord,
  type EntityDefinition,
  type FieldDefinition,
} from '@/lib/api'
import { DataTable } from '@/components/data-table'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Plus, Search, Download } from 'lucide-react'
import { formatDate } from '@/lib/utils'

const PAGE_SIZE = 20

export function EntityListPage() {
  const { entity = '' } = useParams<{ entity: string }>()
  const navigate = useNavigate()
  const [searchParams, setSearchParams] = useSearchParams()
  const [search, setSearch] = useState('')
  const page = Number(searchParams.get('page') ?? '1')

  const { data: schema } = useQuery({
    queryKey: ['schema', entity],
    queryFn: () => getEntitySchema(entity),
    enabled: !!entity,
  })

  const { data: records, isLoading } = useQuery({
    queryKey: ['records', entity, page],
    queryFn: () => listRecords(entity, { offset: (page - 1) * PAGE_SIZE, limit: PAGE_SIZE, search: search || undefined }),
    enabled: !!entity,
  })

  const columns = useMemo<ColumnDef<Record<string, unknown>>[]>(() => {
    if (!schema) return []
    return schema.fields
      .filter((f) => f.isSearchable)
      .slice(0, 6)
      .map((field) => ({
        accessorKey: field.name,
        header: field.label,
        cell: ({ getValue }) => {
          const val = getValue()
          if (val === null || val === undefined) return <span className="text-muted-foreground">—</span>
          if (field.fieldType.type === 'date') return formatDate(val as string)
          if (field.fieldType.type === 'enum') {
            const str = String(val)
            return <Badge variant="secondary">{str}</Badge>
          }
          if (field.fieldType.type === 'boolean') {
            return val ? <Badge variant="success">Yes</Badge> : <Badge variant="secondary">No</Badge>
          }
          const str = String(val)
          return <span className="truncate max-w-[200px] block">{str}</span>
        },
      }))
  }, [schema])

  // Add workflow_state column if entity has a workflow
  const displayColumns = useMemo(() => {
    if (!schema?.workflow) return columns
    return [
      ...columns,
      {
        id: 'workflow_state',
        header: 'Status',
        accessorKey: 'workflow_state',
        cell: ({ getValue }) => {
          const state = getValue() as string | null
          if (!state) return null
          return <Badge variant="info">{state.replace(/_/g, ' ')}</Badge>
        },
      } as ColumnDef<Record<string, unknown>>,
    ]
  }, [columns, schema?.workflow])

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight capitalize">
            {schema?.pluralLabel ?? entity.replace(/_/g, ' ')}
          </h1>
          {schema?.description && <p className="text-muted-foreground">{schema.description}</p>}
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => navigate(`/export/${entity}`)}
          >
            <Download className="mr-2 h-4 w-4" /> Export
          </Button>
          <Button size="sm" onClick={() => navigate(`/${entity}?new=true`)}>
            <Plus className="mr-2 h-4 w-4" /> Add {schema?.label ?? 'Record'}
          </Button>
        </div>
      </div>

      {/* Search */}
      <div className="flex gap-2">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder={`Search ${schema?.pluralLabel ?? entity}…`}
            className="pl-9"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && setSearchParams({ page: '1' })}
          />
        </div>
      </div>

      {/* Table */}
      <Card>
        <CardContent className="p-0">
          <DataTable
            columns={displayColumns}
            data={records?.data ?? []}
            total={records?.meta?.total}
            pageCount={records?.meta ? Math.ceil(records.meta.total / PAGE_SIZE) : undefined}
            isLoading={isLoading}
            onRowClick={(row) => {
              const id = (row as Record<string, unknown>).id as string
              if (id) navigate(`/${entity}/${id}`)
            }}
          />
        </CardContent>
      </Card>

      {/* Create dialog (shown when ?new=true) */}
      <Dialog open={searchParams.get('new') === 'true'} onOpenChange={(open) => !open && setSearchParams({})}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>New {schema?.label ?? 'Record'}</DialogTitle>
          </DialogHeader>
          <EntityForm entity={entity} schema={schema} onDone={() => setSearchParams({})} />
        </DialogContent>
      </Dialog>
    </div>
  )
}

function EntityForm({
  entity,
  schema,
  onDone,
}: {
  entity: string
  schema?: EntityDefinition
  onDone: () => void
}) {
  const [values, setValues] = useState<Record<string, unknown>>({})
  const [saving, setSaving] = useState(false)

  if (!schema) return null

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setSaving(true)
    try {
      await createRecord(entity, values)
      onDone()
    } catch (err) {
      alert(`Failed to create: ${err}`)
    } finally {
      setSaving(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4 max-h-[60vh] overflow-y-auto pr-2">
      {schema?.fields
        .filter((f: FieldDefinition) => !f.isReadOnly && !['id', 'created_at', 'updated_at', 'deleted_at', 'organization_id'].includes(f.name))
        .map((field: FieldDefinition) => (
          <div key={field.name} className="space-y-1">
            <label className="text-sm font-medium">
              {field.label}
              {field.isRequired && <span className="text-destructive ml-1">*</span>}
            </label>
            {field.fieldType.type === 'enum' ? (
              <select
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                value={(values[field.name] as string) ?? ''}
                onChange={(e) => setValues({ ...values, [field.name]: e.target.value })}
              >
                <option value="">Select…</option>
                {field.fieldType.values.map((v: string) => (
                  <option key={v} value={v}>
                    {v}
                  </option>
                ))}
              </select>
            ) : field.fieldType.type === 'boolean' ? (
              <input
                type="checkbox"
                checked={(values[field.name] as boolean) ?? false}
                onChange={(e) => setValues({ ...values, [field.name]: e.target.checked })}
                className="h-4 w-4"
              />
            ) : field.fieldType.type === 'rich_text' ? (
              <textarea
                className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                value={(values[field.name] as string) ?? ''}
                onChange={(e) => setValues({ ...values, [field.name]: e.target.value })}
                placeholder={field.placeholder}
              />
            ) : (
              <Input
                type={field.fieldType.type === 'date' ? 'date' : field.fieldType.type === 'integer' || field.fieldType.type === 'decimal' ? 'number' : 'text'}
                value={(values[field.name] as string) ?? ''}
                onChange={(e) => setValues({ ...values, [field.name]: e.target.value })}
                placeholder={field.placeholder}
                required={field.isRequired}
              />
            )}
          </div>
        ))}
      <div className="flex justify-end gap-2 pt-4">
        <Button type="button" variant="outline" onClick={onDone}>
          Cancel
        </Button>
        <Button type="submit" disabled={saving}>
          {saving ? 'Saving…' : 'Create'}
        </Button>
      </div>
    </form>
  )
}
