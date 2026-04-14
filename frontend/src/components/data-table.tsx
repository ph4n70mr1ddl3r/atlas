import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  flexRender,
  type ColumnDef,
  type SortingState,
} from '@tanstack/react-table'
import { useState, useEffect } from 'react'
import { ChevronUp, ChevronDown, ChevronsUpDown, ChevronLeft, ChevronRight } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[]
  data: TData[]
  total?: number
  pageCount?: number
  /** Current page index (0-based) */
  pageIndex?: number
  isLoading?: boolean
  onRowClick?: (row: TData) => void
  /** Called when sorting changes */
  onSortingChange?: (sorting: SortingState) => void
  /** Called when page changes */
  onPageChange?: (page: number) => void
}

/**
 * Data table with sorting, filtering, and pagination.
 * Supports both client-side and server-side modes.
 * When `pageCount` and `total` are provided, operates in server-side mode.
 */
export function DataTable<TData, TValue>({
  columns,
  data,
  total,
  pageCount,
  pageIndex = 0,
  isLoading,
  onRowClick,
  onSortingChange,
  onPageChange,
}: DataTableProps<TData, TValue>) {
  const [sorting, setSorting] = useState<SortingState>([])
  const isServerSide = pageCount != null

  // Propagate sorting changes to parent for server-side mode
  useEffect(() => {
    if (isServerSide && onSortingChange) {
      onSortingChange(sorting)
    }
  }, [sorting, isServerSide, onSortingChange])

  const table = useReactTable({
    data,
    columns,
    pageCount: isServerSide ? pageCount : undefined,
    state: {
      sorting,
      ...(isServerSide ? { pagination: { pageIndex, pageSize: 20 } } : {}),
    },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: isServerSide ? undefined : getSortedRowModel(),
    manualPagination: isServerSide,
    manualSorting: isServerSide,
  })

  const currentPage = isServerSide ? pageIndex : table.getState().pagination.pageIndex
  const totalPages = isServerSide ? (pageCount ?? 1) : table.getPageCount()

  return (
    <div className="rounded-md border">
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead className="bg-muted/50">
            {table.getHeaderGroups().map((hg) => (
              <tr key={hg.id}>
                {hg.headers.map((header) => (
                  <th
                    key={header.id}
                    className={cn(
                      'h-12 px-4 text-left align-middle font-medium text-muted-foreground whitespace-nowrap',
                      header.column.getCanSort() && 'cursor-pointer select-none hover:bg-muted',
                    )}
                    onClick={header.column.getToggleSortingHandler()}
                  >
                    <div className="flex items-center gap-1">
                      {header.isPlaceholder ? null : flexRender(header.column.columnDef.header, header.getContext())}
                      {{
                        asc: <ChevronUp className="h-4 w-4" />,
                        desc: <ChevronDown className="h-4 w-4" />,
                      }[header.column.getIsSorted() as string] ??
                        (header.column.getCanSort() ? (
                          <ChevronsUpDown className="h-3 w-3 opacity-40" />
                        ) : null)}
                    </div>
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody>
            {isLoading ? (
              <tr>
                <td colSpan={columns.length} className="h-24 text-center text-muted-foreground">
                  <div className="flex items-center justify-center gap-2">
                    <div className="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent" />
                    Loading…
                  </div>
                </td>
              </tr>
            ) : table.getRowModel().rows.length ? (
              table.getRowModel().rows.map((row) => (
                <tr
                  key={row.id}
                  className={cn(
                    'border-t transition-colors hover:bg-muted/50',
                    onRowClick && 'cursor-pointer',
                  )}
                  onClick={() => onRowClick?.(row.original)}
                >
                  {row.getVisibleCells().map((cell) => (
                    <td key={cell.id} className="px-4 py-3 align-middle whitespace-nowrap">
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </td>
                  ))}
                </tr>
              ))
            ) : (
              <tr>
                <td colSpan={columns.length} className="h-24 text-center text-muted-foreground">
                  No records found.
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      <div className="flex items-center justify-between border-t px-4 py-3">
        <div className="text-sm text-muted-foreground">
          {total != null
            ? `${total.toLocaleString()} record${total !== 1 ? 's' : ''} total`
            : `${data.length} rows`}
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              if (isServerSide) onPageChange?.(currentPage - 1)
              else table.previousPage()
            }}
            disabled={currentPage === 0}
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>
          <span className="text-sm text-muted-foreground min-w-[80px] text-center">
            Page {currentPage + 1} of {Math.max(totalPages, 1)}
          </span>
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              if (isServerSide) onPageChange?.(currentPage + 1)
              else table.nextPage()
            }}
            disabled={currentPage >= totalPages - 1}
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  )
}
