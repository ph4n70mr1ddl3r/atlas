import { useState, useEffect, useCallback } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  listComments,
  createComment,
  deleteComment,
  togglePinComment,
  type Comment,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import {
  MessageSquare,
  Send,
  Trash2,
  Pin,
  PinOff,
  Reply,
  ChevronDown,
  ChevronUp,
  Eye,
  EyeOff,
} from 'lucide-react'

interface CommentsProps {
  entity: string
  recordId: string
}

export function Comments({ entity, recordId }: CommentsProps) {
  const queryClient = useQueryClient()
  const [newComment, setNewComment] = useState('')
  const [replyTo, setReplyTo] = useState<string | null>(null)
  const [replyText, setReplyText] = useState('')
  const [isInternal, setIsInternal] = useState(false)
  const [expandedThreads, setExpandedThreads] = useState<Set<string>>(new Set())

  const { data, isLoading } = useQuery({
    queryKey: ['comments', entity, recordId],
    queryFn: () => listComments(entity, recordId, { limit: 100 }),
  })

  const createMutation = useMutation({
    mutationFn: (body: string) =>
      createComment(entity, recordId, {
        body,
        parent_id: replyTo ?? undefined,
        is_internal: isInternal,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['comments', entity, recordId] })
      setNewComment('')
      setReplyText('')
      setReplyTo(null)
      setIsInternal(false)
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (commentId: string) => deleteComment(entity, recordId, commentId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['comments', entity, recordId] })
    },
  })

  const pinMutation = useMutation({
    mutationFn: (commentId: string) => togglePinComment(entity, recordId, commentId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['comments', entity, recordId] })
    },
  })

  const toggleThread = useCallback((id: string) => {
    setExpandedThreads((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }, [])

  const comments = data?.data ?? []

  // Group into threads
  const rootComments = comments.filter((c) => c.depth === 0)
  const getReplies = (parentId: string) => comments.filter((c) => c.parent_id === parentId)

  const formatTime = (dateStr: string) => {
    const date = new Date(dateStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    if (diff < 60000) return 'just now'
    if (diff < 3600000) return `${Math.floor(diff / 60000)}m ago`
    if (diff < 86400000) return `${Math.floor(diff / 3600000)}h ago`
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  }

  const renderComment = (comment: Comment, depth = 0) => {
    const replies = getReplies(comment.id)
    const isExpanded = expandedThreads.has(comment.id)

    return (
      <div key={comment.id} className={depth > 0 ? 'ml-8 border-l-2 border-muted pl-3' : ''}>
        <div className={`flex gap-3 py-3 ${comment.is_pinned ? 'bg-yellow-50/50 -mx-2 px-2 rounded' : ''}`}>
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-muted text-xs font-medium">
            {(comment.user_name ?? 'U').charAt(0).toUpperCase()}
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="text-sm font-medium">{comment.user_name ?? 'Unknown'}</span>
              <span className="text-xs text-muted-foreground">{formatTime(comment.created_at)}</span>
              {comment.is_pinned && (
                <Badge variant="outline" className="text-[10px] h-4 px-1.5 gap-0.5">
                  <Pin className="h-2.5 w-2.5" /> Pinned
                </Badge>
              )}
              {comment.is_internal && (
                <Badge variant="secondary" className="text-[10px] h-4 px-1.5">
                  Internal
                </Badge>
              )}
            </div>
            <p className="text-sm mt-1 whitespace-pre-wrap">{comment.body}</p>
            <div className="flex items-center gap-1 mt-2">
              <Button
                variant="ghost"
                size="sm"
                className="h-6 text-xs gap-1"
                onClick={() => {
                  setReplyTo(comment.id)
                  setReplyText(`@${comment.user_name ?? 'User'} `)
                }}
              >
                <Reply className="h-3 w-3" /> Reply
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-6 text-xs gap-1"
                onClick={() => pinMutation.mutate(comment.id)}
              >
                {comment.is_pinned ? (
                  <><PinOff className="h-3 w-3" /> Unpin</>
                ) : (
                  <><Pin className="h-3 w-3" /> Pin</>
                )}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                className="h-6 text-xs gap-1 text-destructive hover:text-destructive"
                onClick={() => deleteMutation.mutate(comment.id)}
              >
                <Trash2 className="h-3 w-3" /> Delete
              </Button>
            </div>

            {/* Reply input for this comment */}
            {replyTo === comment.id && (
              <div className="mt-2 flex gap-2">
                <Textarea
                  value={replyText}
                  onChange={(e) => setReplyText(e.target.value)}
                  placeholder="Write a reply..."
                  className="min-h-[60px] text-sm"
                />
                <div className="flex flex-col gap-1">
                  <Button
                    size="sm"
                    onClick={() => {
                      if (replyText.trim()) createMutation.mutate(replyText)
                    }}
                    disabled={!replyText.trim() || createMutation.isPending}
                  >
                    <Send className="h-3 w-3" />
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => { setReplyTo(null); setReplyText('') }}
                  >
                    <ChevronUp className="h-3 w-3" />
                  </Button>
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Replies */}
        {replies.length > 0 && (depth === 0 || isExpanded) && (
          <div>
            {replies.map((r) => renderComment(r, depth + 1))}
          </div>
        )}
        {depth === 0 && replies.length > 0 && !isExpanded && (
          <Button
            variant="ghost"
            size="sm"
            className="h-6 text-xs text-muted-foreground ml-8"
            onClick={() => toggleThread(comment.id)}
          >
            <ChevronDown className="h-3 w-3 mr-1" />
            {replies.length} repl{replies.length === 1 ? 'y' : 'ies'}
          </Button>
        )}
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <MessageSquare className="h-5 w-5" />
        <h3 className="font-semibold">Comments ({data?.meta.total ?? 0})</h3>
      </div>

      {/* New comment input */}
      <div className="space-y-2">
        <Textarea
          value={newComment}
          onChange={(e) => setNewComment(e.target.value)}
          placeholder="Add a comment..."
          className="min-h-[80px]"
        />
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            onClick={() => {
              if (newComment.trim()) createMutation.mutate(newComment)
            }}
            disabled={!newComment.trim() || createMutation.isPending}
          >
            <Send className="h-4 w-4 mr-1" /> Comment
          </Button>
          <Button
            variant={isInternal ? 'secondary' : 'ghost'}
            size="sm"
            onClick={() => setIsInternal(!isInternal)}
            className="gap-1"
          >
            {isInternal ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
            Internal
          </Button>
        </div>
      </div>

      {/* Comments list */}
      {isLoading ? (
        <div className="text-sm text-muted-foreground py-4 text-center">Loading comments...</div>
      ) : rootComments.length === 0 ? (
        <div className="text-sm text-muted-foreground py-4 text-center">
          No comments yet. Be the first to comment!
        </div>
      ) : (
        <div className="divide-y">
          {rootComments.map((c) => renderComment(c))}
        </div>
      )}
    </div>
  )
}
