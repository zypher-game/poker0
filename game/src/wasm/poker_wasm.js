import * as wasm from "./poker_wasm_bg.wasm";
import { __wbg_set_wasm } from "./poker_wasm_bg.js";
__wbg_set_wasm(wasm);
export * from "./poker_wasm_bg.js";
