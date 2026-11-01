import { afterEach, describe, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { COMMANDS } from "../ipc.ts";
import "../maze-tool-app.ts";
import type { MazeToolApp } from "../maze-tool-app.ts";
import { fixtureTableView } from "../explore/fixture.ts";
import type { TableViewRequest } from "../bindings/TableViewRequest.ts";
import { WORLD_COMMANDS } from "./world-ipc.ts";
import type { WorldStarted } from "./world-types.ts";
import { TILES, patch } from "./fixture.ts";

// The World tab inside the app: it mounts, and the shared Reload and banner
// reach it. The view itself is tested in world-view.test.ts.

function backend(walkFails = false) {
  mockIPC((cmd, payload) => {
    const request = (payload as { request?: unknown } | undefined)?.request;
    switch (cmd) {
      case COMMANDS.coreVersion:
        return "9.9.9";
      case COMMANDS.tableView:
        return fixtureTableView(request as TableViewRequest);
      case COMMANDS.reloadTileSet:
        return { revision: 2, tileSetPath: "data/tile-sets/core.toml", loadedAt: "2026-11-01T09:30:00Z" };
      case WORLD_COMMANDS.worldStart: {
        const started: WorldStarted = {
          worldId: 1,
          seed: (request as { seed: number }).seed,
          revision: 1,
          loadedAt: "2026-11-01T08:00:00Z",
          tiles: TILES,
          update: {
            worldId: 1,
            player: { x: 0, y: 0 },
            path: [],
            patches: [patch(0, [[0, 0, 2]])],
            frontier: [],
            bounds: { x: 0, y: 0, width: 1, height: 1 },
            steps: 0,
            patchCount: 1,
            storedCount: 1,
          },
        };
        return started;
      }
      case WORLD_COMMANDS.worldWalk:
        if (walkFails) throw { kind: "bad_request", messages: ["A world takes at most 1,000,000 steps."] };
    }
    throw new Error(`unexpected command ${cmd}`);
  });
}

function mount(): MazeToolApp {
  const el = document.createElement("maze-tool-app");
  document.body.append(el);
  return el;
}

const world = (app: MazeToolApp) => app.shadowRoot!.querySelector("world-view")!;
const inWorld = (app: MazeToolApp, selector: string) => world(app).shadowRoot!.querySelector<HTMLElement>(selector);

afterEach(() => {
  document.body.innerHTML = "";
  clearMocks();
});

describe("the World tab", () => {
  test("is last, hidden until chosen, and shows the world view", async () => {
    backend();
    const app = mount();
    await vi.waitFor(() => expect(app.shadowRoot!.querySelector("#tab-world")).not.toBeNull());
    const panel = app.shadowRoot!.querySelector<HTMLElement>("#panel-world")!;
    expect(panel.hidden).toBe(true);
    app.shadowRoot!.querySelector<HTMLButtonElement>("#tab-world")!.click();
    await app.updateComplete;
    expect(panel.hidden).toBe(false);
    expect(world(app)).not.toBeNull();
  });

  test("a good Reload keeps the world and tells it which set it was grown with", async () => {
    backend();
    const app = mount();
    await vi.waitFor(() => expect(inWorld(app, "#new-world")).not.toBeNull());
    inWorld(app, "#new-world")!.click();
    await vi.waitFor(() => expect(inWorld(app, "#status")?.textContent).toContain("1 patch · 1 cells"));
    app.shadowRoot!.querySelector<HTMLButtonElement>("#reload")!.click();
    await vi.waitFor(() => expect(inWorld(app, "#status")?.textContent).toContain("New world uses the reloaded one"));
    expect(inWorld(app, '.cell[data-x="0"][data-y="0"]')?.getAttribute("aria-label")).toContain("Corridor crossroads");
  });

  test("a failed world command shows in the shared banner", async () => {
    backend(true);
    const app = mount();
    await vi.waitFor(() => expect(inWorld(app, "#new-world")).not.toBeNull());
    inWorld(app, "#new-world")!.click();
    await vi.waitFor(() => expect(inWorld(app, "#status")).not.toBeNull());
    inWorld(app, "#step")!.click();
    await vi.waitFor(() =>
      expect(app.shadowRoot!.querySelector(".banner")?.textContent).toBe("A world takes at most 1,000,000 steps."),
    );
  });
});
