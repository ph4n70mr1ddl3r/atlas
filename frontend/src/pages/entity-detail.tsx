import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useParams, useNavigate } from 'react-router-dom'
import { useState } from 'react'
import {
  getRecord,
  getEntitySchema,
  getTransitions,
  executeAction,
  getRecordHistory,
  updateRecord,
  deleteRecord,
} from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { ArrowLeft, Edit, Trash2, Play } from 'lucide-react'
import { formatDateTime, formatDate } from '@/lib/utils'

export function EntityDetailPage() {
  const { entity = '', id = '' } = useParams<{ entity: string; id: string }>()
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [editing, setEditing] = useState(false)
  const [editValues, setEditValues] = useState<Record<string, unknown>>({})
  const [saving, setSaving] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)

  const { data: schema } = useQuery({
    queryKey: ['schema', entity],
    queryFn: () => getEntitySchema(entity),
  })

  const { data: record, isLoading } = useQuery({
    queryKey: ['record', entity, id],
    queryFn: () => getRecord(entity, id),
    enabled: !!entity && !!id,
  })

  const { data: transitions } = useQuery({
    queryKey: ['transitions', entity, id],
    queryFn: () => getTransitions(entity, id),
    enabled: !!entity && !!id && !!schema?.workflow,
  })

  const { data: history } = useQuery({
    queryKey: ['history', entity, id],
    queryFn: () => getRecordHistory(entity, id),
    enabled: !!entity && !!id,
  })

  const startEdit = () => {
    if (record) setEditValues(record)
    setEditing(true)
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault()
    setSaving(true)
    try {
      await updateRecord(entity, id, editValues)
      setEditing(false)
      qc.invalidateQueries({ queryKey: ['record', entity, id] })
    } catch (err) {
      alert(`Update failed: ${err}`)
    } finally {
      setSaving(false)
    }
  }

  const handleDelete = async () => {
    try {
      await deleteRecord(entity, id)
      navigate(`/${entity}`)
    } catch (err) {
      alert(`Delete failed: ${err}`)
    }
  }

  const handleAction = async (action: string, toState: string) => {
    try {
      await executeAction(entity, id, action)
      qc.invalidateQueries({ queryKey: ['record', entity, id] })
      qc.invalidateQueries({ queryKey: ['transitions', entity, id] })
      qc.invalidateQueries({ queryKey: ['history', entity, id] })
    } catch (err) {
      alert(`Action failed: ${err}`)
    }
  }

  if (isLoading) {
    return (
      <div className="space-y-4">
        <div className="h-8 w-48 animate-pulse rounded bg-muted" />
        <div className="h-64 animate-pulse rounded bg-muted" />
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={() => navigate(`/${entity}`)}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <h1 className="text-3xl font-bold tracking-tight capitalize">
              {(record?.[schema?.fields?.[0]?.name ?? 'name'] as string) ?? `Record ${id.slice(0, 8)}…`}
            </h1>
            <p className="text-sm text-muted-foreground">{schema?.label ?? entity}</p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" onClick={startEdit}>
            <Edit className="mr-2 h-4 w-4" /> Edit
          </Button>
          <Button variant="destructive" size="sm" onClick={() => setDeleteOpen(true)}>
            <Trash2 className="mr-2 h-4 w-4" /> Delete
          </Button>
        </div>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Record details */}
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle>Details</CardTitle>
            </CardHeader>
            <CardContent>
              <dl className="grid gap-4 sm:grid-cols-2">
                {schema?.fields
                  .filter((f) => !['id', 'organization_id'].includes(f.name))
                  .map((field) => {
                    const val = record?.[field.name]
                    return (
                      <div key={field.name}>
                        <dt className="text-sm font-medium text-muted-foreground">{field.label}</dt>
                        <dd className="mt-1 text-sm">
                          {val === null || val === undefined ? (
                            <span className="text-muted-foreground">—</span>
                          ) : field.fieldType.type === 'date' ? (
                            formatDate(val as string)
                          ) : field.fieldType.type === 'enum' ? (
                            <Badge variant="secondary">{String(val)}</Badge>
                          ) : field.fieldType.type === 'boolean' ? (
                            val ? (
                              'Yes'
                            ) : (
                              'No'
                            )
                          ) : (
                            String(val)
                          )}
                        </dd>
                      </div>
                    )
                  })}
              </dl>
            </CardContent>
          </Card>
        </div>

        {/* Sidebar: workflow + metadata */}
        <div className="space-y-4">
          {/* Workflow status */}
          {schema?.workflow && transitions && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Workflow Status</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div>
                  <span className="text-sm text-muted-foreground">Current state</span>
                  <Badge variant="info" className="ml-2">
                    {(transitions.current_state ?? 'unknown').replace(/_/g, ' ')}
                  </Badge>
                </div>
                {transitions.transitions.length > 0 && (
                  <div className="space-y-2">
                    <p className="text-sm font-medium">Available Actions</p>
                    {transitions.transitions.map((t) => (
                      <Button
                        key={t.action}
                        variant="outline"
                        size="sm"
                        className="w-full justify-start gap-2"
                        onClick={() => handleAction(t.action, t.to_state)}
                      >
                        <Play className="h-3 w-3" />
                        {t.action_label ?? t.action}
                      </Button>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          )}

          {/* Metadata */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Metadata</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">ID</span>
                <span className="font-mono text-xs">{id.slice(0, 8)}…</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created</span>
                <span>{formatDateTime(record?.created_at as string)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Updated</span>
                <span>{formatDateTime(record?.updated_at as string)}</span>
              </div>
            </CardContent>
          </Card>

          {/* History */}
          {history?.history?.length ? (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">History</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                {history.history.slice(0, 10).map((entry, i) => (
                  <div key={i} className="flex items-start gap-2 text-sm">
                    <div className="mt-1 h-2 w-2 rounded-full bg-primary shrink-0" />
                    <div>
                      <p className="font-medium">{entry.action}</p>
                      <p className="text-xs text-muted-foreground">{formatDateTime(entry.changedAt)}</p>
                    </div>
                  </div>
                ))}
              </CardContent>
            </Card>
          ) : null}
        </div>
      </div>

      {/* Edit dialog */}
      <Dialog open={editing} onOpenChange={setEditing}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Edit {schema?.label ?? 'Record'}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSave} className="space-y-4 max-h-[60vh] overflow-y-auto pr-2">
            {schema?.fields
              .filter((f) => !f.isReadOnly && !['id', 'created_at', 'updated_at', 'deleted_at', 'organization_id'].includes(f.name))
              .map((field) => (
                <div key={field.name} className="space-y-1">
                  <label className="text-sm font-medium">
                    {field.label}
                    {field.isRequired && <span className="text-destructive ml-1">*</span>}
                  </label>
                  {field.fieldType.type === 'enum' ? (
                    <select
                      className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                      value={String(editValues[field.name] ?? '')}
                      onChange={(e) => setEditValues({ ...editValues, [field.name]: e.target.value })}
                    >
                      {field.fieldType.values.map((v) => (
                        <option key={v} value={v}>{v}</option>
                      ))}
                    </select>
                  ) : field.fieldType.type === 'rich_text' ? (
                    <textarea
                      className="flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                      value={String(editValues[field.name] ?? '')}
                      onChange={(e) => setEditValues({ ...editValues, [field.name]: e.target.value })}
                    />
                  ) : (
                    <Input
                      type={field.fieldType.type === 'date' ? 'date' : field.fieldType.type === 'integer' || field.fieldType.type === 'decimal' ? 'number' : 'text'}
                      value={String(editValues[field.name] ?? '')}
                      onChange={(e) => setEditValues({ ...editValues, [field.name]: e.target.value })}
                    />
                  )}
                </div>
              ))}
            <div className="flex justify-end gap-2 pt-4">
              <Button type="button" variant="outline" onClick={() => setEditing(false)}>Cancel</Button>
              <Button type="submit" disabled={saving}>{saving ? 'Saving…' : 'Save'}</Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>

      {/* Delete dialog */}
      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete {schema?.label ?? 'Record'}?</DialogTitle>
          </DialogHeader>
          <p className="text-sm text-muted-foreground">
            This action cannot be undone. The record will be soft-deleted.
          </p>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>Cancel</Button>
            <Button variant="destructive" onClick={handleDelete}>Delete</Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}
