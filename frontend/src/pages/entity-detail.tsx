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
  addFavorite,
  removeFavorite,
  checkFavorite,
  getRelatedRecords,
} from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Input } from '@/components/ui/input'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { DynamicForm } from '@/components/dynamic-form'
import { WorkflowDiagram } from '@/components/workflow-diagram'
import { Comments } from '@/components/comments'
import {
  ArrowLeft, Edit, Trash2, GitBranch, History,
  Star, StarOff, ChevronDown, ChevronRight, Link2,
} from 'lucide-react'
import { formatDateTime, formatDate } from '@/lib/utils'

export function EntityDetailPage() {
  const { entity = '', id = '' } = useParams<{ entity: string; id: string }>()
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [editing, setEditing] = useState(false)
  const [deleteOpen, setDeleteOpen] = useState(false)
  const [showWorkflow, setShowWorkflow] = useState(false)
  const [showComments, setShowComments] = useState(true)
  const [expandedRelated, setExpandedRelated] = useState<Set<string>>(new Set())

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

  // Favorite status
  const { data: favStatus } = useQuery({
    queryKey: ['favorite', entity, id],
    queryFn: () => checkFavorite(entity, id),
    enabled: !!entity && !!id,
  })

  const toggleFavorite = async () => {
    try {
      if (favStatus?.is_favorite) {
        await removeFavorite(entity, id)
      } else {
        await addFavorite(entity, id)
      }
      qc.invalidateQueries({ queryKey: ['favorite', entity, id] })
    } catch (err) {
      console.error('Favorite toggle failed:', err)
    }
  }

  // Discover related entities from schema
  const relatedEntities = schema?.fields.filter(
    (f) => f.fieldType.type === 'one_to_many',
  ) ?? []

  const handleAction = async (action: string) => {
    try {
      await executeAction(entity, id, action)
      qc.invalidateQueries({ queryKey: ['record', entity, id] })
      qc.invalidateQueries({ queryKey: ['transitions', entity, id] })
      qc.invalidateQueries({ queryKey: ['history', entity, id] })
    } catch (err) {
      alert(`Action failed: ${err}`)
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

  const invalidateAll = () => {
    qc.invalidateQueries({ queryKey: ['record', entity, id] })
    qc.invalidateQueries({ queryKey: ['transitions', entity, id] })
    qc.invalidateQueries({ queryKey: ['history', entity, id] })
  }

  if (isLoading) {
    return (
      <div className="space-y-4">
        <div className="h-8 w-48 animate-pulse rounded bg-muted" />
        <div className="h-64 animate-pulse rounded bg-muted" />
      </div>
    )
  }

  const availableActions = transitions?.transitions.map((t) => t.action) ?? []
  const recordTitle = record
    ? (record[schema?.fields?.find((f) => f.isSearchable)?.name ?? 'name'] as string) ?? `Record ${id.slice(0, 8)}…`
    : 'Loading…'

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
              {String(recordTitle)}
            </h1>
            <p className="text-sm text-muted-foreground">{schema?.label ?? entity}</p>
          </div>
          {transitions?.current_state && (
            <Badge variant="info" className="ml-2 text-sm">
              {transitions.current_state.replace(/_/g, ' ')}
            </Badge>
          )}
        </div>
        <div className="flex gap-2">
          {schema?.workflow && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowWorkflow(!showWorkflow)}
            >
              <GitBranch className="mr-2 h-4 w-4" />
              {showWorkflow ? 'Hide Workflow' : 'Show Workflow'}
            </Button>
          )}
          <Button variant="outline" size="sm" onClick={() => setEditing(true)}>
            <Edit className="mr-2 h-4 w-4" /> Edit
          </Button>
          <Button variant="destructive" size="sm" onClick={() => setDeleteOpen(true)}>
            <Trash2 className="mr-2 h-4 w-4" /> Delete
          </Button>
          <Button
            variant={favStatus?.is_favorite ? 'default' : 'outline'}
            size="sm"
            onClick={toggleFavorite}
            title={favStatus?.is_favorite ? 'Remove from favorites' : 'Add to favorites'}
          >
            {favStatus?.is_favorite ? (
              <><Star className="mr-2 h-4 w-4 fill-current" /> Favorited</>
            ) : (
              <><StarOff className="mr-2 h-4 w-4" /> Favorite</>
            )}
          </Button>
        </div>
      </div>

      {/* Workflow visualization */}
      {showWorkflow && schema?.workflow && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <GitBranch className="h-5 w-5" />
              Workflow: {schema.workflow.name.replace(/_/g, ' ')}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <WorkflowDiagram
              workflow={schema.workflow}
              currentState={transitions?.current_state}
              availableActions={availableActions}
              onTransitionClick={(t) => handleAction(t.action)}
            />
          </CardContent>
        </Card>
      )}

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Record details */}
        <div className="lg:col-span-2 space-y-6">
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
                      <div key={field.name} className="space-y-1">
                        <dt className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
                          {field.label}
                        </dt>
                        <dd className="text-sm">
                          {val === null || val === undefined ? (
                            <span className="text-muted-foreground">—</span>
                          ) : field.fieldType.type === 'date' ? (
                            formatDate(val as string)
                          ) : field.fieldType.type === 'date_time' ? (
                            formatDateTime(val as string)
                          ) : field.fieldType.type === 'enum' ? (
                            <Badge variant="secondary">{String(val).replace(/_/g, ' ')}</Badge>
                          ) : field.fieldType.type === 'boolean' ? (
                            val ? (
                              <Badge variant="success">Yes</Badge>
                            ) : (
                              <Badge variant="secondary">No</Badge>
                            )
                          ) : field.fieldType.type === 'currency' || field.fieldType.type === 'decimal' ? (
                            <span className="font-mono">{Number(val).toLocaleString()}</span>
                          ) : field.fieldType.type === 'rich_text' ? (
                            <div className="whitespace-pre-wrap text-sm max-h-[200px] overflow-y-auto">{String(val)}</div>
                          ) : (
                            <span>{String(val)}</span>
                          )}
                        </dd>
                      </div>
                    )
                  })}
              </dl>
            </CardContent>
          </Card>

          {/* Inline workflow actions */}
          {schema?.workflow && transitions && transitions.transitions.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Available Actions</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-2">
                  {transitions.transitions.map((t) => (
                    <Button
                      key={t.action}
                      variant="outline"
                      size="sm"
                      onClick={() => handleAction(t.action)}
                      className="gap-1.5"
                    >
                      <span className="text-xs">→</span>
                      <span className="capitalize">{t.action_label ?? t.action.replace(/_/g, ' ')}</span>
                      <Badge variant="secondary" className="text-[9px] ml-1">
                        {t.to_state.replace(/_/g, ' ')}
                      </Badge>
                    </Button>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </div>

        {/* Sidebar: metadata + history */}
        <div className="space-y-4">
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
                <span className="text-muted-foreground">Entity</span>
                <span className="capitalize">{entity.replace(/_/g, ' ')}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Created</span>
                <span>{formatDateTime(record?.created_at as string)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">Updated</span>
                <span>{formatDateTime(record?.updated_at as string)}</span>
              </div>
              {record?.workflow_state != null && (
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Status</span>
                  <Badge variant="info">{String(record.workflow_state).replace(/_/g, ' ')}</Badge>
                </div>
              )}
            </CardContent>
          </Card>

          {/* History */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base flex items-center gap-2">
                <History className="h-4 w-4" /> History
              </CardTitle>
            </CardHeader>
            <CardContent>
              {history?.history?.length ? (
                <div className="space-y-3 max-h-[400px] overflow-y-auto">
                  {history.history.map((entry, i) => (
                    <div key={i} className="flex items-start gap-3 text-sm">
                      <div className="mt-1.5 h-2 w-2 shrink-0 rounded-full bg-primary" />
                      <div className="min-w-0">
                        <p className="font-medium capitalize">{entry.action.replace(/_/g, ' ')}</p>
                        <p className="text-xs text-muted-foreground">
                          {formatDateTime(entry.changedAt)}
                          {entry.changedBy ? ` · by ${entry.changedBy}` : ''}
                        </p>
                        {entry.action === 'update' && entry.newData != null && (
                          <div className="mt-1 text-xs text-muted-foreground">
                            {Object.keys(entry.newData as Record<string, unknown>).slice(0, 3).map((k) => (
                              <span key={k} className="mr-2">
                                <span className="font-medium">{k}:</span>{' '}
                                {String((entry.newData as Record<string, unknown>)[k]).slice(0, 20)}
                              </span>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-muted-foreground py-4 text-center">No history available</p>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      {/* Related Records Sections */}
      {relatedEntities.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Link2 className="h-5 w-5" />
              Related Records
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {relatedEntities.map((field) => {
              const relEntity = (field.fieldType as { type: string; entity: string }).entity
              const isOpen = expandedRelated.has(relEntity)
              return (
                <RelatedSection
                  key={relEntity}
                  entity={entity}
                  recordId={id}
                  relatedEntity={relEntity}
                  label={field.label}
                  isOpen={isOpen}
                  onToggle={() => {
                    setExpandedRelated((prev) => {
                      const next = new Set(prev)
                      if (next.has(relEntity)) next.delete(relEntity)
                      else next.add(relEntity)
                      return next
                    })
                  }}
                />
              )
            })}
          </CardContent>
        </Card>
      )}

      {/* Comments Section */}
      <Card>
        <CardHeader>
          <CardTitle
            className="flex items-center gap-2 cursor-pointer"
            onClick={() => setShowComments(!showComments)}
          >
            {showComments ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
            Comments
          </CardTitle>
        </CardHeader>
        {showComments && (
          <CardContent>
            <Comments entity={entity} recordId={id} />
          </CardContent>
        )}
      </Card>

      {/* Edit dialog with dynamic form */}
      <Dialog open={editing} onOpenChange={setEditing}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>Edit {schema?.label ?? 'Record'}</DialogTitle>
          </DialogHeader>
          {schema && record && (
            <DynamicForm
              schema={schema}
              initialData={record}
              onSubmit={async (values) => {
                await updateRecord(entity, id, values)
                setEditing(false)
                invalidateAll()
              }}
              onCancel={() => setEditing(false)}
              submitLabel="Save Changes"
            />
          )}
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
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>Cancel</Button>
            <Button variant="destructive" onClick={handleDelete}>Delete</Button>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  )
}

/** Related records sub-section */
function RelatedSection({
  entity,
  recordId,
  relatedEntity,
  label,
  isOpen,
  onToggle,
}: {
  entity: string
  recordId: string
  relatedEntity: string
  label: string
  isOpen: boolean
  onToggle: () => void
}) {
  const { data, isLoading } = useQuery({
    queryKey: ['related', entity, recordId, relatedEntity],
    queryFn: () => getRelatedRecords(entity, recordId, relatedEntity, { limit: 5 }),
    enabled: isOpen,
  })

  return (
    <div>
      <button
        onClick={onToggle}
        className="flex items-center gap-2 text-sm font-medium hover:text-primary transition-colors w-full text-left"
      >
        {isOpen ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
        {label ?? relatedEntity.replace(/_/g, ' ')}
        {data && (
          <Badge variant="secondary" className="text-[10px] ml-1">{data.meta.total}</Badge>
        )}
      </button>
      {isOpen && (
        <div className="mt-2 ml-6">
          {isLoading ? (
            <div className="text-sm text-muted-foreground">Loading...</div>
          ) : data?.data.length === 0 ? (
            <div className="text-sm text-muted-foreground">No records found</div>
          ) : (
            <div className="space-y-1">
              {data?.data.slice(0, 5).map((rec, idx) => (
                <div
                  key={idx}
                  className="flex items-center gap-2 rounded px-2 py-1 text-sm hover:bg-muted cursor-pointer"
                  onClick={() => {
                    window.location.hash = `/${relatedEntity}/${rec.id}`
                  }}
                >
                  <span className="font-mono text-xs text-muted-foreground">
                    {(rec.id as string).slice(0, 8)}…
                  </span>
                  {Object.entries(rec)
                    .filter(([k]) => !['id', 'organization_id', 'created_at', 'updated_at', 'deleted_at'].includes(k))
                    .slice(0, 3)
                    .map(([k, v]) => (
                      <span key={k} className="text-xs">
                        <span className="text-muted-foreground">{k}:</span> {String(v).slice(0, 20)}
                      </span>
                    ))}
                </div>
              ))}
              {data && data.meta.total > 5 && (
                <div className="text-xs text-muted-foreground">
                  +{data.meta.total - 5} more
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
