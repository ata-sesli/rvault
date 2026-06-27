import { describe, expect, test } from "bun:test"

import { logoSrcFromDataBase64Import } from "./logo"

describe("logo source", () => {
  test("does not prefix an import that is already a data URL", () => {
    const dataUrl = "data:image/png;base64,iVBORw0KGgo="

    expect(logoSrcFromDataBase64Import(dataUrl)).toBe(dataUrl)
  })

  test("prefixes a raw base64 import defensively", () => {
    expect(logoSrcFromDataBase64Import("iVBORw0KGgo=")).toBe(
      "data:image/png;base64,iVBORw0KGgo="
    )
  })
})
