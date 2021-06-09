//! Module for everything related to serial I/O.
use std::fs;
use std::os::unix::io::AsRawFd;

use anyhow::{Context, Result};
use nix::{
    fcntl::{self, FcntlArg, OFlag},
    libc,
    sys::termios::{
        self, BaudRate, ControlFlags, InputFlags, LocalFlags, OutputFlags, SetArg,
        SpecialCharacterIndices as CC,
    },
};

mod ioctl {
    use super::libc;
    use nix::{ioctl_none_bad, ioctl_read_bad, ioctl_write_int_bad, ioctl_write_ptr_bad};

    ioctl_none_bad!(tiocexcl, libc::TIOCEXCL);
    ioctl_read_bad!(tiocmget, libc::TIOCMGET, libc::c_int);
    ioctl_write_ptr_bad!(tiocmset, libc::TIOCMSET, libc::c_int);
    ioctl_write_int_bad!(tcflsh, libc::TCFLSH);
}

/// Opens and configures the given `device`.
///
/// Effectively mirrors the behavior of
/// ```
/// $ screen /dev/ttyUSB3 115200
/// ```
/// assuming that `device` is `/dev/ttyUSB3`.
pub fn configure(device: String) -> Result<fs::File> {
    let file = fs::OpenOptions::new()
        .read(true)
        .open(&device)
        .with_context(|| format!("Unable to open {}", device))?;

    unsafe {
        let fd = file.as_raw_fd();

        // Enable exclusive mode. Any further open(2) will fail with EBUSY.
        ioctl::tiocexcl(fd)
            .with_context(|| format!("Failed to put {} into exclusive mode", device))?;

        let mut settings = termios::tcgetattr(fd)
            .with_context(|| format!("Failed to read terminal settings of {}", device))?;

        settings.input_flags |= InputFlags::BRKINT | InputFlags::IGNPAR | InputFlags::IXON;
        settings.input_flags &= !(InputFlags::ICRNL
            | InputFlags::IGNBRK
            | InputFlags::PARMRK
            | InputFlags::INPCK
            | InputFlags::ISTRIP
            | InputFlags::INLCR
            | InputFlags::IGNCR
            | InputFlags::ICRNL
            | InputFlags::IXOFF
            | InputFlags::IXANY
            | InputFlags::IMAXBEL
            | InputFlags::IUTF8);

        settings.output_flags |= OutputFlags::NL0
            | OutputFlags::CR0
            | OutputFlags::TAB0
            | OutputFlags::BS0
            | OutputFlags::VT0
            | OutputFlags::FF0;
        settings.output_flags &= !(OutputFlags::OPOST
            | OutputFlags::ONLCR
            | OutputFlags::OLCUC
            | OutputFlags::OCRNL
            | OutputFlags::ONOCR
            | OutputFlags::ONLRET
            | OutputFlags::OFILL
            | OutputFlags::OFDEL
            | OutputFlags::NL1
            | OutputFlags::CR1
            | OutputFlags::CR2
            | OutputFlags::CR3
            | OutputFlags::TAB1
            | OutputFlags::TAB2
            | OutputFlags::TAB3
            | OutputFlags::XTABS
            | OutputFlags::BS1
            | OutputFlags::VT1
            | OutputFlags::FF1
            | OutputFlags::NLDLY
            | OutputFlags::CRDLY
            | OutputFlags::TABDLY
            | OutputFlags::BSDLY
            | OutputFlags::VTDLY
            | OutputFlags::FFDLY);

        settings.control_flags |= ControlFlags::CS6
            | ControlFlags::CS7
            | ControlFlags::CS8
            | ControlFlags::CREAD
            | ControlFlags::CLOCAL
            | ControlFlags::CBAUDEX // NOTE also via cfsetspeed below
            | ControlFlags::CSIZE;
        settings.control_flags &= !(ControlFlags::HUPCL
            | ControlFlags::CS5
            | ControlFlags::CSTOPB
            | ControlFlags::PARENB
            | ControlFlags::PARODD
            | ControlFlags::CRTSCTS
            | ControlFlags::CBAUD // NOTE also set via cfsetspeed below?
            | ControlFlags::CMSPAR
            | ControlFlags::CIBAUD);

        settings.local_flags |= LocalFlags::ECHOKE
            | LocalFlags::ECHOE
            | LocalFlags::ECHOK
            | LocalFlags::ECHOCTL
            | LocalFlags::IEXTEN;
        settings.local_flags &= !(LocalFlags::ECHO
            | LocalFlags::ISIG
            | LocalFlags::ICANON
            | LocalFlags::ECHONL
            | LocalFlags::ECHOPRT
            | LocalFlags::EXTPROC
            | LocalFlags::TOSTOP
            | LocalFlags::FLUSHO
            | LocalFlags::PENDIN
            | LocalFlags::NOFLSH);

        termios::cfsetspeed(&mut settings, BaudRate::B115200)
            .context("Failed to configure terminal baud rate")?;

        settings.control_chars[CC::VTIME as usize] = 2;
        settings.control_chars[CC::VMIN as usize] = 100;

        // Drain all output, flush all input, and apply settings.
        termios::tcsetattr(fd, SetArg::TCSAFLUSH, &settings)
            .with_context(|| format!("Failed to apply terminal settings to {}", device))?;

        let mut flags: libc::c_int = 0;
        ioctl::tiocmget(fd, &mut flags)
            .with_context(|| format!("Failed to read modem bits of {}", device))?;
        flags |= libc::TIOCM_DTR | libc::TIOCM_RTS;
        ioctl::tiocmset(fd, &flags)
            .with_context(|| format!("Failed to apply modem bits to {}", device))?;

        // Make the tty read-only.
        fcntl::fcntl(fd, FcntlArg::F_SETFL(OFlag::O_RDONLY))
            .with_context(|| format!("Failed to make {} read-only", device))?;

        // Flush all pending I/O, just in case.
        ioctl::tcflsh(fd, libc::TCIOFLUSH)
            .with_context(|| format!("Failed to flush I/O of {}", device))?;
    }

    Ok(file)
}
