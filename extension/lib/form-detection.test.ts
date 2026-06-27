import { describe, expect, test } from "bun:test"

import {
  detectLoginFields,
  getPlatformSuggestion,
  type FieldDescriptor
} from "./form-detection"

describe("detectLoginFields", () => {
  test("finds username and password fields in a login form", () => {
    const fields: FieldDescriptor[] = [
      { index: 0, type: "email", name: "email", id: "email" },
      { index: 1, type: "password", name: "password", id: "password" }
    ]

    expect(detectLoginFields(fields)).toEqual({
      usernameIndex: 0,
      passwordIndex: 1
    })
  })

  test("prefers autocomplete username over unrelated text fields", () => {
    const fields: FieldDescriptor[] = [
      { index: 0, type: "text", name: "search", id: "site-search" },
      { index: 1, type: "text", name: "login", id: "login", autocomplete: "username" },
      { index: 2, type: "password", name: "password", id: "password" }
    ]

    expect(detectLoginFields(fields)).toEqual({
      usernameIndex: 1,
      passwordIndex: 2
    })
  })

  test("uses placeholder text to identify username fields", () => {
    const fields: FieldDescriptor[] = [
      { index: 0, type: "text", name: "q", id: "query" },
      { index: 1, type: "text", placeholder: "Email address" },
      { index: 2, type: "password", placeholder: "Password" }
    ]

    expect(detectLoginFields(fields)).toEqual({
      usernameIndex: 1,
      passwordIndex: 2
    })
  })

  test("uses associated label text to identify username fields", () => {
    const fields: FieldDescriptor[] = [
      { index: 0, type: "text", label: "Account username" },
      { index: 1, type: "password", label: "Account password" }
    ]

    expect(detectLoginFields(fields)).toEqual({
      usernameIndex: 0,
      passwordIndex: 1
    })
  })

  test("returns null when no password field exists", () => {
    const fields: FieldDescriptor[] = [
      { index: 0, type: "email", name: "email", id: "email" }
    ]

    expect(detectLoginFields(fields)).toBeNull()
  })
})

describe("getPlatformSuggestion", () => {
  test("uses hostname without a leading www prefix", () => {
    expect(getPlatformSuggestion("https://www.github.com/login")).toBe("github.com")
  })
})
