import { LitElement, css, html, nothing } from "lit";
import { customElement, query, state } from "lit/decorators.js";
import { coreVersion, errorText, reloadTileSet, tableView, tilesOnlyRequest } from "./ipc.ts";
import { formatLoadTime } from "./table/format.ts";
import { themeStyles } from "./table/draw-tile.ts";
import { TOOL_ERROR_EVENT } from "./tool-error.ts";
import "./table/adjacency-table.ts";
import "./maze/maze-view.ts";
import type { AdjacencyTable } from "./table/adjacency-table.ts";
import type { MazeView } from "./maze/maze-view.ts";

type Tab = "table" | "maze";

const TABS: { id: Tab; label: string }[] = [
  { id: "table", label: "Table" },
  { id: "maze", label: "Maze" },
];

/**
 * The tool's root component: the title and core version, the tile set bar
 * both views share, and the Table and Maze tabs (DR-0029 points 1 and 2).
 */
@customElement("maze-tool-app")
export class MazeToolApp extends LitElement {
  @state() private coreVersion?: string;
  @state() private coreError?: string;
  @state() private tileSet?: { path: string; loadedAt: string };
  /** Rust's error, as a red banner. The views keep their last good state (DR-0025 point 12). */
  @state() private error?: string;
  @state() private reloading = false;
  @state() private tab: Tab = "table";

  @query("adjacency-table") private table?: AdjacencyTable;
  @query("maze-view") private mazeView?: MazeView;

  static styles = [
    themeStyles,
    css`
      :host {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        padding: 1rem 1.5rem;
        font-family: system-ui, sans-serif;
        font-size: 14px;
      }
      header { display: flex; align-items: baseline; gap: 1rem; }
      h1 { font-size: 1.25rem; margin: 0 0 0.25rem; }
      .status { color: var(--muted); font-size: 0.85rem; }
      .status.error { color: var(--problem); }
      .bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 1rem; }
      .path { font-family: var(--mono); }
      .muted { color: var(--muted); }
      button {
        font: inherit;
        padding: 0.25rem 0.75rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
        border-radius: 3px;
        cursor: pointer;
      }
      .banner {
        color: var(--problem);
        background: var(--problem-bg);
        border: 1px solid var(--problem);
        padding: 0.5rem 0.75rem;
        white-space: pre-wrap;
        margin: 0;
        font: inherit;
      }
      [role="tablist"] { display: flex; gap: 0.25rem; border-bottom: 1px solid var(--frame); }
      [role="tab"] { border-bottom: none; border-radius: 3px 3px 0 0; }
      [role="tab"][aria-selected="true"] { background: var(--text); color: var(--page); font-weight: 600; }
    `,
  ];

  connectedCallback(): void {
    super.connectedCallback();
    this.addEventListener(TOOL_ERROR_EVENT, this.onToolError);
    coreVersion()
      .then((version) => (this.coreVersion = version))
      .catch((e: unknown) => (this.coreError = errorText(e)));
    // Only the load details for the bar; each view fetches what it shows.
    tableView(tilesOnlyRequest())
      .then((view) => (this.tileSet = { path: view.tileSetPath, loadedAt: view.loadedAt }))
      .catch((e: unknown) => (this.error = errorText(e)));
  }

  disconnectedCallback(): void {
    this.removeEventListener(TOOL_ERROR_EVENT, this.onToolError);
    super.disconnectedCallback();
  }

  private onToolError = (event: Event) => {
    this.error = (event as CustomEvent<string>).detail;
  };

  /**
   * Re-reads the tile set file. On success the table refetches and a shown
   * maze regenerates from its seed (DR-0029 point 3); on bad data the banner
   * shows Rust's error and both views keep what they show.
   */
  private async reload() {
    this.reloading = true;
    try {
      const result = await reloadTileSet();
      this.tileSet = { path: result.tileSetPath, loadedAt: result.loadedAt };
      this.error = undefined;
    } catch (e) {
      this.error = errorText(e);
      return;
    } finally {
      this.reloading = false;
    }
    this.table?.refresh();
    this.mazeView?.regenerateAfterReload();
  }

  render() {
    return html`
      <header><h1>Maze Maker — Adjacency Tool</h1>${this.renderCoreStatus()}</header>
      <div class="bar">
        <span>Tile set: <span class="path">${this.tileSet?.path ?? "—"}</span></span>
        ${this.tileSet ? html`<span class="muted">loaded ${formatLoadTime(this.tileSet.loadedAt)}</span>` : nothing}
        <button id="reload" @click=${() => this.reload()} ?disabled=${this.reloading}>Reload tile set</button>
      </div>
      ${this.error ? html`<pre class="banner" role="alert">${this.error}</pre>` : nothing}
      <div role="tablist" aria-label="Views">
        ${TABS.map(
          (t) => html`<button
            role="tab"
            id="tab-${t.id}"
            aria-selected=${t.id === this.tab ? "true" : "false"}
            aria-controls="panel-${t.id}"
            @click=${() => (this.tab = t.id)}
          >
            ${t.label}
          </button>`,
        )}
      </div>
      <!-- Both views stay mounted and only hide, so each keeps its state. -->
      <div role="tabpanel" id="panel-table" aria-labelledby="tab-table" ?hidden=${this.tab !== "table"}>
        <adjacency-table></adjacency-table>
      </div>
      <div role="tabpanel" id="panel-maze" aria-labelledby="tab-maze" ?hidden=${this.tab !== "maze"}>
        <maze-view></maze-view>
      </div>
    `;
  }

  private renderCoreStatus() {
    if (this.coreError) {
      return html`<span class="status error">Could not reach maze-core: ${this.coreError}</span>`;
    }
    if (this.coreVersion) {
      return html`<span class="status">maze-core <span id="core-version">${this.coreVersion}</span></span>`;
    }
    return html`<span class="status">Connecting to maze-core…</span>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "maze-tool-app": MazeToolApp;
  }
}
