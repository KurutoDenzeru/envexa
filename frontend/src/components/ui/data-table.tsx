import { useState } from "react"
import {
  useReactTable,
  getCoreRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  flexRender,
  type ColumnDef,
  type SortingState,
  type PaginationState,
} from "@tanstack/react-table"
import {
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  TableCell,
} from "@/components/ui/table"
import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationLink,
  PaginationPrevious,
  PaginationNext,
} from "@/components/ui/pagination"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { ArrowUp, ArrowDown, ArrowUpDown } from "lucide-react"

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[]
  data: TData[]
  defaultPageSize?: number
  pageSizeOptions?: number[]
  className?: string
  containerClassName?: string
  /** Called when page index or page size changes */
  onPageChange?: (page: number, pageSize: number) => void
}

function getPageNumbers(
  currentPage: number,
  totalPages: number,
): (number | string)[] {
  if (totalPages <= 7) {
    return Array.from({ length: totalPages }, (_, i) => i)
  }
  const pages: (number | string)[] = [0]
  if (currentPage > 3) pages.push("...")
  const start = Math.max(1, currentPage - 1)
  const end = Math.min(totalPages - 2, currentPage + 1)
  for (let i = start; i <= end; i++) pages.push(i)
  if (currentPage < totalPages - 4) pages.push("...")
  pages.push(totalPages - 1)
  return pages
}

export function DataTable<TData, TValue>({
  columns,
  data,
  defaultPageSize = 10,
  pageSizeOptions = [5, 10, 25, 50, 100],
  className,
  containerClassName,
  onPageChange,
}: DataTableProps<TData, TValue>) {
  const [sorting, setSorting] = useState<SortingState>([])
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: defaultPageSize,
  })

  const handlePaginationChange = (updater: PaginationState | ((old: PaginationState) => PaginationState)) => {
    setPagination((old) => {
      const next = typeof updater === "function" ? updater(old) : updater
      onPageChange?.(next.pageIndex, next.pageSize)
      return next
    })
  }

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: setSorting,
    onPaginationChange: handlePaginationChange,
    state: { sorting, pagination },
  })

  const filteredCount = table.getFilteredRowModel().rows.length
  const pageSize = table.getState().pagination.pageSize
  const pageIndex = table.getState().pagination.pageIndex

  return (
    <div className={containerClassName}>
      <div className="rounded-md border border-border overflow-hidden">
        <Table className={className}>
          <TableHeader className="bg-muted/50">
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow
                key={headerGroup.id}
                className="border-border hover:bg-transparent"
              >
                {headerGroup.headers.map((header) => {
                  const canSort = header.column.getCanSort()
                  const sorted = header.column.getIsSorted()
                  return (
                    <TableHead
                      key={header.id}
                      className={
                        canSort
                          ? "cursor-pointer select-none hover:text-foreground transition-colors"
                          : ""
                      }
                      onClick={canSort ? header.column.getToggleSortingHandler() : undefined}
                    >
                      <div className="flex items-center gap-1.5">
                        {flexRender(
                          header.column.columnDef.header,
                          header.getContext(),
                        )}
                        {canSort && (
                          <span className="inline-flex shrink-0">
                            {sorted === "asc" ? (
                              <ArrowUp className="h-3.5 w-3.5 text-foreground" />
                            ) : sorted === "desc" ? (
                              <ArrowDown className="h-3.5 w-3.5 text-foreground" />
                            ) : (
                              <ArrowUpDown className="h-3.5 w-3.5 text-muted-foreground/40" />
                            )}
                          </span>
                        )}
                      </div>
                    </TableHead>
                  )
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  data-state={row.getIsSelected() ? "selected" : undefined}
                  className="border-border hover:bg-muted/50 transition-colors"
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id}>
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center text-muted-foreground"
                >
                  No results.
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      {/* Pagination footer */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3 mt-4 pt-4 border-t border-border/50">
        {/* Left: rows per page */}
        <div className="flex items-center gap-2">
          <p className="text-xs text-muted-foreground whitespace-nowrap">
            Rows per page
          </p>
          <Select
            value={String(pageSize)}
            onValueChange={(v) => {
              if (v) {
                table.setPageSize(Number(v))
              }
            }}
          >
            <SelectTrigger className="w-[70px] h-8 bg-popover border-border text-xs">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {pageSizeOptions.map((size) => (
                <SelectItem key={size} value={String(size)}>
                  {size}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* Center: page info */}
        <p className="text-xs text-muted-foreground tabular-nums">
          {filteredCount === 0
            ? "0 results"
            : `${pageIndex * pageSize + 1}–${Math.min((pageIndex + 1) * pageSize, filteredCount)} of ${filteredCount}`}
        </p>

        {/* Right: page navigation */}
        <Pagination className="mx-0 w-auto">
          <PaginationContent>
            <PaginationItem>
              <PaginationPrevious
                onClick={() => table.previousPage()}
                className={
                  !table.getCanPreviousPage()
                    ? "pointer-events-none opacity-50"
                    : "cursor-pointer"
                }
              />
            </PaginationItem>
            {getPageNumbers(pageIndex, table.getPageCount()).map(
              (page, i) => (
                <PaginationItem key={i}>
                  {page === "..." ? (
                    <span className="px-2 h-9 flex items-center text-muted-foreground text-xs">
                      …
                    </span>
                  ) : (
                    <PaginationLink
                      onClick={() => table.setPageIndex(page as number)}
                      isActive={page === pageIndex}
                      className="cursor-pointer"
                    >
                      {(page as number) + 1}
                    </PaginationLink>
                  )}
                </PaginationItem>
              ),
            )}
            <PaginationItem>
              <PaginationNext
                onClick={() => table.nextPage()}
                className={
                  !table.getCanNextPage()
                    ? "pointer-events-none opacity-50"
                    : "cursor-pointer"
                }
              />
            </PaginationItem>
          </PaginationContent>
        </Pagination>
      </div>
    </div>
  )
}
