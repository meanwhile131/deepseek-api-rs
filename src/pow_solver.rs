//! Proof of Work solver using WebAssembly.

use anyhow::{Context, Result, anyhow};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::{json, Value};
use wasmtime::{Engine, Store, Instance, Memory, TypedFunc, Module};

use crate::wasm_download::get_wasm_path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub salt: String,
    pub expire_at: i64,
    pub challenge: String,
    pub difficulty: f64,
    pub algorithm: String,
    pub signature: String,
    pub target_path: String,
}

#[derive(Debug, Serialize)]
pub struct SolveResponse {
    algorithm: String,
    challenge: String,
    salt: String,
    answer: i64,
    signature: String,
    target_path: String,
}

/// Solver for DeepSeek Proof of Work challenges.
pub struct POWSolver {
    engine: Engine,
    store: Store<()>,
    instance: Instance,
    memory: Memory,
    wasm_solve: TypedFunc<(i32, i32, i32, i32, i32, f64), ()>,
    alloc: TypedFunc<(i32, i32), i32>,
    add_stack: TypedFunc<(i32,), i32>,
}

impl POWSolver {
    /// Creates a new PoW solver, loading the WASM module from cache or downloading it.
    pub async fn new() -> Result<Self> {
        let wasm_path = get_wasm_path().await?;
        let wasm_bytes = tokio::fs::read(&wasm_path).await
            .with_context(|| format!("Failed to read WASM file at {:?}", wasm_path))?;

        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes)?;
        let mut store = Store::new(&engine, ());

        let instance = Instance::new(&mut store, &module, &[])?;

        let memory = instance.get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("memory export not found"))?;

        let wasm_solve = instance.get_typed_func::<(i32, i32, i32, i32, i32, f64), ()>(&mut store, "wasm_solve")?;
        let alloc = instance.get_typed_func::<(i32, i32), i32>(&mut store, "__wbindgen_export_0")?;
        let add_stack = instance.get_typed_func::<(i32,), i32>(&mut store, "__wbindgen_add_to_stack_pointer")?;

        Ok(Self {
            engine,
            store,
            instance,
            memory,
            wasm_solve,
            alloc,
            add_stack,
        })
    }

    /// Writes a string to WASM linear memory and returns (pointer, length).
    fn write_str_to_memory(&mut self, data: &str) -> Result<(i32, i32)> {
        let bytes = data.as_bytes();
        let len = bytes.len() as i32;
        let ptr = self.alloc.call(&mut self.store, (len, 1))?;

        let mem = self.memory.data_mut(&mut self.store);
        let range = ptr as usize .. (ptr + len) as usize;
        mem[range].copy_from_slice(bytes);

        Ok((ptr, len))
    }

    /// Solves a challenge, returning the base64-encoded response.
    pub fn solve_challenge(&mut self, challenge: Challenge) -> Result<String> {
        let prefix = format!("{}_{}_", challenge.salt, challenge.expire_at);
        let out_ptr = self.add_stack.call(&mut self.store, (-16,))?;

        let (challenge_ptr, challenge_len) = self.write_str_to_memory(&challenge.challenge)?;
        let (prefix_ptr, prefix_len) = self.write_str_to_memory(&prefix)?;

        self.wasm_solve.call(
            &mut self.store,
            (out_ptr, challenge_ptr, challenge_len, prefix_ptr, prefix_len, challenge.difficulty),
        )?;

        // Read status (first 4 bytes) and answer (bytes 8-16)
        let mem = self.memory.data(&self.store);
        let status = i32::from_le_bytes(mem[out_ptr as usize..(out_ptr+4) as usize].try_into()?);
        if status == 0 {
            // Restore stack pointer before bailing
            self.add_stack.call(&mut self.store, (16,))?;
            anyhow::bail!("WASM solve returned status 0 (failure)");
        }

        let answer_bytes: [u8; 8] = mem[(out_ptr+8) as usize..(out_ptr+16) as usize].try_into()?;
        let answer = f64::from_le_bytes(answer_bytes);

        // Cleanup stack
        self.add_stack.call(&mut self.store, (16,))?;

        let response = SolveResponse {
            algorithm: challenge.algorithm,
            challenge: challenge.challenge,
            salt: challenge.salt,
            answer: answer as i64,
            signature: challenge.signature,
            target_path: challenge.target_path,
        };

        let json_string = serde_json::to_string(&response)?;
        Ok(BASE64.encode(json_string))
    }
}