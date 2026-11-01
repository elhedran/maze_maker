// Hand-written stand-in for the ts-rs type Rowan generates into src/bindings/ (DR-0023).
// Same file and type name, so the swap is the import path in src/ipc.ts; then delete this folder.
export type TileKind = "corridor" | "room" | "hedge";
