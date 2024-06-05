/* tslint:disable */
/* eslint-disable */
/**
* generate keypair
* @returns {any}
*/
export function generate_key(): any;
/**
* generate keypair
* @param {string} seed
* @returns {any}
*/
export function generate_key_by_seed(seed: string): any;
/**
* compute masked to revealed card and the revealed proof
* @param {string} sk
* @param {any} card
* @returns {any}
*/
export function reveal_card(sk: string, card: any): any;
/**
* compute masked to revealed card and the revealed proof
* @param {string} sk
* @param {any} card
* @returns {any}
*/
export function batch_reveal_card(sk: string, card: any): any;
/**
* unmask the card use all reveals
* @param {any} card
* @param {any} reveals
* @returns {number}
*/
export function unmask_card(card: any, reveals: any): number;
/**
* batch unmask the card use all reveals
* @param {any} card
* @param {any} reveals
* @returns {any}
*/
export function batch_unmask_card(card: any, reveals: any): any;
/**
* @param {any} player_env
* @returns {string}
*/
export function create_play_env(player_env: any): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly generate_key: (a: number) => void;
  readonly generate_key_by_seed: (a: number, b: number, c: number) => void;
  readonly reveal_card: (a: number, b: number, c: number, d: number) => void;
  readonly batch_reveal_card: (a: number, b: number, c: number, d: number) => void;
  readonly unmask_card: (a: number, b: number, c: number) => void;
  readonly batch_unmask_card: (a: number, b: number, c: number) => void;
  readonly create_play_env: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
