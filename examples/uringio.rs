use std::{fs, io, os::unix::io::AsRawFd};

use io_uring::{
    platform::iouring::IoUringUserData,
    uringio::{
        operator::{fs::Read, nop::Nop},
        submission::{entry::Sqe64, submitter::Submit},
        uring::{
            enter::UringHandler,
            mode::{Iopoll, Sqpoll},
            UringIo, UringMix,
        },
    },
};

fn main() -> io::Result<()> {
    let (fd, args) = Sqpoll::new(128).setup()?;

    let mut uring = UringIo::new(&fd, &args)?;

    println!("fd: {:#?}", fd);
    println!("uring: {:#?}", uring);

    let file = fs::File::open("README.md")?;
    let mut dst = vec![0; 1024];
    let mut read = Read::new(&file, &mut dst);
    read.user_data = IoUringUserData::from(0x42);
    println!("Read: {:#?}", read);

    let (entry, mut submitter, mut collector) = uring.borrow();
    if let Err(sqe) = submitter.push(read) {
        panic!("submission queue is full");
    }

    loop {
        // println!("submitter {:?}", submitter);
        // println!("collector {:?}", collector);

        collector.update();
        if let Some(cqe) = collector.next() {
            println!("== cqe ==: {:#?}", cqe);
            println!("== dst ==: {:#?}", str::from_utf8(&dst).unwrap());
            break;
        }

        // let _ = submitter.push(Nop::new());
        submitter.update();
        let n = entry.submit(&mut submitter, 0)?;
        println!("submitted: {n}");

        // std::thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(())
}
