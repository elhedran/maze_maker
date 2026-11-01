import { afterEach, describe, expect, test, vi } from "vitest";
import "./number-editor.ts";
import { EDITOR_CLOSE_EVENT, NUMBER_SAVED_EVENT, type NumberEditor } from "./number-editor.ts";
import { SAVE_NUMBER_EVENT, type SaveRequest, type Saved } from "./save-number.ts";

const TARGET = { kind: "tileWeight" as const, tileId: "corridor_corner" };
const RECORD = { at: "2026-11-01T20:31:00Z", target: TARGET, number: "tile.corridor_corner.weight", from: 3, to: 2 };

async function mount(value = 3): Promise<NumberEditor> {
  const el = document.createElement("number-editor");
  el.target = TARGET;
  el.value = value;
  el.label = "Corridor corner weight";
  el.reach = "Every Corridor corner, in all four orientations and every direction.";
  document.body.append(el);
  await el.updateComplete;
  return el;
}

const $ = <T extends Element>(el: NumberEditor, sel: string) => el.shadowRoot!.querySelector<T>(sel);
const input = (el: NumberEditor) => $<HTMLInputElement>(el, "input")!;
const saveButton = (el: NumberEditor) => $<HTMLButtonElement>(el, "button.save")!;

async function type(el: NumberEditor, text: string) {
  input(el).value = text;
  input(el).dispatchEvent(new Event("input"));
  await el.updateComplete;
}

/** Answers saves the way the app does, with `answer`. */
function answerSaves(answer: () => Promise<Saved>) {
  const requests: SaveRequest[] = [];
  const listener = (e: Event) => {
    const detail = (e as CustomEvent<SaveRequest>).detail;
    requests.push(detail);
    detail.result = answer();
  };
  document.body.addEventListener(SAVE_NUMBER_EVENT, listener);
  return { requests, stop: () => document.body.removeEventListener(SAVE_NUMBER_EVENT, listener) };
}

afterEach(() => {
  document.body.innerHTML = "";
});

describe("number-editor", () => {
  test("opens prefilled with the exact value, focused, with what it reaches", async () => {
    const el = await mount(1.0749);
    expect(input(el).value).toBe("1.0749");
    expect(el.shadowRoot!.activeElement).toBe(input(el));
    expect($(el, ".reach")!.textContent).toContain("in all four orientations");
  });

  test("Save is enabled only for a number different from the current one (point 5)", async () => {
    const el = await mount();
    expect(saveButton(el).disabled).toBe(true);
    await type(el, "abc");
    expect(saveButton(el).disabled).toBe(true);
    expect($(el, ".not-a-number")!.textContent).toBe("A number, like 2.0 or 0.5.");
    await type(el, "3");
    expect(saveButton(el).disabled).toBe(true);
    await type(el, "-1");
    expect(saveButton(el).disabled).toBe(false);
    expect($(el, ".not-a-number")).toBeNull();
  });

  test("Enter saves through the app and reports the saved record", async () => {
    let release!: (s: Saved) => void;
    const saves = answerSaves(() => new Promise<Saved>((r) => (release = r)));
    const el = await mount();
    const saved = vi.fn();
    el.addEventListener(NUMBER_SAVED_EVENT, (e) => saved((e as CustomEvent<Saved>).detail));
    await type(el, "2.0");
    input(el).dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
    await el.updateComplete;
    expect(saves.requests.map((r) => r.edit)).toEqual([{ target: TARGET, value: 2 }]);
    expect(saveButton(el).textContent!.trim()).toBe("Saving…");
    expect(saveButton(el).disabled).toBe(true);
    release({ record: RECORD, warnings: [], canCompare: true });
    await vi.waitFor(() => expect(saved).toHaveBeenCalledWith({ record: RECORD, warnings: [], canCompare: true }));
    saves.stop();
  });

  test("a refused edit stays at the field with the loader's reason (point 16)", async () => {
    const saves = answerSaves(() => Promise.reject({ kind: "refused", messages: ["weight is -1; it must be at least 0"] }));
    const el = await mount();
    const saved = vi.fn();
    el.addEventListener(NUMBER_SAVED_EVENT, saved);
    await type(el, "-1");
    saveButton(el).click();
    await vi.waitFor(() =>
      expect($(el, ".refused")?.textContent).toBe("Not saved: weight is -1; it must be at least 0. core.toml is unchanged."),
    );
    expect(input(el).value).toBe("-1");
    expect(saved).not.toHaveBeenCalled();
    await vi.waitFor(() => expect(el.shadowRoot!.activeElement).toBe(input(el)));
    // Typing clears the message.
    await type(el, "-2");
    expect($(el, ".refused")).toBeNull();
    saves.stop();
  });

  test("a stale save shows Rust's message the same way", async () => {
    const saves = answerSaves(() =>
      Promise.reject({ kind: "stale", messages: ["core.toml changed on disk. Press Reload tile set first."] }),
    );
    const el = await mount();
    await type(el, "2");
    saveButton(el).click();
    await vi.waitFor(() => expect($(el, ".refused")?.textContent).toContain("Press Reload tile set first. core.toml is unchanged."));
    saves.stop();
  });

  test("Esc and Cancel close without saving", async () => {
    const el = await mount();
    const closed = vi.fn();
    el.addEventListener(EDITOR_CLOSE_EVENT, closed);
    input(el).dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    $<HTMLButtonElement>(el, "button.cancel")!.click();
    expect(closed).toHaveBeenCalledTimes(2);
  });
});
