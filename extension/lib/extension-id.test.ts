import { describe, expect, test } from "bun:test"
import { createHash } from "node:crypto"

import packageJson from "../package.json"

const RVAULT_HELIUM_EXTENSION_ID = "gnfmkmiklgghclejbbdmjgcldajahfhh"
const RVAULT_EXTENSION_KEY =
  "MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAxeqUqKARu/O10PNBTdVB+ZHHg2KdA0C8DQTw65gxodLMkhL3zFSCTRavtRpFq/glcI65wipR4+S+9bVy/B/JSEcLzEz1d15/8EDPxGSycu92xkxP7OK/eLU/hnJJjWYg9sDstuQFhiRixPpiPGwvYAgMMPhUbrX74Y5UjqRMhAmI982uugvUblp6SKx7EtXvYB0mJFztMsyxjRhYAnI5eEOjUskmDfODaWdYcSXmQApTCG1UQhTb+Gv/garaTV6YohdhEMGEdt939K93nttJOjzXcqQuCIuvdZJQADxDcNRaulEB0ZHzpKZLq61kQVWJc5YrPAdOGLKLnaCPJ+QldwIDAQAB"

function extensionIdFromKey(key: string) {
  const digest = createHash("sha256").update(Buffer.from(key, "base64")).digest()

  return Array.from(digest.subarray(0, 16))
    .map((byte) =>
      [byte >> 4, byte & 0x0f]
        .map((nibble) => String.fromCharCode("a".charCodeAt(0) + nibble))
        .join("")
    )
    .join("")
}

describe("extension identity", () => {
  test("declares the Chrome extension name used by Plasmo", () => {
    expect((packageJson as { displayName?: string }).displayName).toBe("RVault")
  })

  test("declares a toolbar popup for the extension action", () => {
    expect(packageJson.manifest?.action?.default_popup).toBe("./popup.html")
  })

  test("uses the manifest key known by rvault browser enable", () => {
    const key = packageJson.manifest?.key

    expect(key).toBe(RVAULT_EXTENSION_KEY)
    expect(extensionIdFromKey(key as string)).toBe(RVAULT_HELIUM_EXTENSION_ID)
  })
})
