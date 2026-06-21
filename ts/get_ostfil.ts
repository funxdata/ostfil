const res = await fetch("http://127.0.0.1:44944/libostfil", {
  method: "POST",
  headers: {
    "Content-Type": "application/os",
    "FFI-Symbol": "get_fil",
  },
  body: JSON.stringify({
    uuid: "18eb8248-2f5f-490b-96d0-8f155e6f791b",
  }),
});

console.log(await res.json());
