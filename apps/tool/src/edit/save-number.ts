// How an editor asks the app to save a number (DR-0037 point 6). The app
// owns the loaded revision and the reload path that follows a save, so the
// editor sends a request up and the app answers it with a promise in the
// event's detail. A rejection carries Rust's ToolError (`refused`, `stale`,
// `load_failed`), which the editor shows at the field (point 16).

import type { EditRecord } from "../bindings/EditRecord.ts";
import type { NumberEdit } from "../bindings/NumberEdit.ts";

export const SAVE_NUMBER_EVENT = "save-number";
/** Sent by the saved line's [See before and after]; the app shows the Maze tab (point 7). */
export const SHOW_COMPARE_EVENT = "show-compare";

/** What a successful save gives back to the editor that asked. */
export interface Saved {
  record: EditRecord;
  /** Rust's warnings, e.g. the edit record couldn't be appended (DR-0038 rule 3). */
  warnings: string[];
  /** True when a maze is shown, so there is a before and after to see. */
  canCompare: boolean;
}

export interface SaveRequest {
  edit: NumberEdit;
  /** Set by the app's listener, synchronously. */
  result?: Promise<Saved>;
}

/** Asks the app to save one number. Rejects if nothing answered. */
export function requestSave(from: HTMLElement, edit: NumberEdit): Promise<Saved> {
  const detail: SaveRequest = { edit };
  from.dispatchEvent(new CustomEvent<SaveRequest>(SAVE_NUMBER_EVENT, { detail, bubbles: true, composed: true }));
  return detail.result ?? Promise.reject(new Error("Nothing is listening for saves."));
}

export function showCompare(from: HTMLElement): void {
  from.dispatchEvent(new CustomEvent(SHOW_COMPARE_EVENT, { bubbles: true, composed: true }));
}
