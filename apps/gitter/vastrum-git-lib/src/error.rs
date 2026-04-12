#[derive(thiserror::Error, Debug)]
pub enum VastrumGitError {
    #[error("push rejected: local and remote have diverged")]
    Diverged,
    #[error("submodules are not supported")]
    SubmodulesNotSupported,
    #[error("repository not found: {0}")]
    RepoNotFound(String),
    #[error("object not found: {0}")]
    ObjectNotFound(String),
    #[error("repository does not have a head commit yet")]
    RepoDoesNotHaveHeadCommitYet,
    #[error("local repository has detached HEAD; check out a branch before pushing")]
    DetachedHead,
    #[error("transaction confirmation failed: {0}")]
    TxConfirmation(String),
    #[error("object {oid} is too large to upload: {size} bytes (max {max} bytes)")]
    ObjectTooLarge { oid: String, size: usize, max: usize },

    #[error(transparent)]
    GitDecode(#[from] gix_object::decode::Error),

    #[error("failed to decode loose object: {0}")]
    LooseDecode(String),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("checkout failed: {0}")]
    Checkout(String),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    GitOpen(#[from] gix::open::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    GitInit(#[from] gix::init::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    GitFind(#[from] gix::object::find::existing::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    GitFindConverted(#[from] gix::object::find::existing::with_conversion::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    GitWrite(#[from] gix::object::write::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    GitRefEdit(#[from] gix::reference::edit::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    GitHeadId(#[from] gix::reference::head_id::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    GitRefFindExisting(#[from] gix::reference::find::existing::Error),
}

pub type Result<T> = std::result::Result<T, VastrumGitError>;
