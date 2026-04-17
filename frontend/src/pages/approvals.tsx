import { useQuery, useQueryClient } from '@tanstack/react-query'
import { getPendingApprovals, approveApprovalStep, rejectApprovalStep, type ApprovalStep } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { CheckCircle, XCircle, Clock, ArrowLeft, GitBranch } from 'lucide-react'
import { Link } from 'react-router-dom'
import { formatDateTime } from '@/lib/utils'

export function ApprovalsPage() {
  const qc = useQueryClient()

  const { data, isLoading } = useQuery({
    queryKey: ['pending-approvals'],
    queryFn: getPendingApprovals,
    refetchInterval: 15000,
  })

  const steps = data?.data ?? []

  const handleApprove = async (stepId: string) => {
    await approveApprovalStep(stepId)
    qc.invalidateQueries({ queryKey: ['pending-approvals'] })
    qc.invalidateQueries({ queryKey: ['notification-unread-count'] })
  }

  const handleReject = async (stepId: string) => {
    await rejectApprovalStep(stepId)
    qc.invalidateQueries({ queryKey: ['pending-approvals'] })
    qc.invalidateQueries({ queryKey: ['notification-unread-count'] })
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">My Approvals</h1>
          <p className="text-muted-foreground mt-1">
            Review and action items requiring your approval
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Badge variant="secondary">{steps.length} pending</Badge>
        </div>
      </div>

      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        </div>
      ) : steps.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-16 text-center">
            <CheckCircle className="h-12 w-12 text-green-500 mb-4" />
            <h3 className="text-lg font-medium">All caught up!</h3>
            <p className="text-muted-foreground mt-1">
              No items require your approval right now.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {steps.map((step: ApprovalStep) => {
            const stepId = step.id
            const level = step.level
            const approverType = step.approver_type
            const approverRole = step.approver_role
            const isDelegated = step.is_delegated
            const autoApproveHours = step.auto_approve_after_hours
            const createdAt = step.created_at
            const requestId = step.approval_request_id

            return (
              <Card key={stepId} className="border-l-4 border-l-orange-400">
                <CardHeader className="pb-2">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-base flex items-center gap-2">
                      <GitBranch className="h-4 w-4" />
                      Approval Required — Level {level}
                    </CardTitle>
                    <div className="flex items-center gap-2">
                      {isDelegated && (
                        <Badge variant="outline" className="text-xs">Delegated</Badge>
                      )}
                      {approverRole && (
                        <Badge variant="secondary" className="text-xs">
                          Role: {String(approverRole).replace(/_/g, ' ')}
                        </Badge>
                      )}
                      {autoApproveHours && (
                        <Badge variant="outline" className="text-xs">
                          <Clock className="mr-1 h-3 w-3" />
                          Auto-approve after {autoApproveHours}h
                        </Badge>
                      )}
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="flex items-center justify-between">
                    <div className="text-sm text-muted-foreground">
                      Request ID: {requestId.slice(0, 8)}…
                      {' · '}
                      Created: {formatDateTime(createdAt)}
                      {' · '}
                      Type: {approverType.replace(/_/g, ' ')}
                    </div>
                    <div className="flex gap-2">
                      <Button
                        size="sm"
                        variant="default"
                        className="gap-1"
                        onClick={() => handleApprove(stepId)}
                      >
                        <CheckCircle className="h-3.5 w-3.5" />
                        Approve
                      </Button>
                      <Button
                        size="sm"
                        variant="destructive"
                        className="gap-1"
                        onClick={() => handleReject(stepId)}
                      >
                        <XCircle className="h-3.5 w-3.5" />
                        Reject
                      </Button>
                    </div>
                  </div>
                </CardContent>
              </Card>
            )
          })}
        </div>
      )}
    </div>
  )
}