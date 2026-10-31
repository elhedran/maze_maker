import { LitElement, css, html } from "lit";
import { customElement, state } from "lit/decorators.js";
import { invoke } from "@tauri-apps/api/core";

/** The tool's root component. For now it only proves the frontend -> Tauri -> core path. */
@customElement("maze-tool-app")
export class MazeToolApp extends LitElement {
  @state() private coreVersion?: string;
  @state() private error?: string;

  static styles = css`
    :host {
      display: block;
      padding: 1.5rem;
      font-family: system-ui, sans-serif;
    }
    .error {
      color: #b00020;
    }
  `;

  connectedCallback(): void {
    super.connectedCallback();
    invoke<string>("core_version")
      .then((version) => (this.coreVersion = version))
      .catch((e: unknown) => (this.error = String(e)));
  }

  render() {
    let status;
    if (this.error) {
      status = html`<p class="error">Could not reach maze-core: ${this.error}</p>`;
    } else if (this.coreVersion) {
      status = html`<p>maze-core <span id="core-version">${this.coreVersion}</span></p>`;
    } else {
      status = html`<p>Connecting to maze-core…</p>`;
    }
    return html`<h1>Maze Maker — Adjacency Tool</h1>
      ${status}`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "maze-tool-app": MazeToolApp;
  }
}
