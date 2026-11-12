/**
 * 直接通过 Deno FFI 调用 ostfil 动态库（不再经过本地 HTTP 网关）。
 *
 * 使用前：
 *   cargo build --release
 *   # 在仓库根目录运行，storage/ 会落在当前工作目录下
 *   deno run --allow-ffi --allow-env --allow-read ts/storage_ostfil.ts
 *
 * 库路径默认取 ../target/release/ 下对应当前平台的产物，
 * 可用环境变量 OSTFIL_LIB 指定其他路径。
 */

export type FfiSymbol =
  | "post_storage"
  | "post_get_fil"
  | "post_delete_file";

const OUT_BUFFER_SIZE = 65536;

function resolveLibUrl(): string | URL {
  const override = Deno.env.get("OSTFIL_LIB");
  if (override) return override;

  const file = Deno.build.os === "windows"
    ? "ostfil.dll"
    : Deno.build.os === "darwin"
    ? "libostfil.dylib"
    : "libostfil.so";

  return new URL(`../target/release/${file}`, import.meta.url);
}

const lib = Deno.dlopen(resolveLibUrl(), {
  post_storage: { parameters: ["pointer", "pointer"], result: "i32" },
  post_get_fil: { parameters: ["pointer", "pointer"], result: "i32" },
  post_delete_file: { parameters: ["pointer", "pointer"], result: "i32" },
});

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/**
 * 调用一个 FFI 函数：JSON 进、JSON 出。
 *
 * 返回值只表示输出缓冲区是否写入成功（0 成功，-1/-2 失败），
 * 业务结果在返回对象的 code 字段中。
 */
export function callFfi(name: FfiSymbol, payload: unknown): unknown {
  const encoded = encoder.encode(JSON.stringify(payload));
  const input = new Uint8Array(encoded.length + 1);
  input.set(encoded);
  input[encoded.length] = 0; // C 字符串结尾

  const output = new Uint8Array(OUT_BUFFER_SIZE);

  const fn = lib.symbols[name] as (
    jsonIn: Deno.PointerValue,
    jsonOut: Deno.PointerValue,
  ) => number;
  const rc = fn(Deno.UnsafePointer.of(input), Deno.UnsafePointer.of(output));

  if (rc === -1) throw new Error(`${name}: json_out 为空指针`);
  if (rc === -2) throw new Error(`${name}: 输出超过 ${OUT_BUFFER_SIZE} 字节`);

  const end = output.indexOf(0);
  const text = decoder.decode(end === -1 ? output : output.subarray(0, end));
  if (text === "") throw new Error(`${name}: 库未写入输出（rc=${rc}）`);

  return JSON.parse(text);
}

export function closeLib(): void {
  lib.close();
}
