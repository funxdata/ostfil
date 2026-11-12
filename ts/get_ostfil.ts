import { callFfi } from "./ffi.ts";

const result = callFfi("post_get_fil", {
  uuid: "18eb8248-2f5f-490b-96d0-8f155e6f791b",
});

console.log(JSON.stringify(result, null, 2));
