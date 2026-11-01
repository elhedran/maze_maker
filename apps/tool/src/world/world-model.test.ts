import { describe, expect, test } from "vitest";
import { FOLLOW_MARGIN, VIEW_HEIGHT, VIEW_WIDTH, WorldModel, centredOn, followed } from "./world-model.ts";
import { patch } from "./fixture.ts";

describe("WorldModel", () => {
  test("stores each patch's cells with the patch they came from, in order", () => {
    const model = new WorldModel();
    model.addPatch(patch(0, [[0, 0, 1], [1, 0, 2]]));
    model.addPatch(patch(1, [[2, 0, 3]]));
    expect(model.cellAt(0, 0)).toEqual({ tile: 1, patch: 0 });
    expect(model.cellAt(2, 0)).toEqual({ tile: 3, patch: 1 });
    expect(model.cellAt(5, 5)).toBeUndefined();
    expect(model.lastPatch?.index).toBe(1);
  });

  test("sealed cells are stored cells too, drawn as the tile they hold (DR-0047)", () => {
    const model = new WorldModel();
    model.addPatch(patch(0, [[0, 0, 1]], 0, [[5, 5, 7]]));
    expect(model.cellAt(5, 5)).toEqual({ tile: 7, patch: 0 });
    expect(model.cellAt(0, 0)).toEqual({ tile: 1, patch: 0 });
  });

  test("negative coordinates are their own cells", () => {
    const model = new WorldModel();
    model.addPatch(patch(0, [[-1, -2, 4], [1, 2, 5]]));
    expect(model.cellAt(-1, -2)?.tile).toBe(4);
    expect(model.cellAt(1, 2)?.tile).toBe(5);
    expect(model.cellAt(-1, 2)).toBeUndefined();
  });

  test("the frontier is looked up by anchor, and an outward endpoint wins a shared anchor", () => {
    const model = new WorldModel();
    model.setFrontier([
      { cell: { x: 0, y: 0 }, facing: "east", anchor: { x: 1, y: 0 }, outward: false },
      { cell: { x: 1, y: 1 }, facing: "north", anchor: { x: 1, y: 0 }, outward: true },
      { cell: { x: 0, y: 0 }, facing: "west", anchor: { x: -1, y: 0 }, outward: false },
    ]);
    expect(model.endpointAt(1, 0)?.facing).toBe("north");
    expect(model.endpointAt(-1, 0)?.outward).toBe(false);
    expect(model.outwardCount).toBe(1);
    model.setFrontier([]);
    expect(model.endpointAt(1, 0)).toBeUndefined();
  });
});

describe("following the player", () => {
  test("centredOn puts the player in the middle", () => {
    const c = centredOn({ x: 0, y: 0 });
    expect(c).toEqual({ x: -VIEW_WIDTH / 2, y: -VIEW_HEIGHT / 2 });
  });

  test("keeps the view while the player is well inside, re-centres near an edge", () => {
    const corner = centredOn({ x: 0, y: 0 });
    expect(followed(corner, { x: 5, y: 3 })).toBe(corner);
    const nearEast = corner.x + VIEW_WIDTH - FOLLOW_MARGIN;
    expect(followed(corner, { x: nearEast, y: 0 })).toEqual(centredOn({ x: nearEast, y: 0 }));
    const nearNorth = corner.y + FOLLOW_MARGIN - 1;
    expect(followed(corner, { x: 0, y: nearNorth })).toEqual(centredOn({ x: 0, y: nearNorth }));
    expect(followed(corner, { x: 0, y: corner.y + FOLLOW_MARGIN })).toBe(corner);
  });
});
