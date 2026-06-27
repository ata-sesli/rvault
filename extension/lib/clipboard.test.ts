import { describe, expect, test } from "bun:test"

import { copyTextToClipboard } from "./clipboard"

describe("clipboard", () => {
  test("writes text to the provided clipboard", async () => {
    let copied = ""

    await copyTextToClipboard("secret", {
      writeText: async (text) => {
        copied = text
      }
    })

    expect(copied).toBe("secret")
  })

  test("reports when clipboard access is unavailable", async () => {
    await expect(copyTextToClipboard("secret", undefined)).rejects.toThrow(
      "Clipboard is unavailable."
    )
  })
})
