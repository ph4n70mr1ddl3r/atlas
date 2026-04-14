import type { WorkflowDefinition, TransitionDefinition, StateDefinition } from '@/lib/api'
import { cn } from '@/lib/utils'

interface WorkflowDiagramProps {
  workflow: WorkflowDefinition
  currentState?: string
  onTransitionClick?: (transition: TransitionDefinition) => void
  availableActions?: string[]
}

/**
 * Visual workflow state machine diagram.
 * Renders states as nodes connected by transition arrows.
 * Highlights the current state and available transitions.
 */
export function WorkflowDiagram({
  workflow,
  currentState,
  onTransitionClick,
  availableActions = [],
}: WorkflowDiagramProps) {
  const { states, transitions, initialState } = workflow

  // Build adjacency for layout
  const stateMap = new Map(states.map((s) => [s.name, s]))

  // Group transitions by "from" state
  const transitionsFrom = new Map<string, TransitionDefinition[]>()
  for (const t of transitions) {
    const list = transitionsFrom.get(t.fromState) ?? []
    list.push(t)
    transitionsFrom.set(t.fromState, list)
  }

  // Layout states in a flow (left-to-right layers)
  const layers = buildLayers(states, transitions, initialState)

  return (
    <div className="space-y-4">
      <div className="relative overflow-x-auto">
        <div className="inline-flex gap-8 items-start min-w-full py-4">
          {layers.map((layer, layerIdx) => (
            <div key={layerIdx} className="flex flex-col items-center gap-3">
              {layer.map((state) => {
                const stateDef = stateMap.get(state)
                const isCurrent = state === currentState
                const isInitial = state === initialState
                const isFinal = stateDef?.stateType === 'final'

                // Find incoming transitions
                const incomingTransitions = transitions.filter((t) => t.toState === state)
                const outgoingTransitions = transitionsFrom.get(state) ?? []

                return (
                  <div key={state} className="flex flex-col items-center">
                    {/* Incoming arrows */}
                    {incomingTransitions.length > 0 && (
                      <div className="flex items-center justify-center gap-2 mb-2">
                        {incomingTransitions.map((t) => (
                          <div
                            key={t.name}
                            className="flex items-center gap-1 text-xs text-muted-foreground"
                          >
                            <span className="text-[10px] uppercase tracking-wide text-muted-foreground/60">
                              {t.action.replace(/_/g, ' ')}
                            </span>
                            <span>↓</span>
                          </div>
                        ))}
                      </div>
                    )}

                    {/* State node */}
                    <div
                      className={cn(
                        'relative rounded-xl border-2 px-6 py-3 text-center min-w-[140px] transition-all',
                        isCurrent
                          ? 'border-primary bg-primary/10 shadow-md ring-2 ring-primary/30'
                          : isFinal
                            ? 'border-red-200 bg-red-50'
                            : isInitial
                              ? 'border-green-200 bg-green-50'
                              : 'border-border bg-card hover:border-primary/40',
                      )}
                    >
                      {/* State type indicator */}
                      {isInitial && (
                        <div className="absolute -top-2 left-2 rounded-full bg-green-500 px-1.5 py-0.5 text-[9px] font-bold text-white">
                          START
                        </div>
                      )}
                      {isFinal && (
                        <div className="absolute -top-2 right-2 rounded-full bg-red-500 px-1.5 py-0.5 text-[9px] font-bold text-white">
                          END
                        </div>
                      )}

                      <p className={cn(
                        'text-sm font-semibold capitalize',
                        isCurrent ? 'text-primary' : 'text-foreground',
                      )}>
                        {state.replace(/_/g, ' ')}
                      </p>
                      {stateDef?.label && stateDef.label !== state && (
                        <p className="text-[10px] text-muted-foreground">{stateDef.label}</p>
                      )}
                    </div>

                    {/* Outgoing transitions */}
                    {outgoingTransitions.length > 0 && (
                      <div className="mt-2 space-y-1">
                        {outgoingTransitions.map((t) => {
                          const isAvailable = availableActions.includes(t.action)
                          return (
                            <button
                              key={t.name}
                              onClick={() => isAvailable && onTransitionClick?.(t)}
                              disabled={!isAvailable}
                              className={cn(
                                'flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs font-medium transition-colors w-full',
                                isAvailable
                                  ? 'border-primary/30 bg-primary/5 text-primary hover:bg-primary/10 cursor-pointer'
                                  : 'border-border text-muted-foreground cursor-default',
                              )}
                            >
                              <span className="text-[10px]">→</span>
                              <span className="capitalize">{t.actionLabel ?? t.action.replace(/_/g, ' ')}</span>
                              {t.requiredRoles.length > 0 && (
                                <span className="text-[9px] text-muted-foreground/60">
                                  ({t.requiredRoles.join(', ')})
                                </span>
                              )}
                            </button>
                          )
                        })}
                      </div>
                    )}
                  </div>
                )
              })}

              {/* Arrow to next layer */}
              {layerIdx < layers.length - 1 && (
                <div className="text-muted-foreground/30 text-2xl mt-2">→</div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Legend */}
      <div className="flex flex-wrap gap-4 text-xs text-muted-foreground border-t pt-3">
        <div className="flex items-center gap-1.5">
          <div className="h-3 w-3 rounded border-2 border-green-300 bg-green-50" />
          <span>Initial</span>
        </div>
        <div className="flex items-center gap-1.5">
          <div className="h-3 w-3 rounded border-2 border-primary bg-primary/10 ring-1 ring-primary/30" />
          <span>Current</span>
        </div>
        <div className="flex items-center gap-1.5">
          <div className="h-3 w-3 rounded border-2 border-red-300 bg-red-50" />
          <span>Final</span>
        </div>
        <div className="flex items-center gap-1.5">
          <div className="h-3 w-3 rounded border border-primary/30 bg-primary/5" />
          <span>Available action</span>
        </div>
      </div>
    </div>
  )
}

/**
 * Build a simple layered layout for the workflow states.
 * Uses BFS from the initial state to determine layers.
 */
function buildLayers(
  states: StateDefinition[],
  transitions: TransitionDefinition[],
  initialState: string,
): string[][] {
  const visited = new Set<string>()
  const layers: string[][] = []
  const adj = new Map<string, string[]>()

  for (const t of transitions) {
    const list = adj.get(t.fromState) ?? []
    if (!list.includes(t.toState)) list.push(t.toState)
    adj.set(t.fromState, list)
  }

  // BFS
  let queue = [initialState]
  visited.add(initialState)

  // Also collect any states not reachable from initial
  const allStateNames = new Set(states.map((s) => s.name))

  while (queue.length > 0) {
    layers.push(queue)
    const next: string[] = []
    for (const state of queue) {
      for (const neighbor of adj.get(state) ?? []) {
        if (!visited.has(neighbor)) {
          visited.add(neighbor)
          next.push(neighbor)
        }
      }
    }
    queue = next
  }

  // Add any unreachable states as a final layer
  const unreachable = states
    .filter((s) => !visited.has(s.name))
    .map((s) => s.name)
  if (unreachable.length > 0) {
    layers.push(unreachable)
  }

  return layers
}
