export type PopupMode = "compact" | "create" | "edit"
export type SortKey = "platform" | "email"
export type SortableEntry = {
  platform: string
  userId: string
}

export function isDetailMode(mode: PopupMode): mode is Exclude<PopupMode, "compact"> {
  return mode === "create" || mode === "edit"
}

export function getDetailTitle(mode: Exclude<PopupMode, "compact">): string {
  return mode === "edit" ? "Edit Entry" : "Save Entry"
}

export function sortEntries<T extends SortableEntry>(
  entries: T[],
  sortBy: SortKey
): T[] {
  return [...entries].sort((left, right) => {
    const primary =
      sortBy === "platform"
        ? compareText(left.platform, right.platform)
        : compareText(left.userId, right.userId)

    if (primary !== 0) {
      return primary
    }

    return sortBy === "platform"
      ? compareText(left.userId, right.userId)
      : compareText(left.platform, right.platform)
  })
}

function compareText(left: string, right: string): number {
  return left.localeCompare(right, undefined, {
    numeric: true,
    sensitivity: "base"
  })
}
