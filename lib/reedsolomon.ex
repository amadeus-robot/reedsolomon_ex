defmodule ReedSolomonEx do
  use Rustler,
    otp_app: :reedsolomon_ex,
    crate: :reedsolomon_ex
    
  def create_resource(_data_shards, _recovery_shards, _size_shard), do: :erlang.nif_error(:nif_not_loaded)
  def encode_shards(_resource, _data), do: :erlang.nif_error(:nif_not_loaded)
  def decode_shards(_resource, _shards, _total_shards, _original_size), do: :erlang.nif_error(:nif_not_loaded)
end
