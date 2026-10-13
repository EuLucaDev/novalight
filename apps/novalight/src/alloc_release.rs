//! Give memory the system allocator keeps after frees back to the system
//! ([`novalight_engine::memory::set_release_hook`]). The platform call lives in
//! `novalight-sysmem`, the workspace's only crate allowed `unsafe`.

/// Install the hook.
pub fn install() {
    novalight_engine::memory::set_release_hook(release);
}

fn release() {
    let _ = novalight_sysmem::release_free_memory();
}
