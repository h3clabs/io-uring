use std::{fs, io, os::unix::io::AsRawFd};

use io_uring::uringio::{
    operator::fs::read::Read,
    submission::submitter::Submit,
    uring::{
        mode::{Iopoll, Sqpoll},
        UringFd, UringIo, UringMix,
    },
};

fn main() -> io::Result<()> {
    let fd = Iopoll::new_args(128).setup()?;

    let mut uring = UringIo::new(&fd)?;

    println!("fd: {:#?}", fd);
    println!("uring: {:#?}", uring);

    let file = fs::File::open("README.md")?;
    let mut dst = vec![0; 1024];
    let read = Read::new(file, &mut dst);
    println!("Read: {:#?}", read);

    uring.sq.submit(read)?;

    loop {
        std::hint::spin_loop()
    }

    // let mut buf = vec![0; 1024];

    // let read_e = opcode::Read::new(types::Fd(fd.as_raw_fd()), buf.as_mut_ptr(), buf.len() as _)
    //     .build()
    //     .user_data(0x42);

    // // Note that the developer needs to ensure
    // // that the entry pushed into submission queue is valid (e.g. fd, buffer).
    // unsafe {
    //     ring.submission().push(&read_e).expect("submission queue is full");
    // }

    // ring.submit_and_wait(1)?;

    // let cqe = ring.completion().next().expect("completion queue is empty");

    // assert_eq!(cqe.user_data(), 0x42);
    // assert!(cqe.result() >= 0, "read error: {}", cqe.result());

    Ok(())
}
