// Minimal Node.js runner for a raw wasm32-unknown-unknown module.
// Usage: node scripts/run_wasm.mjs [path/to/module.wasm]
import fs from 'node:fs';
import path from 'node:path';
import url from 'node:url';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
const wasmPath = process.argv[2] ?? path.join(__dirname, '..', 'target', 'wasm32-unknown-unknown', 'debug', 'flecs_ecs.wasm');

function buildImports(neededImports) {
  // Construct imports lazily based on what the module actually requests
  const env = {};
  const wasi_snapshot_preview1 = {};
  // Will be set after instantiation
  let wasmInstance = null;

  function memU8() {
    const mem = (env.memory) ? env.memory : (wasmInstance?.exports?.memory);
    return mem ? new Uint8Array(mem.buffer) : null;
  }
  function memDV() {
    const mem = (env.memory) ? env.memory : (wasmInstance?.exports?.memory);
    return mem ? new DataView(mem.buffer) : null;
  }
  let bumpPtr = 0;
  const allocs = new Map(); // ptr -> size
  function ensureHeapBase() {
    if (bumpPtr) return bumpPtr;
    // Prefer exported __heap_base if available
    const exp = wasmInstance?.exports;
    let hb = 0;
    if (exp && exp.__heap_base instanceof WebAssembly.Global) {
      hb = exp.__heap_base.value >>> 0;
    } else if (exp && typeof exp.__heap_base === 'number') {
      hb = exp.__heap_base >>> 0;
    }
    bumpPtr = hb || 0x10000; // fallback to 64KiB
    return bumpPtr;
  }
  function ensureCapacity(endPtr) {
    const mem = (env.memory) ? env.memory : (wasmInstance?.exports?.memory);
    if (!mem) return;
    const pageSize = 65536;
    while (endPtr > mem.buffer.byteLength) {
      const pagesToGrow = Math.ceil((endPtr - mem.buffer.byteLength) / pageSize);
      mem.grow(pagesToGrow);
    }
  }

  const needsEnvMemory = neededImports.some(i => i.module === 'env' && i.name === 'memory');
  const needsEnvTable = neededImports.some(i => i.module === 'env' && i.name === 'table');
  if (needsEnvMemory) {
    env.memory = new WebAssembly.Memory({ initial: 256, maximum: 1024 });
  }
  if (needsEnvTable) {
    // Use 'anyfunc' for broader compatibility
    env.table = new WebAssembly.Table({ initial: 0, element: 'anyfunc' });
  }

  // Provide a few common stubs
  if (neededImports.some(i => i.module === 'env' && i.name === 'abort')) {
    env.abort = (msgPtr, filePtr, line, col) => {
      console.error('abort called', { msgPtr, filePtr, line, col });
      throw new Error('WASM abort');
    };
  }

  // C library shims if requested
  if (neededImports.some(i => i.module === 'env' && i.name === 'isupper')) {
    env.isupper = (c) => {
      return c >= 65 && c <= 90 ? 1 : 0;
    };
  }
  if (neededImports.some(i => i.module === 'env' && i.name === 'isdigit')) {
    env.isdigit = (c) => {
      return c >= 48 && c <= 57 ? 1 : 0;
    };
  }
  if (neededImports.some(i => i.module === 'env' && i.name === 'tolower')) {
    env.tolower = (c) => {
      if (c >= 65 && c <= 90) return c + 32; // 'A'..'Z' -> 'a'..'z'
      return c;
    };
  }
  if (neededImports.some(i => i.module === 'env' && i.name === 'strncpy')) {
    env.strncpy = (dst, src, n) => {
      const u8 = memU8();
      if (!u8) return dst;
      let i = 0;
      for (; i < n; i++) {
        const ch = u8[src + i];
        u8[dst + i] = ch;
        if (ch === 0) {
          i++;
          break;
        }
      }
      for (; i < n; i++) {
        u8[dst + i] = 0;
      }
      return dst;
    };
  }

  // Very small bump-allocator (no free) for demos
  const needsMalloc = neededImports.some(i => i.module === 'env' && ['malloc','calloc','realloc','free'].includes(i.name));
  if (needsMalloc) {
    env.malloc = (size) => {
      ensureHeapBase();
      size = (size + 7) & ~7; // 8-byte align
      const ptr = bumpPtr;
      const end = (bumpPtr + size) >>> 0;
      ensureCapacity(end);
      bumpPtr = end;
      allocs.set(ptr >>> 0, size >>> 0);
      return ptr >>> 0;
    };
    env.calloc = (nmemb, size) => {
      const total = (nmemb >>> 0) * (size >>> 0);
      const ptr = env.malloc(total);
      const u8 = memU8();
      if (u8) u8.fill(0, ptr, ptr + total);
      return ptr >>> 0;
    };
    env.realloc = (ptr, size) => {
      if (ptr === 0) return env.malloc(size);
      const oldSize = allocs.get(ptr >>> 0) || 0;
      const newPtr = env.malloc(size >>> 0);
      const u8 = memU8();
      if (u8) {
        const copy = Math.min(oldSize >>> 0, size >>> 0);
        if (copy > 0) {
          u8.set(u8.subarray(ptr >>> 0, (ptr >>> 0) + copy), newPtr >>> 0);
        }
      }
      allocs.delete(ptr >>> 0);
      return newPtr >>> 0;
    };
    env.free = (_ptr) => {
      if (_ptr) allocs.delete(_ptr >>> 0);
    };
  }

  if (neededImports.some(i => i.module === 'wasi_snapshot_preview1')) {
    // Minimal fd_write: decode iovs from wasm memory and write to stdout/stderr
    wasi_snapshot_preview1.fd_write = (fd, iovsPtr, iovsLen, nwrittenPtr) => {
      try {
        const mem = env.memory || imports?.env?.memory;
        if (!mem) return 0;
        const u8 = new Uint8Array(mem.buffer);
        const dv = new DataView(mem.buffer);
        let written = 0;
        for (let i = 0; i < iovsLen; i++) {
          const base = dv.getUint32(iovsPtr + i * 8, true);
          const len = dv.getUint32(iovsPtr + i * 8 + 4, true);
          const chunk = new TextDecoder().decode(u8.subarray(base, base + len));
          if (fd === 1) process.stdout.write(chunk);
          else if (fd === 2) process.stderr.write(chunk);
          written += len;
        }
        dv.setUint32(nwrittenPtr, written, true);
      } catch (e) {
        // ignore
      }
      return 0; // __WASI_ERRNO_SUCCESS
    };
  }

  const imports = { env, wasi_snapshot_preview1 };
  // Allow caller to patch in the instance after instantiation
  imports.__setWasmInstance = (inst) => { wasmInstance = inst; };
  return imports;
}

async function main() {
  const abs = path.resolve(wasmPath);
  if (!fs.existsSync(abs)) {
    console.error(`WASM file not found: ${abs}`);
    process.exit(1);
  }
  console.log('Loading WASM:', abs);
  const bytes = fs.readFileSync(abs);

  let module;
  try {
    module = await WebAssembly.compile(bytes);
  } catch (e) {
    console.error('Failed to compile WASM:', e);
    process.exit(1);
  }

  const neededImports = WebAssembly.Module.imports(module);
  if (neededImports.length) {
    console.log('Imports required:', neededImports.map(i => `${i.module}.${i.name}`).join(', '));
  } else {
    console.log('No imports required.');
  }

  const imports = buildImports(neededImports);

  let instance;
  try {
    instance = await WebAssembly.instantiate(module, imports);
  } catch (e) {
    console.error('Failed to instantiate WASM with default imports. You may need a WASI runtime (wasmtime/wasmer) or custom JS glue. Error:', e);
    process.exit(1);
  }
  if (typeof imports.__setWasmInstance === 'function') {
    imports.__setWasmInstance(instance);
  }

  const exports = instance.exports;
  const exportNames = Object.keys(exports);
  console.log('Exports:', exportNames);

  // Try to invoke a typical entry point if present
  const candidates = ['_start', 'main'];
  for (const name of candidates) {
    if (typeof exports[name] === 'function') {
      console.log(`Invoking export: ${name}()`);
      try {
        const ret = exports[name]();
        console.log(`${name}() returned:`, ret);
      } catch (e) {
        console.error(`Error calling ${name}():`, e);
      }
      // Don't return; continue to try example helpers to show output.
    }
  }

  // If our example helpers exist, use them to show output
  if (typeof exports.example_pos_x === 'function' && typeof exports.example_pos_y === 'function') {
    const x = exports.example_pos_x();
    const y = exports.example_pos_y();
    console.log(`Position from wasm: (${x}, ${y})`);
    return;
  }

  console.log('No start function exported (_start/main). Module may be a library; inspect exports above for callable functions.');
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
