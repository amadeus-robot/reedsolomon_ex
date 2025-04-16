use rustler::types::{Binary, OwnedBinary};
use rustler::{Encoder, Error, Env, Term, NifResult, ResourceArc};

use reed_solomon_simd::{ReedSolomonDecoder, ReedSolomonEncoder};
use std::sync::Mutex;

pub struct ReedSolomonResource {
    pub p_encoder: Mutex<ReedSolomonEncoder>,
    pub p_decoder: Mutex<ReedSolomonDecoder>,
}

rustler::init!("Elixir.ReedSolomonEx", load = on_load);
fn on_load(env: Env, _info: Term) -> bool {
    let _ = rustler::resource!(ReedSolomonResource, env);
    true
}

#[rustler::nif]
pub fn create_resource(data_shards: usize, recovery_shards: usize, size_shard: usize) -> NifResult<ResourceArc<ReedSolomonResource>> {
    let encoder = ReedSolomonEncoder::new(data_shards, recovery_shards, size_shard)
        .map_err(|_| Error::BadArg)?;
    let decoder = ReedSolomonDecoder::new(data_shards, recovery_shards, size_shard)
        .map_err(|_| Error::BadArg)?;
    let resource = ReedSolomonResource {
        p_encoder: Mutex::new(encoder),
        p_decoder: Mutex::new(decoder),
    };
    Ok(ResourceArc::new(resource))
}

#[rustler::nif]
pub fn encode_shards<'a>(env: Env<'a>, resource: ResourceArc<ReedSolomonResource>, data: Binary) -> Result<rustler::Term<'a>, rustler::Error> {
    let chunk_size = 1024;

    let mut encoder_lock = resource.p_encoder.lock().map_err(|_| Error::Term(Box::new("Poisoned mutex")))?;

    let chunk_count = (data.len() + 1023) / 1024;
    let mut encoded_shards = Vec::with_capacity(chunk_count * 2);
    let mut itr = 0;

    // Step through `data` in increments of `chunk_size`.
    for chunk_start in (0..data.len()).step_by(chunk_size) {
        let chunk_end = (chunk_start + chunk_size).min(data.len());
        let chunk = &data[chunk_start..chunk_end];

        // Create a 1024-byte buffer initialized to 0.
        let mut buffer = [0u8; 1024];
        buffer[..chunk.len()].copy_from_slice(chunk);

        encoder_lock.add_original_shard(&buffer).map_err(|_| Error::BadArg)?;

        let mut bin = OwnedBinary::new(chunk_size).unwrap();
        bin.as_mut_slice().copy_from_slice(&buffer);
        encoded_shards.push((itr, Binary::from_owned(bin, env)));
        itr += 1;
    }
    
    let result = encoder_lock.encode().map_err(|_| Error::BadArg)?;
    let recovery: Vec<_> = result.recovery_iter().collect();
    for recovered_shard in recovery {
        let mut bin = OwnedBinary::new(recovered_shard.len()).unwrap();
        bin.as_mut_slice().copy_from_slice(recovered_shard);
        encoded_shards.push((itr, Binary::from_owned(bin, env)));
        itr += 1;
    }

    Ok(encoded_shards.encode(env))
}

#[rustler::nif]
pub fn decode_shards<'a>(env: Env<'a>, resource: ResourceArc<ReedSolomonResource>, shards_term: Term<'a>, 
    total_shards: usize, original_size: usize) -> Result<rustler::Term<'a>, rustler::Error> 
{
    let shards: Vec<(usize, Binary<'a>)> = shards_term.decode().map_err(|_| Error::BadArg)?;

    let mut decoder_lock = resource.p_decoder.lock().map_err(|_| Error::Term(Box::new("Poisoned mutex")))?;

    let mut combined = vec![0u8; original_size];

    let half = total_shards / 2;
    for (index, bin) in shards {
        let idx_usize = index as usize;
        if idx_usize < half {
            let shard_data = bin.as_slice();

            let offset = idx_usize * 1024;
            // Protect against going past original_size
            let end = (offset + shard_data.len()).min(original_size);
            combined[offset..end].copy_from_slice(&shard_data[..(end - offset)]);

            decoder_lock.add_original_shard(index, shard_data).map_err(|_| Error::BadArg)?;
        } else {
            decoder_lock.add_recovery_shard(index-half, bin.as_slice()).map_err(|_| Error::BadArg)?;
        }
    }
    let result = decoder_lock.decode().map_err(|_| Error::BadArg)?;

    for idx in 0..half {
        if let Some(shard_data) = result.restored_original(idx) {
            let offset = idx * 1024;
            let end = (offset + shard_data.len()).min(original_size);
            combined[offset..end].copy_from_slice(&shard_data[..(end - offset)]);
        }
    }

    let mut out_bin = OwnedBinary::new(combined.len()).unwrap();
    out_bin.as_mut_slice().copy_from_slice(&combined);
    Ok(Binary::from_owned(out_bin, env).encode(env))
}