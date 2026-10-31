//! macOS observations and resources owned by one supervised worker.
use anyhow::{ensure, Context, Result};
use std::{
    collections::HashMap,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

unsafe extern "C" {
    fn mach_port_deallocate(
        task: libc::mach_port_t,
        name: libc::mach_port_t,
    ) -> libc::kern_return_t;
}

fn bsd_info(pid: u32) -> Option<libc::proc_bsdinfo> {
    if pid == 0 || pid > i32::MAX as u32 {
        return None;
    }
    let mut value = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as i32;
    let read = unsafe {
        libc::proc_pidinfo(
            pid as i32,
            libc::PROC_PIDTBSDINFO,
            0,
            value.as_mut_ptr().cast(),
            size,
        )
    };
    (read == size).then(|| unsafe { value.assume_init() })
}

pub fn start_time(pid: u32) -> Option<u64> {
    let info = bsd_info(pid)?;
    info.pbi_start_tvsec
        .checked_mul(1_000_000)?
        .checked_add(info.pbi_start_tvusec)
}

pub fn owned_group(pid: u32, expected: Option<u64>) -> bool {
    let Some(info) = bsd_info(pid) else {
        return false;
    };
    let time = info
        .pbi_start_tvsec
        .checked_mul(1_000_000)
        .and_then(|s| s.checked_add(info.pbi_start_tvusec));
    info.pbi_pid == pid
        && info.pbi_ppid == std::process::id()
        && info.pbi_pgid == pid
        && expected.is_some()
        && time == expected
}

/// Enumerate live descendants while the unreaped group leader pins its ID.
/// PROC_PGRP_ONLY is defined by the macOS SDK but not exported by libc.
pub fn group_members(pgid: i32) -> Option<Vec<(i32, String)>> {
    if pgid <= 0 {
        return None;
    }
    const PROC_PGRP_ONLY: u32 = 2;
    let bytes =
        unsafe { libc::proc_listpids(PROC_PGRP_ONLY, pgid as u32, std::ptr::null_mut(), 0) };
    if bytes < 0 {
        return None;
    }
    let mut capacity = (bytes as usize / std::mem::size_of::<i32>() + 32).max(64);
    for _ in 0..3 {
        if capacity > 262_144 {
            return None;
        }
        let mut pids = vec![0i32; capacity];
        let size = (pids.len() * std::mem::size_of::<i32>()) as i32;
        let read = unsafe {
            libc::proc_listpids(PROC_PGRP_ONLY, pgid as u32, pids.as_mut_ptr().cast(), size)
        };
        if read < 0 || read as usize % std::mem::size_of::<i32>() != 0 {
            return None;
        }
        if read >= size {
            capacity *= 2;
            continue;
        }
        let mut members = Vec::new();
        for pid in pids
            .into_iter()
            .take(read as usize / std::mem::size_of::<i32>())
        {
            if pid <= 0 || pid == pgid {
                continue;
            }
            let Some(info) = bsd_info(pid as u32) else {
                // Darwin can drop BSD info before a dying process disappears
                // from the group. Conservatively retain the kernel-enumerated
                // member until it is gone; only the owned group is signalled.
                let exists = unsafe { libc::kill(pid, 0) } == 0
                    || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH);
                if exists {
                    members.push((pid, "<identity unavailable>".into()));
                }
                continue;
            };
            if info.pbi_pgid != pgid as u32 || info.pbi_status == libc::SZOMB as u32 {
                continue;
            }
            let name: Vec<u8> = info
                .pbi_comm
                .iter()
                .take_while(|c| **c != 0)
                .map(|c| *c as u8)
                .collect();
            members.push((pid, String::from_utf8_lossy(&name).into_owned()));
        }
        members.sort();
        return Some(members);
    }
    None
}

pub fn resident_bytes(pid: u32) -> Option<u64> {
    let mut value = std::mem::MaybeUninit::<libc::proc_taskinfo>::uninit();
    let size = std::mem::size_of::<libc::proc_taskinfo>() as i32;
    let read = unsafe {
        libc::proc_pidinfo(
            pid as i32,
            libc::PROC_PIDTASKINFO,
            0,
            value.as_mut_ptr().cast(),
            size,
        )
    };
    (read == size).then(|| unsafe { value.assume_init().pti_resident_size })
}

pub fn total_bytes() -> Option<u64> {
    let mut value = 0u64;
    let mut size = std::mem::size_of::<u64>();
    let status = unsafe {
        libc::sysctlbyname(
            c"hw.memsize".as_ptr(),
            (&mut value as *mut u64).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    (status == 0 && size == std::mem::size_of::<u64>()).then_some(value)
}

pub fn available_bytes() -> Option<u64> {
    let mut value = std::mem::MaybeUninit::<libc::vm_statistics64>::uninit();
    let mut count = libc::HOST_VM_INFO64_COUNT;
    let host = unsafe { libc::mach_host_self() };
    let status = unsafe {
        libc::host_statistics64(
            host,
            libc::HOST_VM_INFO64,
            value.as_mut_ptr().cast(),
            &mut count,
        )
    };
    unsafe {
        mach_port_deallocate(libc::mach_task_self(), host);
    }
    if status != libc::KERN_SUCCESS || count != libc::HOST_VM_INFO64_COUNT {
        return None;
    }
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if page_size <= 0 {
        return None;
    }
    let stats = unsafe { value.assume_init() };
    // An admission estimate, NOT a guarantee of immediately allocatable RAM.
    // Bare free pages exclude the large filesystem cache after builds. Credit
    // only half the smaller of file-backed and inactive pages; do not add both
    // counters (they overlap), or credit compressed/anonymous pages separately.
    // A further 16 GiB admission reserve and sampled RSS/pressure floor apply.
    let reclaim_estimate = stats.external_page_count.min(stats.inactive_count) as u64 / 2;
    Some((stats.free_count as u64 + reclaim_estimate) * page_size as u64)
}

pub struct MemoryGuard {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl MemoryGuard {
    pub fn new(pid: u32) -> Result<Self> {
        let started = start_time(pid).context("cannot establish worker process identity")?;
        ensure!(
            owned_group(pid, Some(started)),
            "worker must be our process-group leader"
        );
        ensure!(
            available_bytes().is_some(),
            "cannot observe macOS free memory"
        );
        let stop = Arc::new(AtomicBool::new(false));
        let cancel = stop.clone();
        let thread = std::thread::Builder::new().name("macos-memory-watch".into()).spawn(move || {
            while !cancel.load(Ordering::Acquire) {
                if !owned_group(pid, Some(started)) { return; }
                // A zombie may retain BSD identity after task info disappears.
                // Its already-buffered handshake error must remain readable.
                if bsd_info(pid).is_some_and(|info| info.pbi_status == libc::SZOMB as u32) { return; }
                let rss = resident_bytes(pid);
                let available = available_bytes();
                let safe = rss.is_some_and(|n| n <= 32 * 1024u64.pow(3))
                    && available.is_some_and(|n| n >= 8 * 1024u64.pow(3));
                if !safe {
                    tracing::error!("macOS worker {pid}: RSS={rss:?}, estimated available={available:?}; resource observation unavailable or limit reached");
                    if owned_group(pid, Some(started)) { unsafe { libc::kill(-(pid as i32), libc::SIGKILL); } }
                    return;
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        })?;
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub struct HelperLease {
    context: String,
    name: String,
    directory: PathBuf,
}

impl HelperLease {
    pub fn configure(
        backend: &str,
        command: &mut Command,
        overrides: &HashMap<String, String>,
    ) -> Result<Option<Self>> {
        if backend.split(':').next() != Some("risc0")
            || std::path::Path::new(command.get_program())
                .file_name()
                .and_then(|s| s.to_str())
                != Some("zkminer-prove-risc0")
        {
            return Ok(None);
        }
        let setting = |key: &str| {
            overrides
                .get(key)
                .cloned()
                .or_else(|| std::env::var(key).ok())
                .with_context(|| format!("macOS worker requires {key}"))
        };
        let context = setting("ZKMINER_RISC0_DOCKER_CONTEXT")?;
        let base = PathBuf::from(setting("ZKMINER_RISC0_WORK_ROOT")?).canonicalize()?;
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "zkminer-macos-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let directory = base.join(&name);
        std::fs::create_dir(&directory)?;
        command
            .env("RISC0_GROTH16_CONTAINER_NAME", &name)
            .env("RISC0_WORK_DIR", &directory)
            .env("ZKMINER_RISC0_DOCKER_CONTEXT", &context);
        Ok(Some(Self {
            context,
            name,
            directory,
        }))
    }
}

impl Drop for HelperLease {
    fn drop(&mut self) {
        let status = zkminer_prover_protocol::proc::output_with_timeout(
            Command::new("docker").args(["--context", &self.context, "rm", "-f", &self.name]),
            Duration::from_secs(10),
            16 * 1024,
        );
        tracing::debug!("owned helper {} cleanup: {}", self.name, status.describe());
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::CommandExt;

    #[test]
    fn observations_and_process_identity() {
        assert!(total_bytes().unwrap() > 0);
        assert!(available_bytes().unwrap() <= total_bytes().unwrap());
        assert!(resident_bytes(std::process::id()).unwrap() > 0);
        assert!(!owned_group(
            std::process::id(),
            start_time(std::process::id())
        ));
        let mut child = Command::new("sleep")
            .arg("30")
            .process_group(0)
            .spawn()
            .unwrap();
        let pid = child.id();
        let time = start_time(pid).unwrap();
        assert!(owned_group(pid, Some(time)));
        assert!(!owned_group(pid, Some(time + 1)));
        assert!(!owned_group(pid, None));
        assert!(group_members(pid as i32).unwrap().is_empty());
        child.kill().unwrap();
        child.wait().unwrap();
        assert!(!owned_group(pid, Some(time)));
    }

    #[test]
    fn native_group_scan_finds_and_cleans_owned_descendants() {
        let mut child = Command::new("sh")
            .args(["-c", "sleep 30 & wait"])
            .process_group(0)
            .spawn()
            .unwrap();
        let pid = child.id();
        let started = start_time(pid).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        let members = loop {
            let members = group_members(pid as i32).unwrap();
            if !members.is_empty() || std::time::Instant::now() > deadline {
                break members;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(owned_group(pid, Some(started)));
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
        child.wait().unwrap();
        assert!(
            !members.is_empty(),
            "native scan missed the owned sleep process"
        );
    }
}
