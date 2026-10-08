import { Effect } from "effect";

export const failure = Effect.fail(new Error("sentinel"));
