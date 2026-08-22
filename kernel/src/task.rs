use alloc::collections::VecDeque;
use core::arch::global_asm;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

global_asm!(
    r#"
    .global novel_context_switch
    .type novel_context_switch, @function
novel_context_switch:
    movq %rsp, 0(%rdi)
    movq %rbp, 8(%rdi)
    movq %rbx, 16(%rdi)
    movq %r12, 24(%rdi)
    movq %r13, 32(%rdi)
    movq %r14, 40(%rdi)
    movq %r15, 48(%rdi)
    leaq .Lresume(%rip), %rax
    movq %rax, 56(%rdi)

    movq 0(%rsi), %rsp
    movq 8(%rsi), %rbp
    movq 16(%rsi), %rbx
    movq 24(%rsi), %r12
    movq 32(%rsi), %r13
    movq 40(%rsi), %r14
    movq 48(%rsi), %r15
    jmp *56(%rsi)
.Lresume:
    ret
    .size novel_context_switch, .-novel_context_switch
"#,
    options(att_syntax)
);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Context {
    rsp: u64,
    rbp: u64,
    rbx: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    rip: u64,
}

unsafe extern "C" {
    fn novel_context_switch(old: *mut Context, new: *const Context);
}

pub unsafe fn switch_context(old: &mut Context, new: &Context) {
    // SAFETY: callers own both contexts, each context has a valid stack and
    // instruction pointer, and the assembly preserves the C ABI callee-saved set.
    unsafe {
        novel_context_switch(old as *mut Context, new as *const Context);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Finished,
}

pub type TaskEntry = fn();

pub struct Task {
    pub id: u64,
    pub state: TaskState,
    entry: TaskEntry,
}

pub struct Scheduler {
    next_id: u64,
    run_queue: VecDeque<Task>,
    completed: u64,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            run_queue: VecDeque::new(),
            completed: 0,
        }
    }

    pub fn spawn(&mut self, entry: TaskEntry) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.run_queue.push_back(Task {
            id,
            state: TaskState::Ready,
            entry,
        });
        id
    }

    pub fn run_one(&mut self) -> Option<u64> {
        let mut task = self.run_queue.pop_front()?;
        task.state = TaskState::Running;
        (task.entry)();
        task.state = TaskState::Finished;
        self.completed += 1;
        Some(task.id)
    }

    pub fn run_until_empty(&mut self) -> u64 {
        while self.run_one().is_some() {}
        self.completed
    }

    pub fn pending(&self) -> usize {
        self.run_queue.len()
    }
}

static TASK_RUNS: AtomicU64 = AtomicU64::new(0);

fn smoke_task() {
    TASK_RUNS.fetch_add(1, Ordering::Relaxed);
}

pub fn scheduler_smoke_test() -> (u64, usize, u64) {
    TASK_RUNS.store(0, Ordering::Relaxed);
    let mut scheduler = Scheduler::new();
    scheduler.spawn(smoke_task);
    scheduler.spawn(smoke_task);
    let completed = scheduler.run_until_empty();
    (
        completed,
        scheduler.pending(),
        TASK_RUNS.load(Ordering::Relaxed),
    )
}

const CONTEXT_STACK_SIZE: usize = 4096;

#[repr(align(16))]
#[derive(Clone, Copy)]
struct ContextStack([u8; CONTEXT_STACK_SIZE]);

static mut CONTEXT_STACK: ContextStack = ContextStack([0; CONTEXT_STACK_SIZE]);
static mut MAIN_CONTEXT: Context = Context {
    rsp: 0,
    rbp: 0,
    rbx: 0,
    r12: 0,
    r13: 0,
    r14: 0,
    r15: 0,
    rip: 0,
};
static mut CHILD_CONTEXT: Context = Context {
    rsp: 0,
    rbp: 0,
    rbx: 0,
    r12: 0,
    r13: 0,
    r14: 0,
    r15: 0,
    rip: 0,
};
static CONTEXT_RUNS: AtomicU64 = AtomicU64::new(0);

fn context_child() {
    CONTEXT_RUNS.fetch_add(1, Ordering::Relaxed);
    // The child never returns through an uninitialized synthetic return address.
    unsafe {
        switch_context(
            &mut *core::ptr::addr_of_mut!(CHILD_CONTEXT),
            &*core::ptr::addr_of!(MAIN_CONTEXT),
        );
    }
}

pub fn context_switch_smoke_test() -> bool {
    CONTEXT_RUNS.store(0, Ordering::Relaxed);
    let stack_bottom = unsafe { core::ptr::addr_of_mut!(CONTEXT_STACK.0) as *mut u8 };
    let stack_top = unsafe { stack_bottom.add(CONTEXT_STACK_SIZE - 8) } as u64;
    unsafe {
        CHILD_CONTEXT = Context {
            rsp: stack_top,
            rip: context_child as *const () as usize as u64,
            ..Context::default()
        };
        switch_context(
            &mut *core::ptr::addr_of_mut!(MAIN_CONTEXT),
            &*core::ptr::addr_of!(CHILD_CONTEXT),
        );
    }
    CONTEXT_RUNS.load(Ordering::Relaxed) == 1
}

const PREEMPT_TASKS: usize = 2;
const PREEMPT_ITERATIONS: u64 = 4;

static mut PREEMPT_MAIN_CONTEXT: Context = Context {
    rsp: 0,
    rbp: 0,
    rbx: 0,
    r12: 0,
    r13: 0,
    r14: 0,
    r15: 0,
    rip: 0,
};
static mut PREEMPT_CONTEXTS: [Context; PREEMPT_TASKS] = [Context {
    rsp: 0,
    rbp: 0,
    rbx: 0,
    r12: 0,
    r13: 0,
    r14: 0,
    r15: 0,
    rip: 0,
}; PREEMPT_TASKS];
static mut PREEMPT_STACKS: [ContextStack; PREEMPT_TASKS] =
    [ContextStack([0; CONTEXT_STACK_SIZE]); PREEMPT_TASKS];
static mut PREEMPT_CURRENT: usize = 0;
static mut PREEMPT_FINISHED: [bool; PREEMPT_TASKS] = [false; PREEMPT_TASKS];
static mut PREEMPT_ACTIVE: bool = false;
static PREEMPT_RUNS: [AtomicU64; PREEMPT_TASKS] = [const { AtomicU64::new(0) }; PREEMPT_TASKS];
static PREEMPT_SWITCHES: AtomicU64 = AtomicU64::new(0);
static PREEMPT_LOCK: AtomicBool = AtomicBool::new(false);

struct PreemptStateGuard;

impl PreemptStateGuard {
    fn acquire() -> Self {
        while PREEMPT_LOCK
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        Self
    }
}

impl Drop for PreemptStateGuard {
    fn drop(&mut self) {
        PREEMPT_LOCK.store(false, Ordering::Release);
    }
}

fn preempt_task_zero() {
    for _ in 0..PREEMPT_ITERATIONS {
        PREEMPT_RUNS[0].fetch_add(1, Ordering::Relaxed);
        x86_64::instructions::interrupts::enable();
        x86_64::instructions::hlt();
    }
    preempt_task_exit(0);
}

fn preempt_task_one() {
    for _ in 0..PREEMPT_ITERATIONS {
        PREEMPT_RUNS[1].fetch_add(1, Ordering::Relaxed);
        x86_64::instructions::interrupts::enable();
        x86_64::instructions::hlt();
    }
    preempt_task_exit(1);
}

fn preempt_task_exit(index: usize) -> ! {
    let (next, all_finished) = {
        let _state_lock = PreemptStateGuard::acquire();
        unsafe {
            PREEMPT_FINISHED[index] = true;
            let next = (index + 1) % PREEMPT_TASKS;
            if PREEMPT_FINISHED[next] {
                PREEMPT_ACTIVE = false;
                (next, true)
            } else {
                PREEMPT_CURRENT = next;
                PREEMPT_SWITCHES.fetch_add(1, Ordering::Relaxed);
                (next, false)
            }
        }
    };
    unsafe {
        if all_finished {
            // No task remains runnable: restore the context that called the
            // smoke test instead of returning through a synthetic stack.
            switch_context(
                &mut *core::ptr::addr_of_mut!(PREEMPT_CONTEXTS[index]),
                &*core::ptr::addr_of!(PREEMPT_MAIN_CONTEXT),
            );
        } else {
            switch_context(
                &mut *core::ptr::addr_of_mut!(PREEMPT_CONTEXTS[index]),
                &*core::ptr::addr_of!(PREEMPT_CONTEXTS[next]),
            );
        }
    }
    loop {
        x86_64::instructions::hlt();
    }
}

/// Called from IRQ0 while the preemptive demo is active. The interrupt
/// handler's stack is part of the current task context; switching here saves
/// that stack and resumes the next task at its own saved handler or entry.
pub fn on_timer_tick() {
    let (current, next) = {
        let _state_lock = PreemptStateGuard::acquire();
        unsafe {
            if !PREEMPT_ACTIVE {
                return;
            }
            let current = PREEMPT_CURRENT;
            let next = (current + 1) % PREEMPT_TASKS;
            if PREEMPT_FINISHED[next] {
                return;
            }
            PREEMPT_CURRENT = next;
            PREEMPT_SWITCHES.fetch_add(1, Ordering::Relaxed);
            (current, next)
        }
    };
    unsafe {
        switch_context(
            &mut *core::ptr::addr_of_mut!(PREEMPT_CONTEXTS[current]),
            &*core::ptr::addr_of!(PREEMPT_CONTEXTS[next]),
        );
    }
}

pub fn preemptive_smoke_test() -> (u64, u64, u64) {
    PREEMPT_RUNS[0].store(0, Ordering::Relaxed);
    PREEMPT_RUNS[1].store(0, Ordering::Relaxed);
    PREEMPT_SWITCHES.store(0, Ordering::Relaxed);
    {
        let _state_lock = PreemptStateGuard::acquire();
        unsafe {
            PREEMPT_FINISHED = [false; PREEMPT_TASKS];
            PREEMPT_CURRENT = 0;
            PREEMPT_ACTIVE = true;
            for index in 0..PREEMPT_TASKS {
                let stack_bottom = core::ptr::addr_of_mut!(PREEMPT_STACKS[index].0) as *mut u8;
                let stack_top = stack_bottom.add(CONTEXT_STACK_SIZE - 16) as u64;
                PREEMPT_CONTEXTS[index] = Context {
                    rsp: stack_top,
                    rip: if index == 0 {
                        preempt_task_zero as *const () as usize as u64
                    } else {
                        preempt_task_one as *const () as usize as u64
                    },
                    ..Context::default()
                };
            }
            // The guard is dropped before changing stacks. No lock state is
            // allowed to survive in a saved task context.
        }
    }
    unsafe {
        switch_context(
            &mut *core::ptr::addr_of_mut!(PREEMPT_MAIN_CONTEXT),
            &*core::ptr::addr_of!(PREEMPT_CONTEXTS[0]),
        );
    }
    let first = PREEMPT_RUNS[0].load(Ordering::Relaxed);
    let second = PREEMPT_RUNS[1].load(Ordering::Relaxed);
    let switches = PREEMPT_SWITCHES.load(Ordering::Relaxed);
    (first, second, switches)
}

use core::sync::atomic::AtomicU8;

const MAX_USER_PROCESSES: usize = 3;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserProcessState {
    Empty = 0,
    Ready = 1,
    Running = 2,
    Exited = 3,
    Faulted = 4,
}

static USER_PROCESS_IDS: [AtomicU64; MAX_USER_PROCESSES] =
    [const { AtomicU64::new(0) }; MAX_USER_PROCESSES];
static USER_PROCESS_STATES: [AtomicU8; MAX_USER_PROCESSES] =
    [const { AtomicU8::new(UserProcessState::Empty as u8) }; MAX_USER_PROCESSES];
static USER_PROCESS_EXIT_STATUS: [AtomicU64; MAX_USER_PROCESSES] =
    [const { AtomicU64::new(0) }; MAX_USER_PROCESSES];
static CURRENT_USER_PROCESS: AtomicU64 = AtomicU64::new(0);

fn user_process_slot(id: u64) -> Option<usize> {
    (0..MAX_USER_PROCESSES).find(|&slot| USER_PROCESS_IDS[slot].load(Ordering::Acquire) == id)
}

/// Register a process control block before its first CPL3 entry.
///
/// This registry is intentionally bounded and interrupt-safe. It is a state
/// table, not yet a runnable user context switcher; the latter requires saving
/// the complete CPU interrupt frame and CR3 at a PIT boundary.
pub fn register_user_process(id: u64) -> bool {
    if id == 0 || user_process_slot(id).is_some() {
        return false;
    }
    for slot in 0..MAX_USER_PROCESSES {
        if USER_PROCESS_IDS[slot]
            .compare_exchange(0, id, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            USER_PROCESS_STATES[slot].store(UserProcessState::Ready as u8, Ordering::Release);
            USER_PROCESS_EXIT_STATUS[slot].store(0, Ordering::Release);
            return true;
        }
    }
    false
}

pub fn mark_user_process_ready(id: u64) -> bool {
    let Some(slot) = user_process_slot(id) else {
        return false;
    };
    let state = &USER_PROCESS_STATES[slot];
    if state.load(Ordering::Acquire) == UserProcessState::Ready as u8 {
        return true;
    }
    state
        .compare_exchange(
            UserProcessState::Running as u8,
            UserProcessState::Ready as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_ok()
}

pub fn set_current_user_process(id: u64) -> bool {
    let Some(slot) = user_process_slot(id) else {
        return false;
    };
    let state = &USER_PROCESS_STATES[slot];
    loop {
        match state.load(Ordering::Acquire) {
            value if value == UserProcessState::Running as u8 => break,
            value if value == UserProcessState::Ready as u8 => {
                if state
                    .compare_exchange(
                        value,
                        UserProcessState::Running as u8,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    break;
                }
            }
            _ => return false,
        }
    }
    CURRENT_USER_PROCESS.store(id, Ordering::Release);
    if let Some(cpu) = crate::smp::current_per_cpu() {
        cpu.set_current_process(id);
    }
    true
}

pub fn switch_current_user_process(next_id: u64) -> bool {
    let current_id = current_user_process_id();
    if current_id != 0 && current_id != next_id && !mark_user_process_ready(current_id) {
        return false;
    }
    set_current_user_process(next_id)
}

pub fn current_user_process_id() -> u64 {
    CURRENT_USER_PROCESS.load(Ordering::Acquire)
}

pub fn mark_current_user_process_exited(status: u64, fault: bool) -> bool {
    let id = current_user_process_id();
    let Some(slot) = user_process_slot(id) else {
        return false;
    };
    USER_PROCESS_EXIT_STATUS[slot].store(status, Ordering::Release);
    let terminal = if fault {
        UserProcessState::Faulted as u8
    } else {
        UserProcessState::Exited as u8
    };
    USER_PROCESS_STATES[slot]
        .compare_exchange(
            UserProcessState::Running as u8,
            terminal,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_ok()
}

pub fn user_process_state(id: u64) -> Option<UserProcessState> {
    let slot = user_process_slot(id)?;
    Some(match USER_PROCESS_STATES[slot].load(Ordering::Acquire) {
        1 => UserProcessState::Ready,
        2 => UserProcessState::Running,
        3 => UserProcessState::Exited,
        4 => UserProcessState::Faulted,
        _ => UserProcessState::Empty,
    })
}

pub fn user_process_exit_status(id: u64) -> Option<u64> {
    let slot = user_process_slot(id)?;
    Some(USER_PROCESS_EXIT_STATUS[slot].load(Ordering::Acquire))
}
