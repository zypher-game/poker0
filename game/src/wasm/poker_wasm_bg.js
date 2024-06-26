let wasm;
export function __wbg_set_wasm(val) {
  wasm = val;
}

const lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;

let cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = null;

function getUint8Memory0() {
  if (cachedUint8Memory0 === null || cachedUint8Memory0.byteLength === 0) {
    cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
  ptr = ptr >>> 0;
  return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
  if (heap_next === heap.length) heap.push(heap.length + 1);
  const idx = heap_next;
  heap_next = heap[idx];

  heap[idx] = obj;
  return idx;
}

function getObject(idx) {
  return heap[idx];
}

function dropObject(idx) {
  if (idx < 132) return;
  heap[idx] = heap_next;
  heap_next = idx;
}

function takeObject(idx) {
  const ret = getObject(idx);
  dropObject(idx);
  return ret;
}

let WASM_VECTOR_LEN = 0;

const lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;

let cachedTextEncoder = new lTextEncoder('utf-8');

const encodeString =
  typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
        return cachedTextEncoder.encodeInto(arg, view);
      }
    : function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
          read: arg.length,
          written: buf.length,
        };
      };

function passStringToWasm0(arg, malloc, realloc) {
  if (realloc === undefined) {
    const buf = cachedTextEncoder.encode(arg);
    const ptr = malloc(buf.length, 1) >>> 0;
    getUint8Memory0()
      .subarray(ptr, ptr + buf.length)
      .set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
  }

  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;

  const mem = getUint8Memory0();

  let offset = 0;

  for (; offset < len; offset++) {
    const code = arg.charCodeAt(offset);
    if (code > 0x7f) break;
    mem[ptr + offset] = code;
  }

  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset);
    }
    ptr = realloc(ptr, len, (len = offset + arg.length * 3), 1) >>> 0;
    const view = getUint8Memory0().subarray(ptr + offset, ptr + len);
    const ret = encodeString(arg, view);

    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }

  WASM_VECTOR_LEN = offset;
  return ptr;
}

function isLikeNone(x) {
  return x === undefined || x === null;
}

let cachedInt32Memory0 = null;

function getInt32Memory0() {
  if (cachedInt32Memory0 === null || cachedInt32Memory0.byteLength === 0) {
    cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
  }
  return cachedInt32Memory0;
}

let cachedFloat64Memory0 = null;

function getFloat64Memory0() {
  if (cachedFloat64Memory0 === null || cachedFloat64Memory0.byteLength === 0) {
    cachedFloat64Memory0 = new Float64Array(wasm.memory.buffer);
  }
  return cachedFloat64Memory0;
}

let cachedBigInt64Memory0 = null;

function getBigInt64Memory0() {
  if (cachedBigInt64Memory0 === null || cachedBigInt64Memory0.byteLength === 0) {
    cachedBigInt64Memory0 = new BigInt64Array(wasm.memory.buffer);
  }
  return cachedBigInt64Memory0;
}

function debugString(val) {
  // primitive types
  const type = typeof val;
  if (type == 'number' || type == 'boolean' || val == null) {
    return `${val}`;
  }
  if (type == 'string') {
    return `"${val}"`;
  }
  if (type == 'symbol') {
    const description = val.description;
    if (description == null) {
      return 'Symbol';
    } else {
      return `Symbol(${description})`;
    }
  }
  if (type == 'function') {
    const name = val.name;
    if (typeof name == 'string' && name.length > 0) {
      return `Function(${name})`;
    } else {
      return 'Function';
    }
  }
  // objects
  if (Array.isArray(val)) {
    const length = val.length;
    let debug = '[';
    if (length > 0) {
      debug += debugString(val[0]);
    }
    for (let i = 1; i < length; i++) {
      debug += ', ' + debugString(val[i]);
    }
    debug += ']';
    return debug;
  }
  // Test for built-in
  const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
  let className;
  if (builtInMatches.length > 1) {
    className = builtInMatches[1];
  } else {
    // Failed to match the standard '[object ClassName]'
    return toString.call(val);
  }
  if (className == 'Object') {
    // we're a user defined class or Object
    // JSON.stringify avoids problems with cycles, and is generally much
    // easier than looping through ownProperties of `val`.
    try {
      return 'Object(' + JSON.stringify(val) + ')';
    } catch (_) {
      return 'Object';
    }
  }
  // errors
  if (val instanceof Error) {
    return `${val.name}: ${val.message}\n${val.stack}`;
  }
  // TODO we could test for more things here, like `Set`s and `Map`s.
  return className;
}
/**
 * generate keypair
 * @returns {any}
 */
export function generate_key() {
  try {
    const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    wasm.generate_key(retptr);
    let r0 = getInt32Memory0()[retptr / 4 + 0];
    let r1 = getInt32Memory0()[retptr / 4 + 1];
    let r2 = getInt32Memory0()[retptr / 4 + 2];
    if (r2) {
      throw takeObject(r1);
    }
    return takeObject(r0);
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
}

/**
 * generate keypair
 * @param {string} seed
 * @returns {any}
 */
export function generate_key_by_seed(seed) {
  try {
    const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    const ptr0 = passStringToWasm0(seed, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.generate_key_by_seed(retptr, ptr0, len0);
    let r0 = getInt32Memory0()[retptr / 4 + 0];
    let r1 = getInt32Memory0()[retptr / 4 + 1];
    let r2 = getInt32Memory0()[retptr / 4 + 2];
    if (r2) {
      throw takeObject(r1);
    }
    return takeObject(r0);
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
}

/**
 * compute masked to revealed card and the revealed proof
 * @param {string} sk
 * @param {any} card
 * @returns {any}
 */
export function reveal_card(sk, card) {
  try {
    const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    const ptr0 = passStringToWasm0(sk, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.reveal_card(retptr, ptr0, len0, addHeapObject(card));
    let r0 = getInt32Memory0()[retptr / 4 + 0];
    let r1 = getInt32Memory0()[retptr / 4 + 1];
    let r2 = getInt32Memory0()[retptr / 4 + 2];
    if (r2) {
      throw takeObject(r1);
    }
    return takeObject(r0);
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
}

/**
 * compute masked to revealed card and the revealed proof
 * @param {string} sk
 * @param {any} card
 * @returns {any}
 */
export function batch_reveal_card(sk, card) {
  try {
    const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    const ptr0 = passStringToWasm0(sk, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.batch_reveal_card(retptr, ptr0, len0, addHeapObject(card));
    let r0 = getInt32Memory0()[retptr / 4 + 0];
    let r1 = getInt32Memory0()[retptr / 4 + 1];
    let r2 = getInt32Memory0()[retptr / 4 + 2];
    if (r2) {
      throw takeObject(r1);
    }
    return takeObject(r0);
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
}

/**
 * unmask the card use all reveals
 * @param {any} card
 * @param {any} reveals
 * @returns {number}
 */
export function unmask_card(card, reveals) {
  try {
    const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    wasm.unmask_card(retptr, addHeapObject(card), addHeapObject(reveals));
    let r0 = getInt32Memory0()[retptr / 4 + 0];
    let r1 = getInt32Memory0()[retptr / 4 + 1];
    let r2 = getInt32Memory0()[retptr / 4 + 2];
    if (r2) {
      throw takeObject(r1);
    }
    return r0 >>> 0;
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
}

/**
 * batch unmask the card use all reveals
 * @param {any} card
 * @param {any} reveals
 * @returns {any}
 */
export function batch_unmask_card(card, reveals) {
  try {
    const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    wasm.batch_unmask_card(retptr, addHeapObject(card), addHeapObject(reveals));
    let r0 = getInt32Memory0()[retptr / 4 + 0];
    let r1 = getInt32Memory0()[retptr / 4 + 1];
    let r2 = getInt32Memory0()[retptr / 4 + 2];
    if (r2) {
      throw takeObject(r1);
    }
    return takeObject(r0);
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
  }
}

/**
 * @param {any} player_env
 * @returns {string}
 */
export function create_play_env(player_env) {
  let deferred2_0;
  let deferred2_1;
  try {
    const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
    wasm.create_play_env(retptr, addHeapObject(player_env));
    let r0 = getInt32Memory0()[retptr / 4 + 0];
    let r1 = getInt32Memory0()[retptr / 4 + 1];
    let r2 = getInt32Memory0()[retptr / 4 + 2];
    let r3 = getInt32Memory0()[retptr / 4 + 3];
    let ptr1 = r0;
    let len1 = r1;
    if (r3) {
      ptr1 = 0;
      len1 = 0;
      throw takeObject(r2);
    }
    deferred2_0 = ptr1;
    deferred2_1 = len1;
    return getStringFromWasm0(ptr1, len1);
  } finally {
    wasm.__wbindgen_add_to_stack_pointer(16);
    wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
  }
}

function handleError(f, args) {
  try {
    return f.apply(this, args);
  } catch (e) {
    wasm.__wbindgen_exn_store(addHeapObject(e));
  }
}

export function __wbindgen_string_new(arg0, arg1) {
  const ret = getStringFromWasm0(arg0, arg1);
  return addHeapObject(ret);
}

export function __wbindgen_object_drop_ref(arg0) {
  takeObject(arg0);
}

export function __wbindgen_string_get(arg0, arg1) {
  const obj = getObject(arg1);
  const ret = typeof obj === 'string' ? obj : undefined;
  let ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  let len1 = WASM_VECTOR_LEN;
  getInt32Memory0()[arg0 / 4 + 1] = len1;
  getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}

export function __wbindgen_is_object(arg0) {
  const val = getObject(arg0);
  const ret = typeof val === 'object' && val !== null;
  return ret;
}

export function __wbindgen_is_undefined(arg0) {
  const ret = getObject(arg0) === undefined;
  return ret;
}

export function __wbindgen_in(arg0, arg1) {
  const ret = getObject(arg0) in getObject(arg1);
  return ret;
}

export function __wbindgen_is_bigint(arg0) {
  const ret = typeof getObject(arg0) === 'bigint';
  return ret;
}

export function __wbindgen_bigint_from_u64(arg0) {
  const ret = BigInt.asUintN(64, arg0);
  return addHeapObject(ret);
}

export function __wbindgen_jsval_eq(arg0, arg1) {
  const ret = getObject(arg0) === getObject(arg1);
  return ret;
}

export function __wbindgen_number_new(arg0) {
  const ret = arg0;
  return addHeapObject(ret);
}

export function __wbindgen_error_new(arg0, arg1) {
  const ret = new Error(getStringFromWasm0(arg0, arg1));
  return addHeapObject(ret);
}

export function __wbindgen_jsval_loose_eq(arg0, arg1) {
  const ret = getObject(arg0) == getObject(arg1);
  return ret;
}

export function __wbindgen_boolean_get(arg0) {
  const v = getObject(arg0);
  const ret = typeof v === 'boolean' ? (v ? 1 : 0) : 2;
  return ret;
}

export function __wbindgen_number_get(arg0, arg1) {
  const obj = getObject(arg1);
  const ret = typeof obj === 'number' ? obj : undefined;
  getFloat64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? 0 : ret;
  getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
}

export function __wbindgen_as_number(arg0) {
  const ret = +getObject(arg0);
  return ret;
}

export function __wbindgen_object_clone_ref(arg0) {
  const ret = getObject(arg0);
  return addHeapObject(ret);
}

export function __wbg_getwithrefkey_edc2c8960f0f1191(arg0, arg1) {
  const ret = getObject(arg0)[getObject(arg1)];
  return addHeapObject(ret);
}

export function __wbg_set_f975102236d3c502(arg0, arg1, arg2) {
  getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
}

export function __wbg_crypto_1d1f22824a6a080c(arg0) {
  const ret = getObject(arg0).crypto;
  return addHeapObject(ret);
}

export function __wbg_process_4a72847cc503995b(arg0) {
  const ret = getObject(arg0).process;
  return addHeapObject(ret);
}

export function __wbg_versions_f686565e586dd935(arg0) {
  const ret = getObject(arg0).versions;
  return addHeapObject(ret);
}

export function __wbg_node_104a2ff8d6ea03a2(arg0) {
  const ret = getObject(arg0).node;
  return addHeapObject(ret);
}

export function __wbindgen_is_string(arg0) {
  const ret = typeof getObject(arg0) === 'string';
  return ret;
}

export function __wbg_msCrypto_eb05e62b530a1508(arg0) {
  const ret = getObject(arg0).msCrypto;
  return addHeapObject(ret);
}

export function __wbg_require_cca90b1a94a0255b() {
  return handleError(function () {
    const ret = module.require;
    return addHeapObject(ret);
  }, arguments);
}

export function __wbindgen_is_function(arg0) {
  const ret = typeof getObject(arg0) === 'function';
  return ret;
}

export function __wbg_randomFillSync_5c9c955aa56b6049() {
  return handleError(function (arg0, arg1) {
    getObject(arg0).randomFillSync(takeObject(arg1));
  }, arguments);
}

export function __wbg_getRandomValues_3aa56aa6edec874c() {
  return handleError(function (arg0, arg1) {
    getObject(arg0).getRandomValues(getObject(arg1));
  }, arguments);
}

export function __wbg_get_bd8e338fbd5f5cc8(arg0, arg1) {
  const ret = getObject(arg0)[arg1 >>> 0];
  return addHeapObject(ret);
}

export function __wbg_length_cd7af8117672b8b8(arg0) {
  const ret = getObject(arg0).length;
  return ret;
}

export function __wbg_new_16b304a2cfa7ff4a() {
  const ret = new Array();
  return addHeapObject(ret);
}

export function __wbg_newnoargs_e258087cd0daa0ea(arg0, arg1) {
  const ret = new Function(getStringFromWasm0(arg0, arg1));
  return addHeapObject(ret);
}

export function __wbg_next_40fc327bfc8770e6(arg0) {
  const ret = getObject(arg0).next;
  return addHeapObject(ret);
}

export function __wbg_next_196c84450b364254() {
  return handleError(function (arg0) {
    const ret = getObject(arg0).next();
    return addHeapObject(ret);
  }, arguments);
}

export function __wbg_done_298b57d23c0fc80c(arg0) {
  const ret = getObject(arg0).done;
  return ret;
}

export function __wbg_value_d93c65011f51a456(arg0) {
  const ret = getObject(arg0).value;
  return addHeapObject(ret);
}

export function __wbg_iterator_2cee6dadfd956dfa() {
  const ret = Symbol.iterator;
  return addHeapObject(ret);
}

export function __wbg_get_e3c254076557e348() {
  return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
  }, arguments);
}

export function __wbg_call_27c0f87801dedf93() {
  return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
  }, arguments);
}

export function __wbg_new_72fb9a18b5ae2624() {
  const ret = new Object();
  return addHeapObject(ret);
}

export function __wbg_self_ce0dbfc45cf2f5be() {
  return handleError(function () {
    const ret = self.self;
    return addHeapObject(ret);
  }, arguments);
}

export function __wbg_window_c6fb939a7f436783() {
  return handleError(function () {
    const ret = window.window;
    return addHeapObject(ret);
  }, arguments);
}

export function __wbg_globalThis_d1e6af4856ba331b() {
  return handleError(function () {
    const ret = globalThis.globalThis;
    return addHeapObject(ret);
  }, arguments);
}

export function __wbg_global_207b558942527489() {
  return handleError(function () {
    const ret = global.global;
    return addHeapObject(ret);
  }, arguments);
}

export function __wbg_set_d4638f722068f043(arg0, arg1, arg2) {
  getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
}

export function __wbg_isArray_2ab64d95e09ea0ae(arg0) {
  const ret = Array.isArray(getObject(arg0));
  return ret;
}

export function __wbg_instanceof_ArrayBuffer_836825be07d4c9d2(arg0) {
  let result;
  try {
    result = getObject(arg0) instanceof ArrayBuffer;
  } catch (_) {
    result = false;
  }
  const ret = result;
  return ret;
}

export function __wbg_call_b3ca7c6051f9bec1() {
  return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
  }, arguments);
}

export function __wbg_isSafeInteger_f7b04ef02296c4d2(arg0) {
  const ret = Number.isSafeInteger(getObject(arg0));
  return ret;
}

export function __wbg_buffer_12d079cc21e14bdb(arg0) {
  const ret = getObject(arg0).buffer;
  return addHeapObject(ret);
}

export function __wbg_newwithbyteoffsetandlength_aa4a17c33a06e5cb(arg0, arg1, arg2) {
  const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
  return addHeapObject(ret);
}

export function __wbg_new_63b92bc8671ed464(arg0) {
  const ret = new Uint8Array(getObject(arg0));
  return addHeapObject(ret);
}

export function __wbg_set_a47bac70306a19a7(arg0, arg1, arg2) {
  getObject(arg0).set(getObject(arg1), arg2 >>> 0);
}

export function __wbg_length_c20a40f15020d68a(arg0) {
  const ret = getObject(arg0).length;
  return ret;
}

export function __wbg_instanceof_Uint8Array_2b3bbecd033d19f6(arg0) {
  let result;
  try {
    result = getObject(arg0) instanceof Uint8Array;
  } catch (_) {
    result = false;
  }
  const ret = result;
  return ret;
}

export function __wbg_newwithlength_e9b4878cebadb3d3(arg0) {
  const ret = new Uint8Array(arg0 >>> 0);
  return addHeapObject(ret);
}

export function __wbg_subarray_a1f73cd4b5b42fe1(arg0, arg1, arg2) {
  const ret = getObject(arg0).subarray(arg1 >>> 0, arg2 >>> 0);
  return addHeapObject(ret);
}

export function __wbindgen_bigint_get_as_i64(arg0, arg1) {
  const v = getObject(arg1);
  const ret = typeof v === 'bigint' ? v : undefined;
  getBigInt64Memory0()[arg0 / 8 + 1] = isLikeNone(ret) ? BigInt(0) : ret;
  getInt32Memory0()[arg0 / 4 + 0] = !isLikeNone(ret);
}

export function __wbindgen_debug_string(arg0, arg1) {
  const ret = debugString(getObject(arg1));
  const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len1 = WASM_VECTOR_LEN;
  getInt32Memory0()[arg0 / 4 + 1] = len1;
  getInt32Memory0()[arg0 / 4 + 0] = ptr1;
}

export function __wbindgen_throw(arg0, arg1) {
  throw new Error(getStringFromWasm0(arg0, arg1));
}

export function __wbindgen_memory() {
  const ret = wasm.memory;
  return addHeapObject(ret);
}
