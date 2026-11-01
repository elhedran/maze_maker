import { LitElement, css, html, nothing } from "lit";
import { customElement, query, state } from "lit/decorators.js";
import { coreVersion, errorText, reloadTileSet, tableView, tilesOnlyRequest } from "./ipc.ts";
import { fileName, formatLoadTime } from "./table/format.ts";
import { themeStyles } from "./table/draw-tile.ts";
import { TOOL_ERROR_EVENT } from "./tool-error.ts";
import { BACK_TO_MAZE_EVENT, OPEN_EXPLORE_EVENT, type ExploreTarget } from "./explore/open-explore.ts";
import "./table/adjacency-table.ts";
import "./explore/explore-view.ts";
import "./maze/maze-view.ts";
import type { AdjacencyTable } from "./table/adjacency-table.ts";
import type { ExploreView } from "./explore/explore-view.ts";
import type { MazeView } from "./maze/maze-view.ts";
// Slice 2.3, saving a number (DR-0037 point 6, DR-0038).
import { editNumbers } from "./ipc.ts";
import { SAVE_NUMBER_EVENT, SHOW_COMPARE_EVENT, type SaveRequest, type Saved } from "./edit/save-number.ts";
import type { NumberEdit } from "./bindings/NumberEdit.ts";

type Tab = "table" | "explore" | "maze";

// Explore sits next to Table because it reads the same table (DR-0026 point 1).
const TABS: { id: Tab; label: string }[] = [
  { id: "table", label: "Table" },
  { id: "explore", label: "Explore" },
  { id: "maze", label: "Maze" },
];

/**
 * The tool's root component: the title and core version, the tile set bar
 * the views share, and the Table, Explore and Maze tabs (DR-0029 points 1
 * and 2, DR-0026 point 1).
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
  /** The loaded tile set's revision; a save names it so Rust can refuse a stale one (DR-0038 rule 3). */
  private revision?: number;

  @query("adjacency-table") private table?: AdjacencyTable;
  @query("explore-view") private explore?: ExploreView;
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
      .path { font-family: var(--mono); white-space: nowrap; }
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
    this.addEventListener(OPEN_EXPLORE_EVENT, this.onOpenExplore);
    this.addEventListener(BACK_TO_MAZE_EVENT, this.onBackToMaze);
    this.addEventListener(SAVE_NUMBER_EVENT, this.onSaveNumber);
    this.addEventListener(SHOW_COMPARE_EVENT, this.onShowCompare);
    coreVersion()
      .then((version) => (this.coreVersion = version))
      .catch((e: unknown) => (this.coreError = errorText(e)));
    // Only the load details for the bar; each view fetches what it shows.
    tableView(tilesOnlyRequest())
      .then((view) => {
        this.tileSet = { path: view.tileSetPath, loadedAt: view.loadedAt };
        this.revision = view.revision;
      })
      .catch((e: unknown) => (this.error = errorText(e)));
  }

  disconnectedCallback(): void {
    this.removeEventListener(TOOL_ERROR_EVENT, this.onToolError);
    this.removeEventListener(OPEN_EXPLORE_EVENT, this.onOpenExplore);
    this.removeEventListener(BACK_TO_MAZE_EVENT, this.onBackToMaze);
    this.removeEventListener(SAVE_NUMBER_EVENT, this.onSaveNumber);
    this.removeEventListener(SHOW_COMPARE_EVENT, this.onShowCompare);
    super.disconnectedCallback();
  }

  private onToolError = (event: Event) => {
    this.error = (event as CustomEvent<string>).detail;
  };

  /** DR-0026 point 10: any view can open the explore view on a tile; the maze does with its cell (DR-0033). */
  private onOpenExplore = (event: Event) => {
    this.tab = "explore";
    this.explore?.open((event as CustomEvent<ExploreTarget>).detail);
  };

  /** DR-0033 point 8: "< Back to maze" shows the Maze tab, still with its maze and selection. */
  private onBackToMaze = () => {
    this.tab = "maze";
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
      this.revision = result.revision;
      this.error = undefined;
    } catch (e) {
      this.error = errorText(e);
      return;
    } finally {
      this.reloading = false;
    }
    this.afterLoad();
  }

  /** After any successful load, Reload or a save: every view refetches where it is (DR-0029 point 3, DR-0037 point 6). */
  private afterLoad() {
    this.table?.refresh();
    this.explore?.refresh(this.revision);
    this.mazeView?.showChange();
  }

  // ---- Slice 2.3: saving a number (DR-0037 points 6 and 7, DR-0038) ----

  /** An editor asks to save; the answer goes back in the event (see `requestSave`). */
  private onSaveNumber = (event: Event) => {
    const request = (event as CustomEvent<SaveRequest>).detail;
    request.result = this.saveNumber(request.edit);
  };

  /** DR-0037 point 7: [See before and after] shows the Maze tab. */
  private onShowCompare = () => {
    this.tab = "maze";
  };

  /**
   * Rust writes core.toml and reloads it; then the same path as Reload. A
   * rejection (`refused`, `stale`, a failed write) goes back to the editor,
   * which shows it at the field, not in the banner (DR-0037 point 16).
   */
  private async saveNumber(edit: NumberEdit): Promise<Saved> {
    if (this.revision === undefined) throw new Error("No tile set is loaded. Press Reload tile set.");
    const result = await editNumbers({ revision: this.revision, edits: [edit] });
    this.tileSet = { path: result.reload.tileSetPath, loadedAt: result.reload.loadedAt };
    this.revision = result.reload.revision;
    this.error = undefined;
    const record = result.edits[0];
    if (!record) throw new Error("edit_numbers saved nothing.");
    const canCompare = this.mazeView?.hasMaze ?? false;
    this.afterLoad();
    return { record, warnings: result.warnings, canCompare };
  }

  // ---- end of slice 2.3's saving ----

  render() {
    return html`
      <header><h1>Maze Maker — Adjacency Tool</h1>${this.renderCoreStatus()}</header>
      <div class="bar">
        <!-- The file name only, so it never wraps mid-word; the full path on hover (DR-0029 review). -->
        <span>Tile set: <span class="path" title=${this.tileSet?.path ?? ""}>${this.tileSet ? fileName(this.tileSet.path) : "—"}</span></span>
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
      <!-- The views stay mounted and only hide, so each keeps its state. -->
      <div role="tabpanel" id="panel-table" aria-labelledby="tab-table" ?hidden=${this.tab !== "table"}>
        <adjacency-table></adjacency-table>
      </div>
      <div role="tabpanel" id="panel-explore" aria-labelledby="tab-explore" ?hidden=${this.tab !== "explore"}>
        <explore-view></explore-view>
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
