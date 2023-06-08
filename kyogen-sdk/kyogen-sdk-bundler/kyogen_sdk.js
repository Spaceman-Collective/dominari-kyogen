import * as wasm from "./kyogen_sdk_bg.wasm";
import { __wbg_set_wasm } from "./kyogen_sdk_bg.js";
__wbg_set_wasm(wasm);
export * from "./kyogen_sdk_bg.js";
