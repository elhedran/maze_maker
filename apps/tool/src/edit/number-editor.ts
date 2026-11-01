import { LitElement, css, html, nothing } from "lit";
import { customElement, property, query, state } from "lit/decorators.js";
import { errorText } from "../ipc.ts";
import { themeStyles } from "../table/draw-tile.ts";
import { NOT_A_NUMBER, exactText, notSavedText, parseNumber } from "./edit-text.ts";
import { requestSave, type Saved } from "./save-number.ts";
import type { NumberTarget } from "../pending-bindings.ts";

/** Sent when the editor closes without saving (Cancel or Esc). */
export const EDITOR_CLOSE_EVENT = "editor-close";
/** Sent with the `Saved` result once Rust has written and reloaded the file. */
export const NUMBER_SAVED_EVENT = "number-saved";

/**
 * The in-place editor for one stored number (DR-0037 points 3 to 6 and 16):
 * the field, prefilled with Rust's exact value, what a change reaches, and
 * [Save to core.toml] / [Cancel]. The host opens and closes it; one per host.
 */
@customElement("number-editor")
export class NumberEditor extends LitElement {
  @property({ attribute: false }) target?: NumberTarget;
  /** The current value, exactly as Rust sent it. */
  @property({ type: Number }) value = 0;
  /** "Corridor corner weight". */
  @property() label = "";
  /** Point 4's reach line. */
  @property() reach = "";
  /** Extra guidance under the field, e.g. the pressure's range (point 14). */
  @property() hint = "";

  @state() private text = "";
  @state() private saving = false;
  /** Point 16: the loader's reason or Rust's error, shown at the field. */
  @state() private problem?: string;

  @query("input") private input?: HTMLInputElement;

  static styles = [
    themeStyles,
    css`
      :host {
        display: block;
        margin: 0.25rem 0 0.4rem 1.2em;
        padding: 0.5rem 0.6rem;
        border: 1px solid var(--frame);
        font-family: system-ui, sans-serif;
        font-size: 14px;
      }
      label { display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
      input {
        font: inherit;
        font-family: var(--mono);
        width: 7rem;
        padding: 0.2rem 0.4rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
      }
      p { margin: 0.3rem 0; }
      .reach, .hint { color: var(--muted); }
      .problem { color: var(--problem); white-space: pre-wrap; }
      .buttons { display: flex; gap: 0.5rem; margin-top: 0.4rem; }
      button {
        font: inherit;
        padding: 0.2rem 0.6rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
        border-radius: 3px;
        cursor: pointer;
      }
      button:disabled { cursor: default; opacity: 0.6; }
    `,
  ];

  connectedCallback(): void {
    super.connectedCallback();
    this.text = exactText(this.value);
  }

  protected firstUpdated(): void {
    this.input?.focus();
    this.input?.select();
  }

  /** The typed number, if it is one and differs from the current value (point 5). */
  private get newValue(): number | undefined {
    const v = parseNumber(this.text);
    return v !== undefined && v !== this.value ? v : undefined;
  }

  private async save() {
    const value = this.newValue;
    if (value === undefined || this.saving || !this.target) return;
    this.saving = true;
    this.problem = undefined;
    let saved: Saved;
    try {
      saved = await requestSave(this, { target: this.target, value });
    } catch (e) {
      // Point 16: stays open with what Ian typed; nothing else moves.
      const messages = (e as { messages?: unknown } | null)?.messages;
      this.problem = notSavedText(Array.isArray(messages) ? messages.map(String) : [errorText(e)]);
      this.saving = false;
      await this.updateComplete;
      this.input?.focus();
      return;
    }
    this.saving = false;
    this.dispatchEvent(new CustomEvent<Saved>(NUMBER_SAVED_EVENT, { detail: saved, bubbles: true, composed: true }));
  }

  private cancel() {
    this.dispatchEvent(new CustomEvent(EDITOR_CLOSE_EVENT, { bubbles: true, composed: true }));
  }

  private onKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      void this.save();
    } else if (event.key === "Escape") {
      event.preventDefault();
      this.cancel();
    }
  }

  render() {
    const notANumber = parseNumber(this.text) === undefined;
    return html`
      <label
        >${this.label}
        <input
          inputmode="decimal"
          autocomplete="off"
          .value=${this.text}
          aria-invalid=${notANumber ? "true" : "false"}
          ?disabled=${this.saving}
          @input=${(e: Event) => {
            this.text = (e.target as HTMLInputElement).value;
            this.problem = undefined;
          }}
          @keydown=${this.onKeydown}
      /></label>
      ${notANumber ? html`<p class="problem not-a-number">${NOT_A_NUMBER}</p>` : nothing}
      ${this.problem ? html`<p class="problem refused" role="alert">${this.problem}</p>` : nothing}
      <p class="reach">${this.reach}</p>
      ${this.hint ? html`<p class="hint">${this.hint}</p>` : nothing}
      <div class="buttons">
        <button class="save" ?disabled=${this.newValue === undefined || this.saving} @click=${() => this.save()}>
          ${this.saving ? "Saving…" : "Save to core.toml"}
        </button>
        <button class="cancel" ?disabled=${this.saving} @click=${() => this.cancel()}>Cancel</button>
      </div>
    `;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "number-editor": NumberEditor;
  }
}
