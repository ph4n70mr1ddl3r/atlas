import { useQuery } from '@tanstack/react-query'
import { getEntityReport, getAdminConfig, getDashboardReport } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { useState } from 'react'
import { formatNumber } from '@/lib/utils'
import { StatePieChart, EntityCountBarChart, TimelineAreaChart, COLORS } from '@/components/charts'
import { BarChart3, TrendingUp, FileText, Download } from 'lucide-react'

export function ReportsPage() {
  const { data: config } = useQuery({ queryKey: ['admin-config'], queryFn: getAdminConfig })
  const { data: dashboard } = useQuery({ queryKey: ['dashboard'], queryFn: getDashboardReport })
  const [selected, setSelected] = useState<string | null>(null)

  const { data: report, isLoading } = useQuery({
    queryKey: ['report', selected],
    queryFn: () => getEntityReport(selected!),
    enabled: !!selected,
  })

  const entityCounts = (dashboard?.data?.entity_counts ?? {}) as Record<string, number>
  const byState = (report?.data?.by_state ?? {}) as Record<string, number> | undefined
  const timeline = (report?.data?.timeline ?? []) as { date: string; count: number }[] | undefined
  const topRecords = (report?.data?.top_records ?? []) as Record<string, unknown>[] | undefined
  const fieldStats = (report?.data?.field_stats ?? {}) as Record<string, { min: number; max: number; avg: number }> | undefined

  // Build overview chart data
  const overviewChartData = Object.entries(entityCounts).map(([name, count]) => ({
    name,
    count,
  }))

  // Build state pie data
  const statePieData = byState
    ? Object.entries(byState).map(([name, value]) => ({ name, value }))
    : []

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">Reports</h1>
          <p className="text-muted-foreground">Summary reports and analytics by entity</p>
        </div>
        {selected && (
          <Button variant="outline" size="sm" onClick={() => setSelected(null)}>
            ← All Reports
          </Button>
        )}
      </div>

      {!selected ? (
        <>
          {/* Overview section */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <BarChart3 className="h-5 w-5" />
                Overview
              </CardTitle>
            </CardHeader>
            <CardContent>
              {overviewChartData.length > 0 ? (
                <EntityCountBarChart data={overviewChartData} />
              ) : (
                <div className="flex h-[200px] items-center justify-center text-muted-foreground text-sm">
                  No data available. Create entities and add records to see analytics.
                </div>
              )}
            </CardContent>
          </Card>

          {/* Entity selector */}
          <div>
            <h3 className="text-sm font-medium text-muted-foreground mb-3">Select an entity for detailed report:</h3>
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {(config?.entities ?? []).map((entity, idx) => (
                <Card
                  key={entity}
                  className="cursor-pointer hover:border-primary/50 transition-colors"
                  onClick={() => setSelected(entity)}
                >
                  <CardContent className="p-4">
                    <div className="flex items-center gap-3">
                      <div
                        className="flex h-10 w-10 items-center justify-center rounded-lg text-white text-sm font-bold"
                        style={{ backgroundColor: COLORS[idx % COLORS.length] }}
                      >
                        {entity.charAt(0).toUpperCase()}
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="font-medium capitalize truncate">{entity.replace(/_/g, ' ')}</p>
                        <p className="text-sm text-muted-foreground">
                          {formatNumber(entityCounts[entity] ?? 0)} records
                        </p>
                      </div>
                      <TrendingUp className="h-4 w-4 text-muted-foreground" />
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        </>
      ) : (
        <div className="space-y-6">
          {/* Entity report header */}
          <div className="flex items-center gap-3">
            <FileText className="h-6 w-6 text-primary" />
            <h2 className="text-2xl font-bold capitalize">{selected.replace(/_/g, ' ')} Report</h2>
          </div>

          {/* Summary cards */}
          <div className="grid gap-4 md:grid-cols-3">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Total Records</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {isLoading ? '—' : formatNumber((report?.data?.total_records as number) ?? entityCounts[selected] ?? 0)}
                </div>
              </CardContent>
            </Card>

            {fieldStats && Object.keys(fieldStats).length > 0 && (
              Object.entries(fieldStats).slice(0, 2).map(([fieldName, stats]) => (
                <Card key={fieldName}>
                  <CardHeader className="pb-2">
                    <CardTitle className="text-sm font-medium capitalize">{fieldName.replace(/_/g, ' ')} Stats</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="text-lg font-bold font-mono">
                      avg: {stats.avg.toFixed(1)}
                    </div>
                    <p className="text-xs text-muted-foreground">
                      range: {stats.min.toFixed(0)} – {stats.max.toFixed(0)}
                    </p>
                  </CardContent>
                </Card>
              ))
            )}
          </div>

          {/* Charts */}
          <div className="grid gap-6 lg:grid-cols-2">
            {/* State distribution */}
            {statePieData.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-base">Status Distribution</CardTitle>
                </CardHeader>
                <CardContent>
                  <StatePieChart data={statePieData} />
                </CardContent>
              </Card>
            )}

            {/* Timeline */}
            {timeline && timeline.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle className="text-base">Activity Over Time</CardTitle>
                </CardHeader>
                <CardContent>
                  <TimelineAreaChart data={timeline} label={`${selected} created`} />
                </CardContent>
              </Card>
            )}
          </div>

          {/* State breakdown table */}
          {statePieData.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Breakdown by Status</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {statePieData.map((item, idx) => {
                    const total = statePieData.reduce((a, b) => a + b.value, 0)
                    const pct = total > 0 ? (item.value / total) * 100 : 0
                    return (
                      <div key={item.name} className="flex items-center gap-3">
                        <div
                          className="h-3 w-3 rounded-full shrink-0"
                          style={{ backgroundColor: COLORS[idx % COLORS.length] }}
                        />
                        <span className="text-sm capitalize min-w-[100px]">{item.name.replace(/_/g, ' ')}</span>
                        <div className="flex-1 h-2 rounded-full bg-muted overflow-hidden">
                          <div
                            className="h-full rounded-full"
                            style={{
                              width: `${pct}%`,
                              backgroundColor: COLORS[idx % COLORS.length],
                            }}
                          />
                        </div>
                        <span className="text-sm font-medium min-w-[60px] text-right">
                          {formatNumber(item.value)}
                        </span>
                        <span className="text-xs text-muted-foreground min-w-[40px] text-right">
                          {pct.toFixed(0)}%
                        </span>
                      </div>
                    )
                  })}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Top records */}
          {topRecords && topRecords.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Recent Records</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="divide-y">
                  {topRecords.slice(0, 10).map((rec, i) => (
                    <div key={i} className="flex items-center gap-3 py-2">
                      <span className="text-sm text-muted-foreground w-6">{i + 1}.</span>
                      <span className="text-sm font-medium truncate flex-1">
                        {String(rec.name ?? rec.title ?? rec.id ?? `Record ${i + 1}`)}
                      </span>
                      {rec.workflow_state != null && (
                        <Badge variant="info" className="text-[10px]">
                          {String(rec.workflow_state).replace(/_/g, ' ')}
                        </Badge>
                      )}
                      {rec.created_at != null && (
                        <span className="text-xs text-muted-foreground">
                          {new Date(rec.created_at as string).toLocaleDateString()}
                        </span>
                      )}
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {statePieData.length === 0 && (!timeline || timeline.length === 0) && (
            <Card>
              <CardContent className="flex h-48 items-center justify-center">
                <div className="text-center">
                  <BarChart3 className="mx-auto h-12 w-12 text-muted-foreground/30 mb-3" />
                  <p className="text-muted-foreground">No detailed report data available for this entity.</p>
                  <p className="text-xs text-muted-foreground mt-1">Add records with workflows to see analytics.</p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}
    </div>
  )
}
