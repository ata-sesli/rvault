export type PopupMode = "compact" | "create" | "edit"
export type SortKey = "platform" | "username" | "updatedAt"
export type SortDirection = "asc" | "desc"
export type SortState = {
  key: SortKey
  direction: SortDirection
}
export type SortableEntry = {
  platform: string
  userId: string
  updatedAt: number
}

export function isDetailMode(mode: PopupMode): mode is Exclude<PopupMode, "compact"> {
  return mode === "create" || mode === "edit"
}

export function getDetailTitle(mode: Exclude<PopupMode, "compact">): string {
  return mode === "edit" ? "Edit Entry" : "Save Entry"
}

export function sortEntries<T extends SortableEntry>(
  entries: T[],
  sort: SortState
): T[] {
  return [...entries].sort((left, right) => {
    const primary = compareBySortKey(left, right, sort.key)

    if (primary !== 0) {
      return sort.direction === "asc" ? primary : -primary
    }

    const fallback = sort.key === "platform"
      ? compareText(left.userId, right.userId)
      : compareText(left.platform, right.platform)

    return fallback
  })
}

export function nextSortState(current: SortState, key: SortKey): SortState {
  if (current.key === key) {
    return {
      key,
      direction: current.direction === "asc" ? "desc" : "asc"
    }
  }

  return {
    key,
    direction: key === "updatedAt" ? "desc" : "asc"
  }
}

function compareBySortKey(
  left: SortableEntry,
  right: SortableEntry,
  sortBy: SortKey
): number {
  if (sortBy === "updatedAt") {
    return left.updatedAt - right.updatedAt
  }

  return sortBy === "platform"
    ? compareText(left.platform, right.platform)
    : compareText(left.userId, right.userId)
}

function compareText(left: string, right: string): number {
  return left.localeCompare(right, undefined, {
    numeric: true,
    sensitivity: "base"
  })
}
