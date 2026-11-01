/// <reference types="vite/client" />
// The frontend's half of the IPC contract (DR-0023): every command name in
// COMMANDS (and the World tab's WORLD_COMMANDS) is one the Rust side registers, and the commands that take a
// request get it under the argument name Rust declares, `request`.

import { afterEach, describe, expect, test } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { COMMANDS, compareMazes, editNumbers } from "./ipc.ts";
import { WORLD_COMMANDS } from "./world/world-ipc.ts";

// lib.rs as text, through Vite's ?raw import, so no Node types are needed.
import LIB_RS from "../src-tauri/src/lib.rs?raw";

afterEach(() => clearMocks());

describe("ipc contract", () => {
  test("COMMANDS are exactly the commands lib.rs registers", () => {
    const lib = LIB_RS;
    const handler = lib.match(/generate_handler!\[([^\]]*)\]/);
    expect(handler).not.toBeNull();
    // The World tab's commands live in world.rs, registered as `world::name`.
    const registered = handler![1]
      .split(",")
      .map((s) => s.trim().replace(/^world::/, ""))
      .filter((s) => s !== "");
    const known = [...Object.values(COMMANDS), ...Object.values(WORLD_COMMANDS)];
    expect(known.sort()).toEqual(registered.sort());
  });

  test("edit_numbers and compare_mazes take their request as `request`", async () => {
    const lib = LIB_RS;
    expect(lib).toMatch(/fn edit_numbers\([^)]*request: ipc::EditRequest/s);
    expect(lib).toMatch(/fn compare_mazes\([^)]*request: ipc::CompareRequest/s);

    const calls: { cmd: string; payload: unknown }[] = [];
    mockIPC((cmd, payload) => {
      calls.push({ cmd, payload });
      return {};
    });
    const edit = { revision: 3, edits: [{ target: { kind: "openingPressure" as const }, value: 0.5 }] };
    await editNumbers(edit);
    await compareMazes({ seed: 1, width: 24, height: 16 });
    expect(calls).toEqual([
      { cmd: "edit_numbers", payload: { request: edit } },
      { cmd: "compare_mazes", payload: { request: { seed: 1, width: 24, height: 16 } } },
    ]);
  });
});
