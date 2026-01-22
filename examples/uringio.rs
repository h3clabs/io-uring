use std::{
    fs::{File, OpenOptions},
    io,
    os::unix::{fs::OpenOptionsExt, io::AsRawFd},
};

use io_uring::{
    platform::iouring::IoUringUserData,
    uringio::{
        operator::{fs::Read, nop::Nop},
        submission::{entry::Sqe64, submitter::Submit},
        uring::{
            enter::UringEnter,
            mode::{Iopoll, Sqpoll},
            UringIo, UringMix,
        },
    },
};

fn main() -> io::Result<()> {
    let (fd, args) = Sqpoll::new(128).setup()?;

    let mut uring = UringIo::new(&fd, &args)?.register()?;

    println!("fd: {:#?}", fd);
    println!("args: {:#?}", args);
    println!("uring: {:#?}", uring);

    let file = File::open("README.md")?;

    let mut dst = vec![0; 1024];
    let mut read = Read::new(&file, &mut dst);
    read.user_data = IoUringUserData::from(0x42);
    println!("Read: {:#?}", read);

    let (enter, mut submitter, mut collector) = uring.borrow();

    if let Err(sqe) = submitter.push(read) {
        panic!("submission queue is full");
    }

    loop {
        println!("rerun");

        submitter.submit();

        collector.update();
        if let Some(cqe) = collector.next() {
            println!("== cqe ==: {:?}", cqe);
            println!("== dst ==: {:#?}", str::from_utf8(&dst).unwrap());
            break;
        } else {
            println!("flush collector");
            collector.flush(enter, 1)?;
        }

        // std::thread::sleep(std::time::Duration::from_millis(1500));
    }

    Ok(())
}
