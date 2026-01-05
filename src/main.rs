#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(type_alias_impl_trait)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use blog_os::println;
use bootloader::{BootInfo, entry_point};

use blog_os::memory;

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

// === Entry point ===
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::memory::BootInfoFrameAllocator;
    use blog_os::task::keyboard;
    use blog_os::task::{Task, executor::Executor};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    blog_os::init();

    // === Initialize mapper using OffsetPageTable ===
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // === Initialize heap ===
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    // === Run async task through our executor ===
    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    // === Run tests in test mode ===
    #[cfg(test)]
    test_main();
}

// === Panic handler (non-test) ===
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

// === Panic handler for test builds ===
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

