import { afterEach, describe, expect, test, vi } from "vitest";
import "./why-band.ts";
import { FACTOR_SELECT_EVENT, type FactorKey, type WhyBand } from "./why-band.ts";
import { whyFrom, type Why } from "./explore-data.ts";
import { TILES, fixtureTrace } from "./fixture.ts";
import type { CellTrace } from "../bindings/CellTrace.ts";

const CELL = { revision: 1, seed: 48213, width: 24, height: 16, cell: { x: 14, y: 2 } };
const LIST = { revision: 1, tiles: TILES, exceptions: [] };

function whyOf(edit: (t: CellTrace) => CellTrace = (t) => t): Why {
  return whyFrom(edit(fixtureTrace(CELL)), CELL.seed, LIST);
}

async function mount(why?: Why, note?: string): Promise<WhyBand> {
  const el = document.createElement("why-band");
  el.why = why;
  el.note = note;
  document.body.append(el);
  await el.updateComplete;
  return el;
}

const root = (el: Element) => el.shadowRoot!;
const squash = (s: string | null | undefined) => (s ?? "").replace(/\s+/g, " ").trim();
const cells = (tr: Element) => [...tr.querySelectorAll("td")].map((td) => squash(td.textContent));

afterEach(() => {
  document.body.innerHTML = "";
});

describe("why-band", () => {
  test("heads the band with the cell, its seed and when it was drawn", async () => {
    const el = await mount(whyOf());
    expect(squash(root(el).querySelector("#why-heading")!.textContent)).toBe(
      "Why ┐ is here · seed 48213 · col 14, row 2 · drawn 41st of 384",
    );
  });

  test("the cross names each neighbour counted, came later or edge", async () => {
    const el = await mount(whyOf());
    const word = (d: string) => squash(root(el).querySelector(`.neighbour.${d} .word`)!.textContent);
    expect([word("north"), word("east"), word("south"), word("west")]).toEqual([
      "north: counted",
      "east: edge",
      "south: came later",
      "west: counted",
    ]);
    expect(root(el).querySelector(".neighbour.east .glyph.blank")).not.toBeNull();
    expect(root(el).querySelector(".neighbour.south.later .glyph")!.textContent).toBe("└");
  });

  test("one row per candidate, with a factor column per counted neighbour, the score, chance and the drawn one", async () => {
    const el = await mount(whyOf());
    const headers = [...root(el).querySelectorAll("thead th")].map((th) => squash(th.textContent));
    expect(headers).toEqual(["Candidate", "weight", "× north ░", "× west ─", "= score", "chance", ""]);
    const rows = [...root(el).querySelectorAll("tbody tr")];
    expect(rows.map(cells)).toEqual([
      ["┐Corridor corner, open S and W", "1.2", "× 1.0", "× 1.0", "= 1.2", "70.6%", "drawn"],
      ["─Corridor straight, E–W", "1.0", "× 1.0", "× 0.5", "= 0.5", "29.4%", ""],
    ]);
    expect(rows[0].classList).toContain("drawn");
    expect(rows[1].classList).not.toContain("drawn");
    expect(squash(root(el).querySelector(".ruled-out")!.textContent)).toBe("+ 6 ruled out before the draw");
    const formula = squash(root(el).querySelector(".formula")!.textContent);
    expect(formula).toContain("chance = score ÷ 1.7, the sum of the scores.");
    expect(formula).toContain("Neighbours that came later were drawn to fit ┐");
    expect(root(el).querySelector(".nothing-settled")).toBeNull();
    expect(root(el).querySelector(".no-choice")).toBeNull();
  });

  test("with nothing settled around the cell, only tile weights counted (DR-0033 point 15)", async () => {
    const why = whyOf((t) => ({
      ...t,
      collapse: {
        ...t.collapse!,
        neighbours: t.collapse!.neighbours.map((n) => (n.state === "counted" ? { ...n, state: "later" as const } : n)) as never,
        candidates: t.collapse!.candidates.map((c) => ({ ...c, factors: [] })),
      },
    }));
    const el = await mount(why);
    expect(squash(root(el).querySelector(".nothing-settled")!.textContent)).toBe(
      "Nothing around it was settled yet, so only tile weights counted.",
    );
    expect([...root(el).querySelectorAll("thead th")].map((th) => squash(th.textContent))).toEqual([
      "Candidate", "weight", "= score", "chance", "",
    ]);
  });

  test("with one candidate left, the draw had no choice (point 16)", async () => {
    const why = whyOf((t) => ({
      ...t,
      collapse: { ...t.collapse!, candidates: [{ ...t.collapse!.candidates[0], share: 1 }], total: 1.2, ruledOut: 7 },
    }));
    const el = await mount(why);
    expect(squash(root(el).querySelector(".no-choice")!.textContent)).toBe(
      "Only one tile could fit here, so the draw had no choice.",
    );
    expect(squash(root(el).querySelector("tbody .chance")!.textContent)).toBe("100.0%");
  });

  test("a click on a factor asks for it, and the selected factor is outlined", async () => {
    const el = await mount(whyOf());
    const asked: FactorKey[] = [];
    el.addEventListener(FACTOR_SELECT_EVENT, (e) => asked.push((e as CustomEvent<FactorKey>).detail));
    const factor = () => root(el).querySelector<HTMLElement>('tbody tr[data-tile="corridor_straight#1"] td.factor[data-direction="west"]')!;

    factor().click();
    expect(asked).toEqual([{ candidate: { id: "corridor_straight", orientation: 1 }, direction: "west" }]);

    el.selected = asked[0];
    await vi.waitFor(() => expect(factor().classList).toContain("selected"));
    expect(root(el).querySelectorAll("td.factor.selected")).toHaveLength(1);
  });

  test("shows its note when there's no trace", async () => {
    const el = await mount(undefined, "The tile set was reloaded. Pick the cell again in the Maze tab.");
    expect(squash(root(el).textContent)).toBe("The tile set was reloaded. Pick the cell again in the Maze tab.");
  });

  describe("opening pressure (DR-0037 point 15)", () => {
    // ┐ got the pressure (its open sides all face settled cells); ─ didn't.
    const withPressure = (pressure: number) =>
      whyFrom(
        (() => {
          const t = fixtureTrace(CELL);
          const c = t.collapse!;
          return { ...t, collapse: { ...c, candidates: c.candidates.map((x) => ({ ...x, pressure: x.tile === 2 ? pressure : 1 })) } };
        })(),
        CELL.seed,
        { ...LIST, openingPressure: pressure },
      );

    test("below 1, a × pressure column after the neighbours: the value where it applied, a muted 1 where not", async () => {
      const el = await mount(withPressure(0.5));
      const headers = [...root(el).querySelectorAll("thead th")].map((th) => squash(th.textContent));
      expect(headers).toEqual(["Candidate", "weight", "× north ░", "× west ─", "× pressure", "= score", "chance", ""]);
      const pressureCells = [...root(el).querySelectorAll("tbody td.pressure")];
      expect(pressureCells.map((td) => squash(td.textContent))).toEqual(["× 0.5", "1"]);
      expect(pressureCells[1].classList).toContain("off");
      expect(squash(root(el).querySelector(".formula")!.textContent)).toContain(
        "× opening pressure, where the tile adds no new open side (a side past the maze's edge counts as new).",
      );
    });

    test("at 1 the column is hidden, so 2.2's band is unchanged", async () => {
      const el = await mount(withPressure(1));
      expect(root(el).querySelector("th.pressure")).toBeNull();
      expect(root(el).querySelector("td.pressure")).toBeNull();
      expect(squash(root(el).querySelector(".formula")!.textContent)).not.toContain("opening pressure");
    });
  });
});
