use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd;

pub fn suppress_output<F, T>(f: F, should_suppress: bool) -> T
where
    F: FnOnce() -> T,
{
    if should_suppress {
        let dev_null = File::open("/dev/null").expect("Failed to open /dev/null");
        let null_fd = dev_null.as_raw_fd();

        // Backup stdout and stderr using `dup`
        let stdout_backup = unsafe { libc::dup(io::stdout().as_raw_fd()) };
        let stderr_backup = unsafe { libc::dup(io::stderr().as_raw_fd()) };

        if stdout_backup < 0 || stderr_backup < 0 {
            panic!("Failed to backup stdout or stderr");
        }

        // Redirect stdout and stderr to /dev/null
        unsafe {
            libc::dup2(null_fd, io::stdout().as_raw_fd());
            libc::dup2(null_fd, io::stderr().as_raw_fd());
        }

        let result = f(); // Run the closure

        // Restore original stdout and stderr
        unsafe {
            libc::dup2(stdout_backup, io::stdout().as_raw_fd());
            libc::dup2(stderr_backup, io::stderr().as_raw_fd());
            libc::close(stdout_backup); // Close the backup descriptors
            libc::close(stderr_backup);
        }

        result
    } else {
        f() // Execute without suppressing output
    }
}
