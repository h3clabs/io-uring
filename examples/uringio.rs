use std::{fs, io, os::unix::io::AsRawFd};

use io_uring::{
    platform::iouring::IoUringUserData,
    uringio::{
        operator::{fs::Read, nop::Nop},
        ring::{
            fd::RingFd,
            mode::{Iopoll, Sqpoll},
            UringIo, UringMix,
        },
        submission::{entry::Sqe64, submitter::Submit},
    },
};

fn main() -> io::Result<()> {
    let fd = Sqpoll::new_args(128).setup()?;

    let mut uring = UringIo::new(&rd.arena, &params)?;

    println!("fd: {:#?}", rd);
    println!("uring: {:#?}", uring);

    let file = fs::File::open("README.md")?;
    let mut dst = vec![0; 1024];
    let mut read = Read::new(file, &mut dst);
    read.user_data = IoUringUserData::from(0x42);
    println!("Read: {:#?}", read);

    let (mut submitter, mut collector) = uring.borrow();
    let res = submitter.push(read);
    println!("res: {:#?}", res);
    submitter.update();

    loop {
        println!("submitter {:?}", submitter);
        println!("collector {:?}", collector);

        while let Some(cqe) = collector.next() {
            println!("== cqe ==: {:#?}", cqe);
        }

        while let Err(sqe) = submitter.push(Nop::new()) {
            println!("submit error: {:#?}", sqe);
            continue;
        }

        std::thread::sleep(std::time::Duration::from_millis(500));
        collector.update();

        // break;
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
