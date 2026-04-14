import { useQuery } from '@tanstack/react-query'
import { getDashboardReport, getAdminConfig } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Link } from 'react-router-dom'
import { Users, Briefcase, ShoppingCart, FileText, RefreshCw, TrendingUp } from 'lucide-react'
import { formatNumber } from '@/lib/utils'
import { EntityCountBarChart, StatePieChart, TimelineAreaChart, COLORS } from '@/components/charts'
import { Badge } from '@/components/ui/badge'

const statIcons: Record<string, React.ReactNode> = {
  employees: <Users className="h-5 w-5 text-blue-600" />,
  customers: <Briefcase className="h-5 w-5 text-green-600" />,
  purchase_orders: <ShoppingCart className="h-5 w-5 text-orange-600" />,
  projects: <FileText className="h-5 w-5 text-purple-600" />,
}

const statColors: Record<string, string> = {
  employees: 'bg-blue-50',
  customers: 'bg-green-50',
  purchase_orders: 'bg-orange-50',
  projects: 'bg-purple-50',
}

export function DashboardPage() {
  const {
    data: dashboard,
    isLoading,
    refetch,
    isFetching,
  } = useQuery({
    queryKey: ['dashboard'],
    queryFn: getDashboardReport,
  })

  const { data: config } = useQuery({
    queryKey: ['admin-config'],
    queryFn: getAdminConfig,
  })

  const entityCounts = (dashboard?.data?.entity_counts ?? {}) as Record<string, number>
  const byState = (dashboard?.data?.by_state ?? {}) as Record<string, Record<string, number>>
  const recentActivity = (dashboard?.data?.recent_activity ?? []) as { entity: string; action: string; timestamp: string; description?: string }[]
  const timeline = (dashboard?.data?.timeline ?? []) as { date: string; count: number }[]

  // Build chart data
  const entityChartData = Object.entries(entityCounts).map(([name, count]) => ({
    name,
    count,
  }))

  // Build state distribution pie chart data across all entities
  const statePieData = Object.entries(byState).flatMap(([entity, states]) =>
    Object.entries(states).map(([state, count]) => ({
      name: state,
      value: count,
    }))
  )
  // Aggregate same states
  const aggregatedStates = new Map<string, number>()
  for (const item of statePieData) {
    aggregatedStates.set(item.name, (aggregatedStates.get(item.name) ?? 0) + item.value)
  }
  const pieData = Array.from(aggregatedStates.entries()).map(([name, value]) => ({ name, value }))

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
          <p className="text-muted-foreground">Welcome to Atlas ERP — your business at a glance</p>
        </div>
        <Button variant="outline" onClick={() => refetch()} disabled={isFetching}>
          <RefreshCw className={`mr-2 h-4 w-4 ${isFetching ? 'animate-spin' : ''}`} />
          Refresh
        </Button>
      </div>

      {/* KPI Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {(config?.entities ?? []).slice(0, 8).map((entity, idx) => (
          <Card key={entity}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium capitalize">{entity.replace(/_/g, ' ')}</CardTitle>
              <div className={`rounded-lg p-2 ${statColors[entity] ?? 'bg-muted'}`}>
                {statIcons[entity] ?? <FileText className="h-5 w-5 text-gray-600" />}
              </div>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {isLoading ? '—' : formatNumber(entityCounts[entity] ?? 0)}
              </div>
              <p className="text-xs text-muted-foreground">
                <Link to={`/${entity}`} className="text-primary hover:underline">
                  View all →
                </Link>
              </p>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Charts row */}
      <div className="grid gap-6 lg:grid-cols-2">
        {/* Entity count bar chart */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <TrendingUp className="h-4 w-4" />
              Records by Entity
            </CardTitle>
          </CardHeader>
          <CardContent>
            {entityChartData.length > 0 ? (
              <EntityCountBarChart data={entityChartData} />
            ) : (
              <div className="flex h-[200px] items-center justify-center text-muted-foreground text-sm">
                No entity data available
              </div>
            )}
          </CardContent>
        </Card>

        {/* State distribution pie chart */}
        <Card>
          <CardHeader>
            <CardTitle>Status Distribution</CardTitle>
          </CardHeader>
          <CardContent>
            {pieData.length > 0 ? (
              <StatePieChart data={pieData} />
            ) : (
              <div className="flex h-[200px] items-center justify-center text-muted-foreground text-sm">
                No workflow state data available
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Timeline chart + Quick actions */}
      <div className="grid gap-6 lg:grid-cols-3">
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Activity Over Time</CardTitle>
          </CardHeader>
          <CardContent>
            {timeline.length > 0 ? (
              <TimelineAreaChart data={timeline} label="Records created" />
            ) : (
              <div className="flex h-[200px] items-center justify-center text-muted-foreground text-sm">
                <div className="text-center">
                  <p>No timeline data available yet.</p>
                  <p className="text-xs mt-1">Activity will appear here as records are created.</p>
                </div>
              </div>
            )}
          </CardContent>
        </Card>

        {/* Quick actions */}
        <Card>
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid gap-3">
              <Link to="/employees?new=true">
                <Button variant="outline" className="w-full justify-start gap-2">
                  <Users className="h-4 w-4" /> Add Employee
                </Button>
              </Link>
              <Link to="/customers?new=true">
                <Button variant="outline" className="w-full justify-start gap-2">
                  <Briefcase className="h-4 w-4" /> Add Customer
                </Button>
              </Link>
              <Link to="/orders?new=true">
                <Button variant="outline" className="w-full justify-start gap-2">
                  <ShoppingCart className="h-4 w-4" /> New Order
                </Button>
              </Link>
              <Link to="/reports">
                <Button variant="outline" className="w-full justify-start gap-2">
                  <FileText className="h-4 w-4" /> View Reports
                </Button>
              </Link>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Recent activity */}
      {recentActivity.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3">
              {recentActivity.slice(0, 10).map((entry, i) => (
                <div key={i} className="flex items-center gap-3">
                  <div className="h-2 w-2 rounded-full bg-primary shrink-0" />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm truncate">
                      <span className="font-medium capitalize">{entry.entity.replace(/_/g, ' ')}</span>
                      {' — '}
                      <Badge variant="secondary" className="text-[10px]">{entry.action}</Badge>
                      {entry.description && (
                        <span className="text-muted-foreground">: {entry.description}</span>
                      )}
                    </p>
                  </div>
                  <span className="text-xs text-muted-foreground whitespace-nowrap">
                    {new Date(entry.timestamp).toLocaleString('en-US', {
                      month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
                    })}
                  </span>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* System info */}
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>System Status</CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Entities loaded</span>
              <span className="font-medium">{config?.entities?.length ?? 0}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Schema version</span>
              <span className="font-medium">{config?.version ?? '—'}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Total records</span>
              <span className="font-medium">
                {Object.values(entityCounts).reduce((a, b) => a + b, 0).toLocaleString()}
              </span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Getting Started</CardTitle>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground space-y-2">
            <p>Use the sidebar to browse entities. Go to <Link to="/admin" className="text-primary hover:underline">Admin</Link> to manage schemas and create new entity types dynamically.</p>
            <p>Every entity supports configurable workflows, validation rules, and computed fields — all defined declaratively.</p>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
