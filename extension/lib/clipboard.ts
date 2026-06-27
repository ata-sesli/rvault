type WritableClipboard = {
  writeText: (text: string) => Promise<void>
}

export async function copyTextToClipboard(
  text: string,
  clipboard: WritableClipboard | undefined = globalThis.navigator?.clipboard
): Promise<void> {
  if (!clipboard) {
    throw new Error("Clipboard is unavailable.")
  }

  await clipboard.writeText(text)
}
