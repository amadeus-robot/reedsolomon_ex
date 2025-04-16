# reedsolomon_ex
Reed Solomon Rustler bindings for Elixir

## How

Use it like

```elixir
msg = :crypto.strong_rand_bytes(1024*3)
shards = 3
shards_recovery = 3
shard_size = 1024

r = ReedSolomonEx.create_resource(shards, shards_recovery, shard_size)
frags = ReedSolomonEx.encode_shards(r, msg)
true = msg == ReedSolomonEx.decode_shards(r, frags, shards+shards_recovery, byte_size(msg))

msg = :crypto.strong_rand_bytes(1024*3)
frags = ReedSolomonEx.encode_shards(r, msg)
true = msg == ReedSolomonEx.decode_shards(r, frags, shards+shards_recovery, byte_size(msg))

msg = :crypto.strong_rand_bytes(1024*3)
frags = ReedSolomonEx.encode_shards(r, msg)
true = msg == ReedSolomonEx.decode_shards(r, frags, shards+shards_recovery, byte_size(msg))
```