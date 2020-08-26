use std::{
    convert::TryInto,
    error::Error,
    fmt::Write,
    future::Future,
    io,
    path::Path,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};
use tempdir::TempDir;
use tokio::{io::AsyncRead, process::Child};

use super::{DebugProbe, DynAsyncRead};
use crate::subprocess;

#[derive(thiserror::Error, Debug)]
enum Fe310JLinkDebugProbeGetOutputError {
    #[error("{0}")]
    ProcessElf(#[source] ProcessElfError),
    #[error("Error while creating a temporary directory: {0}")]
    CreateTempDir(#[source] std::io::Error),
    #[error("Error while creating a temporary file: {0}")]
    CreateTempFile(#[source] std::io::Error),
    #[error("Error while flashing the device: {0}")]
    Flash(#[source] subprocess::SubprocessError),
    #[error("Error while opening the probe: {0}")]
    OpenProbe(#[source] probe_rs::DebugProbeError),
    #[error("Error while attaching to the probe: {0}")]
    Attach(#[source] probe_rs::Error),
}

pub(super) struct Fe310JLinkDebugProbe {}

impl Fe310JLinkDebugProbe {
    pub(super) fn new() -> Self {
        Self {}
    }
}

impl DebugProbe for Fe310JLinkDebugProbe {
    fn program_and_get_output(
        &mut self,
        exe: &Path,
    ) -> Pin<Box<dyn Future<Output = Result<DynAsyncRead<'_>, Box<dyn Error>>> + '_>> {
        let exe = exe.to_owned();
        Box::pin(async move {
            // Extract loadable sections
            let sections = read_loaded_region_from_elf(&exe)
                .await
                .map_err(Fe310JLinkDebugProbeGetOutputError::ProcessElf)?;

            // Extract loadable sections to separate binary files
            let tempdir = TempDir::new("constance_test_runner")
                .map_err(Fe310JLinkDebugProbeGetOutputError::CreateTempDir)?;
            let section_files: Vec<_> = (0..sections.len())
                .map(|i| {
                    let name = format!("{}.bin", i);
                    tempdir.path().join(&name)
                })
                .collect();
            for (path, (data, _)) in section_files.iter().zip(sections.iter()) {
                log::debug!("Writing {} byte(s) to '{}'", data.len(), path.display());
                tokio::fs::write(&path, data)
                    .await
                    .map_err(Fe310JLinkDebugProbeGetOutputError::CreateTempFile)?;
            }

            // Generate commands for `JLinkExe`
            let mut cmd = String::new();
            for (path, (_, offset)) in section_files.iter().zip(sections.iter()) {
                writeln!(cmd, "loadbin \"{}\" 0x{:08x}", path.display(), offset).unwrap();
            }
            writeln!(cmd, "rnh").unwrap();
            writeln!(cmd, "q").unwrap();

            // Flash the program and reset the chip
            // (`probe-rs` doesn't support FE310-based boards at this time)
            log::debug!("Launching JLinkExe and executing '{:?}'", cmd);
            subprocess::CmdBuilder::new("JLinkExe")
                .args(&[
                    "-device",
                    "FE310",
                    "-if",
                    "JTAG",
                    "-speed",
                    "4000",
                    "-jtagconf",
                    "-1,-1",
                    "-autoconnect",
                    "1",
                    "-exitonerror",
                    "1",
                    "-nogui",
                    "1",
                ])
                .spawn_expecting_success_quiet_with_input(cmd.as_bytes())
                .await
                .map_err(Fe310JLinkDebugProbeGetOutputError::Flash)?;

            // Open the probe using `probe-rs`
            // (Actually, `JLinkExe` comes with their RTT host, but I'm too lazy
            // to check its usage)
            // TODO: Use the J-Link software for RTT connection
            let selector: probe_rs::DebugProbeSelector = "1366:1061".try_into().unwrap();
            let probe = probe_rs::Probe::open(selector)
                .map_err(Fe310JLinkDebugProbeGetOutputError::OpenProbe)?;

            let selector: probe_rs::config::TargetSelector = "riscv".try_into().unwrap();
            let session = Arc::new(Mutex::new(
                probe
                    .attach(selector)
                    .map_err(Fe310JLinkDebugProbeGetOutputError::Attach)?,
            ));

            // Open the RTT channels
            Ok(super::probe_rs::attach_rtt(session, &exe).await?)
        })
    }
}

#[derive(thiserror::Error, Debug)]
enum ProcessElfError {
    #[error("Couldn't read the ELF file: {0}")]
    Read(#[source] std::io::Error),
    #[error("Couldn't parse the ELF file: {0}")]
    Parse(#[source] goblin::error::Error),
}

/// Read the specified ELF file and return regions to be loaded onto the target.
async fn read_loaded_region_from_elf(exe: &Path) -> Result<Vec<(Vec<u8>, u64)>, ProcessElfError> {
    let elf_bytes = tokio::fs::read(&exe).await.map_err(ProcessElfError::Read)?;
    let elf = goblin::elf::Elf::parse(&elf_bytes).map_err(ProcessElfError::Parse)?;

    let regions = elf
        .program_headers
        .iter()
        .filter_map(|ph| {
            if ph.p_type == goblin::elf32::program_header::PT_LOAD && ph.p_filesz > 0 {
                Some((
                    elf_bytes[ph.p_offset as usize..][..ph.p_filesz as usize].to_vec(),
                    ph.p_paddr,
                ))
            } else {
                None
            }
        })
        .collect();

    Ok(regions)
}

struct OutputReader {
    child: Child,
}

impl AsyncRead for OutputReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(self.child.stdout.as_mut().unwrap()).poll_read(cx, buf)
    }
}
