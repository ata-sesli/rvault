import { describe, expect, test } from "bun:test"

import {
  getDetailTitle,
  isDetailMode,
  nextSortState,
  sortEntries,
  type PopupMode
} from "./popup-state"

describe("popup mode helpers", () => {
  test("keeps compact mode free of the detail editor", () => {
    expect(isDetailMode("compact")).toBe(false)
  })

  test("shows detail editor for create and edit modes", () => {
    expect(isDetailMode("create")).toBe(true)
    expect(isDetailMode("edit")).toBe(true)
  })

  test("names detail modes by their user intent", () => {
    const modes: PopupMode[] = ["create", "edit"]

    expect(modes.map(getDetailTitle)).toEqual(["Save Entry", "Edit Entry"])
  })

  test("sorts entries by platform ascending with username as a tie breaker", () => {
    const entries = [
      { platform: "PyPI", userId: "zed@example.com", updatedAt: 10 },
      { platform: "Gmail", userId: "zed@example.com", updatedAt: 20 },
      { platform: "Gmail", userId: "alice@example.com", updatedAt: 30 }
    ]

    expect(sortEntries(entries, { key: "platform", direction: "asc" }).map((entry) => entry.userId)).toEqual([
      "alice@example.com",
      "zed@example.com",
      "zed@example.com"
    ])
  })

  test("sorts entries by username descending with platform as a tie breaker", () => {
    const entries = [
      { platform: "PyPI", userId: "zed@example.com", updatedAt: 10 },
      { platform: "Gmail", userId: "alice@example.com", updatedAt: 20 },
      { platform: "Apple", userId: "zed@example.com", updatedAt: 30 }
    ]

    expect(sortEntries(entries, { key: "username", direction: "desc" }).map((entry) => entry.platform)).toEqual([
      "Apple",
      "PyPI",
      "Gmail"
    ])
  })

  test("sorts entries by last updated descending", () => {
    const entries = [
      { platform: "Older", userId: "a@example.com", updatedAt: 10 },
      { platform: "Newest", userId: "b@example.com", updatedAt: 30 },
      { platform: "Middle", userId: "c@example.com", updatedAt: 20 }
    ]

    expect(sortEntries(entries, { key: "updatedAt", direction: "desc" }).map((entry) => entry.platform)).toEqual([
      "Newest",
      "Middle",
      "Older"
    ])
  })

  test("selecting the active sort key flips direction", () => {
    expect(nextSortState({ key: "platform", direction: "asc" }, "platform")).toEqual({
      key: "platform",
      direction: "desc"
    })
  })

  test("selecting a new date sort defaults to newest first", () => {
    expect(nextSortState({ key: "platform", direction: "asc" }, "updatedAt")).toEqual({
      key: "updatedAt",
      direction: "desc"
    })
  })
})
