import { LitElement, css, html } from "lit";
import { customElement, state } from "lit/decorators.js";
import { coreVersion, errorText } from "./ipc.ts";
import "./table/adjacency-table.ts";

/** The tool's root component: the title, the core version and the table view (slice 1.1). */
@customElement("maze-tool-app")
export class MazeToolApp extends LitElement {
  @state() private coreVersion?: string;
  @state() private error?: string;

  static styles = css`
    :host {
      display: block;
      padding: 1rem 1.5rem;
      font-family: system-ui, sans-serif;
    }
    header {
      display: flex;
      align-items: baseline;
      gap: 1rem;
    }
    h1 {
      font-size: 1.25rem;
      margin: 0 0 0.75rem;
    }
    .status {
      color: #6d737d;
      font-size: 0.85rem;
    }
    .error {
      color: #b00020;
    }
  `;

  connectedCallback(): void {
    super.connectedCallback();
    coreVersion()
      .then((version) => (this.coreVersion = version))
      .catch((e: unknown) => (this.error = errorText(e)));
  }

  render() {
    let status;
    if (this.error) {
      status = html`<span class="status error">Could not reach maze-core: ${this.error}</span>`;
    } else if (this.coreVersion) {
      status = html`<span class="status">maze-core <span id="core-version">${this.coreVersion}</span></span>`;
    } else {
      status = html`<span class="status">Connecting to maze-core…</span>`;
    }
    return html`<header><h1>Maze Maker — Adjacency Tool</h1>${status}</header>
      <adjacency-table></adjacency-table>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "maze-tool-app": MazeToolApp;
  }
}
