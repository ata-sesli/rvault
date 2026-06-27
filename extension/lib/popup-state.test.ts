import { describe, expect, test } from "bun:test"

import {
  getDetailTitle,
  isDetailMode,
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

  test("sorts entries by platform with email as a tie breaker", () => {
    const entries = [
      { platform: "PyPI", userId: "zed@example.com" },
      { platform: "Gmail", userId: "zed@example.com" },
      { platform: "Gmail", userId: "alice@example.com" }
    ]

    expect(sortEntries(entries, "platform").map((entry) => entry.userId)).toEqual([
      "alice@example.com",
      "zed@example.com",
      "zed@example.com"
    ])
  })

  test("sorts entries by email with platform as a tie breaker", () => {
    const entries = [
      { platform: "PyPI", userId: "zed@example.com" },
      { platform: "Gmail", userId: "alice@example.com" },
      { platform: "Apple", userId: "zed@example.com" }
    ]

    expect(sortEntries(entries, "email").map((entry) => entry.platform)).toEqual([
      "Gmail",
      "Apple",
      "PyPI"
    ])
  })
})
