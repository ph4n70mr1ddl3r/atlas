import { useQuery } from '@tanstack/react-query'
import { getEntityReport, getAdminConfig, getDashboardReport } from '@/lib/api'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { useState } from 'react'
import { formatNumber } from '@/lib/utils'

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

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Reports</h1>
        <p className="text-muted-foreground">Summary reports by entity</p>
      </div>

      {/* Entity selector */}
      <div className="flex flex-wrap gap-2">
        {(config?.entities ?? []).map((entity) => (
          <Button
            key={entity}
            variant={selected === entity ? 'default' : 'outline'}
            size="sm"
            onClick={() => setSelected(entity)}
          >
            {entity.replace(/_/g, ' ')}
          </Button>
        ))}
      </div>

      {selected && (
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

          {(report?.data?.by_state as Record<string, number> | undefined) && Object.keys(report?.data?.by_state as Record<string, number>).length > 0 && (
            <Card className="md:col-span-2">
              <CardHeader>
                <CardTitle className="text-base">By State</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid gap-2 sm:grid-cols-3">
                  {Object.entries(report!.data!.by_state as Record<string, number>).map(([state, count]) => (
                    <div key={state} className="rounded-lg border p-3">
                      <p className="text-sm text-muted-foreground capitalize">{state.replace(/_/g, ' ')}</p>
                      <p className="text-xl font-bold">{formatNumber(count)}</p>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      )}

      {!selected && (
        <Card>
          <CardContent className="flex h-48 items-center justify-center">
            <p className="text-muted-foreground">Select an entity above to view its report.</p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
