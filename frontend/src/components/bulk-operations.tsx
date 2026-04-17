import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { executeBulkOperation, type FilterExpression } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import {
  CheckSquare,
  Trash2,
  Edit3,
  Play,
  Eye,
  AlertTriangle,
  Loader2,
} from 'lucide-react'

interface BulkOperationsProps {
  entity: string
  selectedIds: string[]
  filter?: FilterExpression
  onCompleted: () => void
}

export function BulkOperations({ entity, selectedIds, filter, onCompleted }: BulkOperationsProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [operation, setOperation] = useState<'update' | 'delete' | 'workflow_action'>('update')
  const [updateField, setUpdateField] = useState('')
  const [updateValue, setUpdateValue] = useState('')
  const [workflowAction, setWorkflowAction] = useState('')
  const [dryRunResult, setDryRunResult] = useState<{
    total_records: number
    operation: string
  } | null>(null)
  const [isDryRun, setIsDryRun] = useState(false)

  const mutation = useMutation({
    mutationFn: (dryRun: boolean) => {
      const payload = buildPayload()
      return executeBulkOperation({
        entity_type: entity,
        operation,
        record_ids: selectedIds.length > 0 ? selectedIds : undefined,
        filter: selectedIds.length === 0 ? filter : undefined,
        payload,
        dry_run: dryRun,
      })
    },
    onSuccess: (data) => {
      if (isDryRun) {
        setDryRunResult(data)
      } else {
        setIsOpen(false)
        setDryRunResult(null)
        onCompleted()
      }
    },
  })

  const buildPayload = () => {
    switch (operation) {
      case 'update':
        return { values: { [updateField]: updateValue } }
      case 'delete':
        return {}
      case 'workflow_action':
        return { action: workflowAction }
    }
  }

  const count = selectedIds.length

  if (count === 0 && !filter) return null

  return (
    <Dialog open={isOpen} onOpenChange={(open) => { setIsOpen(open); if (!open) setDryRunResult(null) }}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm" className="gap-1" disabled={count === 0 && !filter}>
          <CheckSquare className="h-4 w-4" />
          Bulk Action{count > 0 ? ` (${count})` : ''}
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>Bulk Operation</DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          {/* Target info */}
          <div className="rounded-lg bg-muted p-3 text-sm">
            <span className="font-medium">{count > 0 ? `${count} records selected` : 'All filtered records'}</span>
            <span className="text-muted-foreground"> on </span>
            <span className="font-mono text-xs">{entity}</span>
          </div>

          {/* Operation type */}
          <div className="flex gap-2">
            {[
              { op: 'update' as const, icon: Edit3, label: 'Update' },
              { op: 'delete' as const, icon: Trash2, label: 'Delete' },
              { op: 'workflow_action' as const, icon: Play, label: 'Workflow' },
            ].map(({ op, icon: Icon, label }) => (
              <Button
                key={op}
                variant={operation === op ? 'default' : 'outline'}
                size="sm"
                onClick={() => { setOperation(op); setDryRunResult(null) }}
                className="gap-1"
              >
                <Icon className="h-4 w-4" /> {label}
              </Button>
            ))}
          </div>

          {/* Operation config */}
          {operation === 'update' && (
            <div className="space-y-2">
              <Input
                value={updateField}
                onChange={(e) => setUpdateField(e.target.value)}
                placeholder="Field name to update"
              />
              <Input
                value={updateValue}
                onChange={(e) => setUpdateValue(e.target.value)}
                placeholder="New value"
              />
            </div>
          )}

          {operation === 'workflow_action' && (
            <Input
              value={workflowAction}
              onChange={(e) => setWorkflowAction(e.target.value)}
              placeholder="Workflow action (e.g., approve, reject)"
            />
          )}

          {operation === 'delete' && (
            <div className="flex items-center gap-2 rounded-lg bg-destructive/10 p-3 text-sm text-destructive">
              <AlertTriangle className="h-4 w-4 shrink-0" />
              This will delete {count > 0 ? `${count}` : 'all matching'} records.
              This action cannot be undone.
            </div>
          )}

          {/* Dry run result */}
          {dryRunResult && (
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm">Preview</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-sm">
                  <Badge variant="outline" className="mr-2">
                    {dryRunResult.total_records} records
                  </Badge>
                  will be affected by this {dryRunResult.operation} operation.
                </div>
              </CardContent>
            </Card>
          )}

          {/* Actions */}
          <div className="flex items-center gap-2 justify-end">
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                setIsDryRun(true)
                mutation.mutate(true)
              }}
              disabled={mutation.isPending}
            >
              {mutation.isPending && isDryRun ? (
                <Loader2 className="h-4 w-4 mr-1 animate-spin" />
              ) : (
                <Eye className="h-4 w-4 mr-1" />
              )}
              Preview
            </Button>
            <Button
              size="sm"
              variant={operation === 'delete' ? 'destructive' : 'default'}
              onClick={() => {
                setIsDryRun(false)
                mutation.mutate(false)
              }}
              disabled={mutation.isPending || (operation === 'update' && (!updateField || !updateValue))}
            >
              {mutation.isPending && !isDryRun ? (
                <Loader2 className="h-4 w-4 mr-1 animate-spin" />
              ) : null}
              Execute
            </Button>
          </div>

          {/* Result */}
          {mutation.data && !isDryRun && (
            <div className="rounded-lg bg-muted p-3 text-sm space-y-1">
              <div className="flex justify-between">
                <span>Succeeded:</span>
                <Badge variant="default">{mutation.data.succeeded}</Badge>
              </div>
              {mutation.data.failed > 0 && (
                <div className="flex justify-between">
                  <span>Failed:</span>
                  <Badge variant="destructive">{mutation.data.failed}</Badge>
                </div>
              )}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
