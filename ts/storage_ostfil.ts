const res = await fetch("http://127.0.0.1:44944/libostfil", {
  method: "POST",
  headers: {
    "Content-Type": "application/os",
    "FFI-Symbol": "storage",
  },
  body: JSON.stringify({
    file_path: "/home/funx/Downloads/logo.png",
  }),
});

console.log(await res.json());
