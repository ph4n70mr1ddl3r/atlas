import { useState } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import {
  listNotifications,
  getUnreadNotificationCount,
  markNotificationRead,
  markAllNotificationsRead,
  dismissNotification,
  type Notification,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Bell,
  Check,
  CheckCheck,
  X,
  ArrowRight,
  AlertTriangle,
  Info,
  Clock,
  UserPlus,
  GitBranch,
} from 'lucide-react'

function getNotificationIcon(type: string) {
  switch (type) {
    case 'approval_required': return <UserPlus className="h-4 w-4 text-orange-600 shrink-0" />
    case 'escalation': return <AlertTriangle className="h-4 w-4 text-red-600 shrink-0" />
    case 'workflow_action': return <GitBranch className="h-4 w-4 text-blue-600 shrink-0" />
    case 'duplicate_detected': return <AlertTriangle className="h-4 w-4 text-yellow-600 shrink-0" />
    default: return <Info className="h-4 w-4 text-gray-600 shrink-0" />
  }
}

function getPriorityBorder(priority: string) {
  switch (priority) {
    case 'urgent': return 'border-l-red-500'
    case 'high': return 'border-l-orange-500'
    case 'low': return 'border-l-gray-300'
    default: return 'border-l-blue-500'
  }
}

function formatTimeAgo(dateStr: string): string {
  const date = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`
  return date.toLocaleDateString()
}

export function NotificationBell() {
  const [open, setOpen] = useState(false)
  const qc = useQueryClient()

  const { data: countData } = useQuery({
    queryKey: ['notification-unread-count'],
    queryFn: getUnreadNotificationCount,
    refetchInterval: 30000, // Poll every 30 seconds
  })

  const { data: notifData, isLoading } = useQuery({
    queryKey: ['notifications', { include_read: false }],
    queryFn: () => listNotifications({ include_read: false, limit: 20, offset: 0 }),
    enabled: open,
  })

  const unreadCount = countData?.count ?? 0
  const notifications = notifData?.data ?? []

  const handleMarkRead = async (id: string) => {
    await markNotificationRead(id)
    qc.invalidateQueries({ queryKey: ['notification-unread-count'] })
    qc.invalidateQueries({ queryKey: ['notifications'] })
  }

  const handleMarkAllRead = async () => {
    await markAllNotificationsRead()
    qc.invalidateQueries({ queryKey: ['notification-unread-count'] })
    qc.invalidateQueries({ queryKey: ['notifications'] })
  }

  const handleDismiss = async (id: string) => {
    await dismissNotification(id)
    qc.invalidateQueries({ queryKey: ['notification-unread-count'] })
    qc.invalidateQueries({ queryKey: ['notifications'] })
  }

  return (
    <div className="relative">
      <Button
        variant="ghost"
        size="icon"
        className="relative"
        onClick={() => setOpen(!open)}
      >
        <Bell className="h-5 w-5" />
        {unreadCount > 0 && (
          <span className="absolute -top-1 -right-1 flex h-5 min-w-5 items-center justify-center rounded-full bg-red-600 text-[10px] font-bold text-white px-1">
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        )}
      </Button>

      {open && (
        <>
          {/* Backdrop */}
          <div className="fixed inset-0 z-40" onClick={() => setOpen(false)} />

          {/* Notification panel */}
          <div className="absolute right-0 top-12 z-50 w-96 max-h-[500px] overflow-hidden rounded-xl border bg-white shadow-xl">
            <div className="flex items-center justify-between border-b px-4 py-3">
              <div className="flex items-center gap-2">
                <h3 className="text-sm font-semibold">Notifications</h3>
                {unreadCount > 0 && (
                  <Badge variant="secondary" className="text-[10px]">
                    {unreadCount} new
                  </Badge>
                )}
              </div>
              <div className="flex items-center gap-1">
                {unreadCount > 0 && (
                  <Button variant="ghost" size="sm" className="h-7 text-xs" onClick={handleMarkAllRead}>
                    <CheckCheck className="mr-1 h-3 w-3" /> Mark all read
                  </Button>
                )}
                <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => setOpen(false)}>
                  <X className="h-4 w-4" />
                </Button>
              </div>
            </div>

            <div className="overflow-y-auto max-h-[420px]">
              {isLoading ? (
                <div className="flex items-center justify-center py-8">
                  <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                </div>
              ) : notifications.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
                  <Bell className="h-8 w-8 mb-2 opacity-40" />
                  <p className="text-sm">No new notifications</p>
                </div>
              ) : (
                notifications.map((n: Notification) => (
                  <div
                    key={n.id}
                    className={`flex items-start gap-3 border-b px-4 py-3 hover:bg-muted/50 cursor-pointer transition-colors border-l-4 ${getPriorityBorder(n.priority)}`}
                    onClick={() => {
                      if (!n.is_read) handleMarkRead(n.id)
                      if (n.action_url) window.location.href = n.action_url
                    }}
                  >
                    <div className="mt-0.5">{getNotificationIcon(n.notification_type)}</div>
                    <div className="flex-1 min-w-0">
                      <p className={`text-sm ${n.is_read ? 'text-muted-foreground' : 'font-medium'}`}>
                        {n.title}
                      </p>
                      {n.message && (
                        <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">
                          {n.message}
                        </p>
                      )}
                      <div className="flex items-center gap-2 mt-1">
                        <Clock className="h-3 w-3 text-muted-foreground" />
                        <span className="text-[10px] text-muted-foreground">
                          {formatTimeAgo(n.created_at)}
                        </span>
                        {n.notification_type === 'approval_required' && (
                          <Badge variant="outline" className="text-[9px] h-4">Needs Approval</Badge>
                        )}
                      </div>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6 shrink-0"
                      onClick={(e) => {
                        e.stopPropagation()
                        handleDismiss(n.id)
                      }}
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </div>
                ))
              )}
            </div>
          </div>
        </>
      )}
    </div>
  )
}