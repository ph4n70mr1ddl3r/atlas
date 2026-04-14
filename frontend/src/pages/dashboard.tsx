import { useQuery } from '@tanstack/react-query'
import { getDashboardReport, getAdminConfig } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Link } from 'react-router-dom'
import { Users, Briefcase, ShoppingCart, FileText, RefreshCw } from 'lucide-react'
import { formatNumber } from '@/lib/utils'

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
        {(config?.entities ?? []).map((entity) => (
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

      {/* Quick actions */}
      <Card>
        <CardHeader>
          <CardTitle>Quick Actions</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
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
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Getting Started</CardTitle>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground">
            <p>Use the sidebar to browse entities. Go to Admin to manage schemas and create new entity types dynamically.</p>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
