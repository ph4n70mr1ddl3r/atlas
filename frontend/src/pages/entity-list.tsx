import { useState, useMemo, useCallback } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useParams, useNavigate, useSearchParams } from 'react-router-dom'
import type { ColumnDef, SortingState } from '@tanstack/react-table'
import {
  listRecordsAdvanced,
  getEntitySchema,
  createRecord,
  exportCsv,
  type FilterExpression,
} from '@/lib/api'
import { DataTable } from '@/components/data-table'
import { DynamicForm } from '@/components/dynamic-form'
import { FilterPanel } from '@/components/filter-panel'
import { BulkOperations } from '@/components/bulk-operations'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import {
  Plus, Search, Download, X, FileSpreadsheet, CheckSquare,
} from 'lucide-react'
import { formatDate } from '@/lib/utils'

const PAGE_SIZE = 20

export function EntityListPage() {
  const { entity = '' } = useParams<{ entity: string }>()
  const navigate = useNavigate()
  const [searchParams, setSearchParams] = useSearchParams()
  const qc = useQueryClient()
  const [search, setSearch] = useState('')
  const [filterField, setFilterField] = useState<string | null>(null)
  const [filterValue, setFilterValue] = useState('')
  const [structuredFilter, setStructuredFilter] = useState<FilterExpression | null>(null)
  const [selectedRows, setSelectedRows] = useState<Set<string>>(new Set())
  const page = Number(searchParams.get('page') ?? '1') - 1 // 0-based

  const { data: schema } = useQuery({
    queryKey: ['schema', entity],
    queryFn: () => getEntitySchema(entity),
    enabled: !!entity,
  })

  const [sorting, setSorting] = useState<SortingState>([])

  const sortParam = sorting.length > 0 ? sorting[0].id : undefined
  const orderParam = sorting.length > 0 ? (sorting[0].desc ? 'desc' : 'asc') : undefined

  const { data: records, isLoading } = useQuery({
    queryKey: ['records', entity, page, search, sortParam, orderParam, structuredFilter],
    queryFn: () =>
      listRecordsAdvanced(entity, {
        offset: page * PAGE_SIZE,
        limit: PAGE_SIZE,
        search: search || undefined,
        sort: sortParam,
        order: orderParam,
        filter: structuredFilter ?? undefined,
      }),
    enabled: !!entity,
  })

  const columns = useMemo<ColumnDef<Record<string, unknown>>[]>(() => {
    if (!schema) return []
    const searchable = schema.fields.filter((f) => f.isSearchable).slice(0, 6)
    const cols: ColumnDef<Record<string, unknown>>[] = searchable.map((field) => ({
      accessorKey: field.name,
      header: field.label,
      enableSorting: true,
      cell: ({ getValue }) => {
        const val = getValue()
        if (val === null || val === undefined) return <span className="text-muted-foreground">—</span>
        if (field.fieldType.type === 'date') return formatDate(val as string)
        if (field.fieldType.type === 'date_time') return formatDate(val as string)
        if (field.fieldType.type === 'enum') {
          return <Badge variant="secondary">{String(val).replace(/_/g, ' ')}</Badge>
        }
        if (field.fieldType.type === 'boolean') {
          return val ? <Badge variant="success">Yes</Badge> : <Badge variant="secondary">No</Badge>
        }
        if (field.fieldType.type === 'currency' || field.fieldType.type === 'decimal') {
          return <span className="font-mono">{Number(val).toLocaleString()}</span>
        }
        return <span className="truncate max-w-[200px] block">{String(val)}</span>
      },
    }))

    // Add workflow_state column if entity has a workflow
    if (schema.workflow) {
      cols.push({
        id: 'workflow_state',
        header: 'Status',
        accessorKey: 'workflow_state',
        enableSorting: true,
        cell: ({ getValue }) => {
          const state = getValue() as string | null
          if (!state) return null
          return <Badge variant="info">{state.replace(/_/g, ' ')}</Badge>
        },
      })
    }

    return cols
  }, [schema])

  const showCreateDialog = searchParams.get('new') === 'true'
  const pageCount = records?.meta ? Math.ceil(records.meta.total / PAGE_SIZE) : undefined

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight capitalize">
            {schema?.pluralLabel ?? entity.replace(/_/g, ' ')}
          </h1>
          {schema?.description && <p className="text-muted-foreground mt-1">{schema.description}</p>}
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={async () => {
              try {
                const blob = await exportCsv(entity, {
                  fields: schema?.fields.filter((f) => f.isSearchable).map((f) => f.name),
                  filter: structuredFilter ?? undefined,
                })
                const url = URL.createObjectURL(blob)
                const a = document.createElement('a')
                a.href = url
                a.download = `${entity}_export.csv`
                a.click()
                URL.revokeObjectURL(url)
              } catch (err) {
                alert(`Export failed: ${err}`)
              }
            }}
          >
            <FileSpreadsheet className="mr-2 h-4 w-4" /> CSV
          </Button>
          <Button size="sm" onClick={() => setSearchParams({ new: 'true' })}>
            <Plus className="mr-2 h-4 w-4" /> Add {schema?.label ?? 'Record'}
          </Button>
        </div>
      </div>

      {/* Structured Filter Panel */}
      {schema && (
        <FilterPanel
          fields={schema.fields.map((f) => ({ name: f.name, label: f.label, type: f.fieldType.type }))}
          filter={structuredFilter}
          onChange={setStructuredFilter}
        />
      )}

      {/* Search & Filter bar */}
      <div className="flex gap-2 items-center">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder={`Search ${schema?.pluralLabel ?? entity}…`}
            className="pl-9"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value)
              setSearchParams((prev) => { prev.delete('page'); return prev })
            }}
          />
          {search && (
            <button
              className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
              onClick={() => setSearch('')}
            >
              <X className="h-3 w-3" />
            </button>
          )}
        </div>

        {records?.meta && (
          <span className="text-sm text-muted-foreground ml-auto">
            {records.meta.total.toLocaleString()} record{records.meta.total !== 1 ? 's' : ''}
          </span>
        )}

        <BulkOperations
          entity={entity}
          selectedIds={Array.from(selectedRows)}
          filter={structuredFilter ?? undefined}
          onCompleted={() => {
            setSelectedRows(new Set())
            qc.invalidateQueries({ queryKey: ['records', entity] })
          }}
        />
      </div>

      {/* Table */}
      <Card>
        <CardContent className="p-0">
          <DataTable
            columns={columns}
            data={records?.data ?? []}
            total={records?.meta?.total}
            pageCount={pageCount}
            pageIndex={page}
            isLoading={isLoading}
            onRowClick={(row) => {
              const id = (row as Record<string, unknown>).id as string
              if (id) navigate(`/${entity}/${id}`)
            }}
            onSortingChange={setSorting}
            onPageChange={(p) => {
              setSearchParams({ page: String(p + 1) })
            }}
          />
        </CardContent>
      </Card>

      {/* Create dialog with dynamic form */}
      <Dialog open={showCreateDialog} onOpenChange={(open) => !open && setSearchParams({})}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>New {schema?.label ?? 'Record'}</DialogTitle>
          </DialogHeader>
          {schema && (
            <DynamicForm
              schema={schema}
              onSubmit={async (values) => {
                await createRecord(entity, values)
                setSearchParams({})
                qc.invalidateQueries({ queryKey: ['records', entity] })
              }}
              onCancel={() => setSearchParams({})}
              submitLabel="Create"
            />
          )}
        </DialogContent>
      </Dialog>
    </div>
  )
}
