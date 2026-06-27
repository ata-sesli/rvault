export function logoSrcFromDataBase64Import(value: string): string {
  if (value.startsWith("data:")) {
    return value
  }

  return `data:image/png;base64,${value}`
}
