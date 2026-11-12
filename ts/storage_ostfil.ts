import { callFfi } from "./ffi.ts";

const result = callFfi("post_storage", {
  file_path: "/home/funx/Downloads/logo.png",
});

console.log(JSON.stringify(result, null, 2));
