import { useQuery, useQueryClient } from '@tanstack/react-query'
import { getAdminConfig, getEntitySchema, deleteEntity } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Settings, Trash2, RefreshCw, Database } from 'lucide-react'

export function AdminPage() {
  const qc = useQueryClient()
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
        <Button variant="outline" onClick={() => refetch()} disabled={isFetching}>
          <RefreshCw className={`mr-2 h-4 w-4 ${isFetching ? 'animate-spin' : ''}`} /> Refresh
        </Button>
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
                <EntityRow key={entity} entity={entity} onDelete={() => handleDelete(entity)} />
              ))}
              {config?.entities?.length === 0 && (
                <p className="py-8 text-center text-muted-foreground">No entities defined yet.</p>
              )}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

function EntityRow({ entity, onDelete }: { entity: string; onDelete: () => void }) {
  const { data: schema } = useQuery({
    queryKey: ['schema', entity],
    queryFn: () => getEntitySchema(entity),
  })

  return (
    <div className="flex items-center justify-between rounded-lg border p-4">
      <div className="flex items-center gap-4">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10 text-lg">
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
        </div>
      </div>
      <div className="flex gap-2">
        <Button variant="ghost" size="icon" onClick={onDelete} title="Delete entity">
          <Trash2 className="h-4 w-4 text-destructive" />
        </Button>
      </div>
    </div>
  )
}
