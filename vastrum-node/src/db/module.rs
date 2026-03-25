fn module_file_path(dir: &std::path::Path, hash: Sha256Digest) -> PathBuf {
    dir.join(format!("{:?}.cwasm", hash))
}

impl Db {
    pub fn write_module(&self, compiled_module: CompiledModule) {
        let dir = self.compiled_modules_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = module_file_path(&dir, compiled_module.key);
        std::fs::write(&path, &compiled_module.data).unwrap();
    }

    pub fn module_file_path(&self, wasm_hash: Sha256Digest) -> PathBuf {
        module_file_path(&self.compiled_modules_dir(), wasm_hash)
    }
}

impl BatchDb {
    pub fn write_module(&self, compiled_module: CompiledModule) {
        let dir = self.db.compiled_modules_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let path = module_file_path(&dir, compiled_module.key);
        std::fs::write(&path, &compiled_module.data).unwrap();
    }

    pub fn calculate_module_file_path(&self, wasm_hash: Sha256Digest) -> PathBuf {
        module_file_path(&self.db.compiled_modules_dir(), wasm_hash)
    }
}

use super::{BatchDb, Db};
use crate::execution::types::compiled_module::CompiledModule;
use vastrum_shared_types::crypto::sha256::Sha256Digest;
use std::path::PathBuf;
