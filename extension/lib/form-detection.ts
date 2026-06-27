export type FieldDescriptor = {
  index: number
  type: string
  name?: string
  id?: string
  autocomplete?: string
  placeholder?: string
  ariaLabel?: string
  label?: string
}

export type LoginFields = {
  usernameIndex: number
  passwordIndex: number
}

export function detectLoginFields(fields: FieldDescriptor[]): LoginFields | null {
  const password = fields.find((field) => field.type.toLowerCase() === "password")
  if (!password) {
    return null
  }

  const candidates = fields.filter((field) => {
    if (field.index === password.index) {
      return false
    }
    const type = field.type.toLowerCase()
    return type === "email" || type === "text" || type === "tel" || type === "url"
  })

  const username = candidates
    .map((field) => ({ field, score: scoreUsernameField(field, password.index) }))
    .sort((a, b) => b.score - a.score || a.field.index - b.field.index)[0]

  if (!username || username.score <= 0) {
    return null
  }

  return {
    usernameIndex: username.field.index,
    passwordIndex: password.index
  }
}

export function getPlatformSuggestion(url: string): string {
  try {
    const hostname = new URL(url).hostname.toLowerCase()
    return hostname.startsWith("www.") ? hostname.slice(4) : hostname
  } catch {
    return ""
  }
}

function scoreUsernameField(field: FieldDescriptor, passwordIndex: number): number {
  const haystack = [
    field.type,
    field.name,
    field.id,
    field.autocomplete,
    field.placeholder,
    field.ariaLabel,
    field.label
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase()

  let score = 1
  if (field.type.toLowerCase() === "email") {
    score += 5
  }
  if (haystack.includes("username") || haystack.includes("login")) {
    score += 6
  }
  if (haystack.includes("email")) {
    score += 5
  }
  if (field.autocomplete?.toLowerCase() === "username") {
    score += 10
  }
  if (field.autocomplete?.toLowerCase() === "email") {
    score += 7
  }
  if (haystack.includes("user id") || haystack.includes("userid")) {
    score += 6
  }
  if (field.index < passwordIndex) {
    score += 2
  } else {
    score -= 2
  }
  if (haystack.includes("search")) {
    score -= 8
  }

  return score
}
