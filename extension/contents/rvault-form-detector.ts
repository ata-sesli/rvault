import type { PlasmoCSConfig } from "plasmo"

import {
  detectLoginFields,
  getPlatformSuggestion,
  type FieldDescriptor
} from "../lib/form-detection"

export const config: PlasmoCSConfig = {
  matches: ["http://*/*", "https://*/*"],
  run_at: "document_idle"
}

type DetectedForm = {
  platformSuggestion: string
  username: string
  password: string
  usernameIndex: number
  passwordIndex: number
}

type ContentMessage =
  | { kind: "rvault:get-detected-form" }
  | { kind: "rvault:fill"; username: string; password: string }

let detectedForm: DetectedForm | null = null
let fieldRefs: HTMLInputElement[] = []

function visibleInputs(): HTMLInputElement[] {
  return Array.from(document.querySelectorAll<HTMLInputElement>("input")).filter((input) => {
    const type = (input.getAttribute("type") || "text").toLowerCase()
    if (["hidden", "submit", "button", "checkbox", "radio", "file"].includes(type)) {
      return false
    }
    if (input.disabled || input.readOnly) {
      return false
    }
    const rect = input.getBoundingClientRect()
    return rect.width > 0 && rect.height > 0
  })
}

function describeInput(input: HTMLInputElement, index: number): FieldDescriptor {
  return {
    index,
    type: input.getAttribute("type") || "text",
    name: input.getAttribute("name") || undefined,
    id: input.id || undefined,
    autocomplete: input.getAttribute("autocomplete") || undefined,
    placeholder: input.getAttribute("placeholder") || undefined,
    ariaLabel: input.getAttribute("aria-label") || undefined,
    label: labelTextFor(input)
  }
}

function labelTextFor(input: HTMLInputElement): string | undefined {
  const labels = Array.from(input.labels || [])
    .map((label) => label.textContent?.trim())
    .filter(Boolean)

  const labelledBy = (input.getAttribute("aria-labelledby") || "")
    .split(/\s+/)
    .filter(Boolean)
    .map((id) => document.getElementById(id)?.textContent?.trim())
    .filter(Boolean)

  const text = [...labels, ...labelledBy].join(" ").trim()
  return text || undefined
}

function refreshDetectedForm() {
  fieldRefs = visibleInputs()
  const descriptors = fieldRefs.map(describeInput)
  const loginFields = detectLoginFields(descriptors)

  if (!loginFields) {
    detectedForm = null
    return
  }

  detectedForm = {
    platformSuggestion: getPlatformSuggestion(location.href),
    username: fieldRefs[loginFields.usernameIndex]?.value || "",
    password: fieldRefs[loginFields.passwordIndex]?.value || "",
    usernameIndex: loginFields.usernameIndex,
    passwordIndex: loginFields.passwordIndex
  }
}

function setInputValue(input: HTMLInputElement | undefined, value: string) {
  if (!input) {
    return
  }

  const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set
  setter?.call(input, value)
  input.dispatchEvent(new Event("input", { bubbles: true }))
  input.dispatchEvent(new Event("change", { bubbles: true }))
}

function fillDetectedForm(username: string, password: string): boolean {
  refreshDetectedForm()
  if (!detectedForm) {
    return false
  }

  setInputValue(fieldRefs[detectedForm.usernameIndex], username)
  setInputValue(fieldRefs[detectedForm.passwordIndex], password)
  refreshDetectedForm()
  return true
}

refreshDetectedForm()
document.addEventListener("input", refreshDetectedForm, true)
document.addEventListener("change", refreshDetectedForm, true)

chrome.runtime.onMessage.addListener((message: ContentMessage, _sender, sendResponse) => {
  if (message?.kind === "rvault:get-detected-form") {
    refreshDetectedForm()
    sendResponse(detectedForm)
    return true
  }

  if (message?.kind === "rvault:fill") {
    sendResponse({
      filled: fillDetectedForm(message.username, message.password)
    })
    return true
  }

  return false
})
