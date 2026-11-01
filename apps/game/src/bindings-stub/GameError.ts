// Hand-written stand-in for the ts-rs type Rowan generates into src/bindings/ (DR-0023).
// Same file and type name, so the swap is the import path in src/ipc.ts; then delete this folder.
/** Load errors, one per line, or "no game started" (DR-0043 point 7). */
export type GameError = { message: string };
