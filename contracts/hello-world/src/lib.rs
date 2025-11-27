#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, String, Symbol, symbol_short,
};

// File metadata structure
#[contracttype]
#[derive(Clone)]
pub struct FileRecord {
    pub file_id: u64,
    pub owner: Address,
    pub file_name: String,
    pub file_hash: String,      // IPFS hash or content hash for verification
    pub file_size: u64,         // Size in bytes
    pub upload_time: u64,
    pub is_public: bool,
    pub download_count: u64,
}

// Access permission structure for shared files
#[contracttype]
#[derive(Clone)]
pub struct SharePermission {
    pub permission_id: u64,
    pub file_id: u64,
    pub owner: Address,
    pub shared_with: Address,
    pub can_download: bool,
    pub expiry_time: u64,       // 0 means no expiry
    pub granted_time: u64,
}

// Platform statistics
#[contracttype]
#[derive(Clone)]
pub struct SyncStats {
    pub total_files: u64,
    pub total_shares: u64,
    pub total_downloads: u64,
    pub active_users: u64,
}

const SYNC_STATS: Symbol = symbol_short!("S_STATS");
const FILE_COUNT: Symbol = symbol_short!("F_COUNT");
const SHARE_COUNT: Symbol = symbol_short!("SH_COUNT");

#[contracttype]
pub enum FileBook {
    File(u64)
}

#[contracttype]
pub enum ShareBook {
    Share(u64)
}

#[contracttype]
pub enum UserFiles {
    Count(Address)
}

#[contract]
pub struct StreamSyncContract;

#[contractimpl]
impl StreamSyncContract {

    /// Register a new file upload on the blockchain
    /// Returns the unique file_id
    pub fn upload_file(
        env: Env,
        owner: Address,
        file_name: String,
        file_hash: String,
        file_size: u64,
        is_public: bool
    ) -> u64 {
        owner.require_auth();

        let mut file_count: u64 = env.storage().instance().get(&FILE_COUNT).unwrap_or(0);
        file_count += 1;

        let time = env.ledger().timestamp();

        let file_record = FileRecord {
            file_id: file_count,
            owner: owner.clone(),
            file_name,
            file_hash,
            file_size,
            upload_time: time,
            is_public,
            download_count: 0,
        };

        // Update user's file count
        let user_key = UserFiles::Count(owner.clone());
        let mut user_count: u64 = env.storage().instance().get(&user_key).unwrap_or(0);
        user_count += 1;
        env.storage().instance().set(&user_key, &user_count);

        // Update platform stats
        let mut stats = Self::get_sync_stats(env.clone());
        stats.total_files += 1;

        env.storage().instance().set(&FileBook::File(file_count), &file_record);
        env.storage().instance().set(&SYNC_STATS, &stats);
        env.storage().instance().set(&FILE_COUNT, &file_count);

        env.storage().instance().extend_ttl(5000, 5000);

        file_count
    }

    /// Share a file with another user
    /// Returns the permission_id
    pub fn share_file(
        env: Env,
        owner: Address,
        file_id: u64,
        shared_with: Address,
        expiry_time: u64
    ) -> u64 {
        owner.require_auth();

        let file = Self::get_file(env.clone(), file_id);

        if file.file_id == 0 {
            panic!("File not found");
        }

        let mut share_count: u64 = env.storage().instance().get(&SHARE_COUNT).unwrap_or(0);
        share_count += 1;

        let time = env.ledger().timestamp();

        let permission = SharePermission {
            permission_id: share_count,
            file_id,
            owner: owner.clone(),
            shared_with: shared_with.clone(),
            can_download: true,
            expiry_time,
            granted_time: time,
        };

        let mut stats = Self::get_sync_stats(env.clone());
        stats.total_shares += 1;

        env.storage().instance().set(&ShareBook::Share(share_count), &permission);
        env.storage().instance().set(&SYNC_STATS, &stats);
        env.storage().instance().set(&SHARE_COUNT, &share_count);

        env.storage().instance().extend_ttl(5000, 5000);

        share_count
    }

    /// Record a file download (increments download counter)
    pub fn record_download(env: Env, file_id: u64, downloader: Address) {
        downloader.require_auth();

        let mut file = Self::get_file(env.clone(), file_id);

        if file.file_id == 0 {
            panic!("File not found");
        }

        file.download_count += 1;

        let mut stats = Self::get_sync_stats(env.clone());
        stats.total_downloads += 1;

        env.storage().instance().set(&FileBook::File(file_id), &file);
        env.storage().instance().set(&SYNC_STATS, &stats);

        env.storage().instance().extend_ttl(5000, 5000);
    }

    /// Revoke share permission
    pub fn revoke_share(env: Env, owner: Address, permission_id: u64) {
        owner.require_auth();

        let mut permission = Self::get_share(env.clone(), permission_id);

        if permission.permission_id == 0 {
            panic!("Permission not found");
        }

        permission.can_download = false;

        let mut stats = Self::get_sync_stats(env.clone());
        if stats.total_shares > 0 {
            stats.total_shares -= 1;
        }

        env.storage().instance().set(&ShareBook::Share(permission_id), &permission);
        env.storage().instance().set(&SYNC_STATS, &stats);

        env.storage().instance().extend_ttl(5000, 5000);
    }

    /// Get file details
    pub fn get_file(env: Env, file_id: u64) -> FileRecord {
        let key = FileBook::File(file_id);

        env.storage().instance().get(&key).unwrap_or(FileRecord {
            file_id: 0,
            owner: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            file_name: String::from_str(&env, "Not_Found"),
            file_hash: String::from_str(&env, ""),
            file_size: 0,
            upload_time: 0,
            is_public: false,
            download_count: 0,
        })
    }

    /// Get share permission details
    pub fn get_share(env: Env, permission_id: u64) -> SharePermission {
        let key = ShareBook::Share(permission_id);

        env.storage().instance().get(&key).unwrap_or(SharePermission {
            permission_id: 0,
            file_id: 0,
            owner: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            shared_with: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            can_download: false,
            expiry_time: 0,
            granted_time: 0,
        })
    }

    /// Get user's file count
    pub fn get_user_file_count(env: Env, owner: Address) -> u64 {
        let key = UserFiles::Count(owner);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    /// Get platform statistics
    pub fn get_sync_stats(env: Env) -> SyncStats {
        env.storage().instance().get(&SYNC_STATS).unwrap_or(SyncStats {
            total_files: 0,
            total_shares: 0,
            total_downloads: 0,
            active_users: 0,
        })
    }
}