while (true) {
  console.log(await Deno.core.ops.op_electrum_poll_events())
}