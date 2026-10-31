// How a view tells the app a command failed, so the error shows in the one
// red banner above the tabs (DR-0029 point 2) rather than inside each view.

export const TOOL_ERROR_EVENT = "tool-error";

/** Sends `message` up to the app's banner. The view keeps its last good state. */
export function reportToolError(from: HTMLElement, message: string): void {
  from.dispatchEvent(new CustomEvent<string>(TOOL_ERROR_EVENT, { detail: message, bubbles: true, composed: true }));
}
