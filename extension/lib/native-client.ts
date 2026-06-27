export const HOST_NAME = "io.github.ata_sesli.rvault"

export type HostRequest =
  | { type: "status" }
  | { type: "unlock"; masterPassword: string }
  | { type: "lock" }
  | { type: "quit" }
  | { type: "list"; query?: string; vault?: string }
  | { type: "get"; platform: string; userId: string; vault?: string }
  | { type: "create"; platform: string; userId: string; password: string; vault?: string }
  | {
      type: "update"
      platform: string
      oldUserId: string
      newUserId: string
      password: string
      vault?: string
    }
  | { type: "delete"; platform: string; userId: string; vault?: string }
  | { type: "generate"; length: number; specialCharacters: boolean }

export type HostErrorCode =
  | "locked"
  | "setup_required"
  | "unlock_failed"
  | "not_found"
  | "invalid_request"
  | "storage_error"

export type HostResponse<T> =
  | { ok: true; data: T }
  | { ok: false; error: { code: HostErrorCode; message: string } }

export class HostApiError extends Error {
  code: HostErrorCode

  constructor(code: HostErrorCode, message: string) {
    super(message)
    this.name = "HostApiError"
    this.code = code
  }
}

export function createHostRequest<T extends HostRequest["type"]>(
  type: T,
  payload: Omit<Extract<HostRequest, { type: T }>, "type">
): Extract<HostRequest, { type: T }> {
  return { type, ...payload } as Extract<HostRequest, { type: T }>
}

export function parseHostResponse<T>(response: HostResponse<T>): T {
  if (response.ok) {
    return response.data
  }
  throw new HostApiError(response.error.code, response.error.message)
}

export async function sendHostRequest<T>(request: HostRequest): Promise<T> {
  const response = await chrome.runtime.sendMessage({
    kind: "rvault:native",
    request
  })

  return parseHostResponse(response as HostResponse<T>)
}
