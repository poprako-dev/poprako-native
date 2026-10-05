export function isKeyboardComposing(event: KeyboardEvent): boolean {
  if (event.isComposing) return true;
  // WKWebView can report 229 for ordinary Option+digit shortcuts too.
  if (event.altKey && /^Digit\d$/u.test(event.code)) return false;
  // eslint-disable-next-line @typescript-eslint/no-deprecated -- IME confirmation compatibility.
  return event.keyCode === 229;
}
