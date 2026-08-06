import { describe, expect, test } from "bun:test"

const workflow = await Bun.file(
  new URL("../../.github/workflows/extension-release.yml", import.meta.url)
).text()

describe("extension release workflow", () => {
  test("builds and signs the Firefox extension", () => {
    expect(workflow).toContain("bun run build:firefox")
    expect(workflow).toContain("web-ext@10.3.0 sign")
    expect(workflow).toContain("AMO_JWT_ISSUER")
    expect(workflow).toContain("AMO_JWT_SECRET")
    expect(workflow).toContain('rvault-extension-firefox-${VERSION}.xpi')
  })

  test("uploads Chromium and Firefox release artifacts", () => {
    expect(workflow).toContain('"$CHROMIUM_ZIP" "$FIREFOX_XPI"')
  })
})
