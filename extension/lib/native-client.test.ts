import { describe, expect, test } from "bun:test"

import {
  HOST_NAME,
  createHostRequest,
  parseHostResponse,
  type HostResponse
} from "./native-client"

describe("native-client", () => {
  test("uses the RVault native host name", () => {
    expect(HOST_NAME).toBe("io.github.ata_sesli.rvault")
  })

  test("creates a typed fill request", () => {
    expect(
      createHostRequest("get", {
        platform: "github",
        userId: "alice"
      })
    ).toEqual({
      type: "get",
      platform: "github",
      userId: "alice"
    })
  })

  test("creates a quit request", () => {
    expect(createHostRequest("quit", {})).toEqual({ type: "quit" })
  })

  test("normalizes successful host responses", () => {
    const response: HostResponse<{ locked: boolean }> = {
      ok: true,
      data: { locked: false }
    }

    expect(parseHostResponse(response)).toEqual({ locked: false })
  })

  test("throws host error messages", () => {
    const response: HostResponse<never> = {
      ok: false,
      error: { code: "locked", message: "Vault is locked." }
    }

    expect(() => parseHostResponse(response)).toThrow("Vault is locked.")
  })
})
