import { HOST_NAME, type HostRequest } from "./lib/native-client"

type NativeMessage = {
  kind: "rvault:native"
  request: HostRequest
}

chrome.runtime.onMessage.addListener((message: NativeMessage, _sender, sendResponse) => {
  if (message?.kind !== "rvault:native") {
    return false
  }

  chrome.runtime.sendNativeMessage(HOST_NAME, message.request, (response) => {
    const error = chrome.runtime.lastError
    if (error) {
      sendResponse({
        ok: false,
        error: {
          code: "storage_error",
          message: error.message || "RVault native host is unavailable."
        }
      })
      return
    }

    sendResponse(
      response ?? {
        ok: false,
        error: {
          code: "storage_error",
          message: "RVault native host returned an empty response."
        }
      }
    )
  })

  return true
})
