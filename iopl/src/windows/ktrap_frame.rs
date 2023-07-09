#![allow(clippy::upper_case_acronyms)]

use std::arch::x86_64::__m128i;

pub type KProcessorMode = u8;
pub type KIRQL = u8;

#[repr(C)]
pub union GsBaseOrGsSwap {
    pub gs_base: u64,
    pub gs_swap: u64
}

#[repr(C)]
pub union FaultAddrOrContextRecord {
    pub fault_address: u64,
    pub context_record: u64
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DrRegs {
    pub dr0: u64,
    pub dr1: u64,
    pub dr2: u64,
    pub dr3: u64,
    pub dr6: u64,
    pub dr7: u64
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ShadowStackFrame {
    pub shadow_stack_frame: u64,
    pub spare: [u64; 5]
}

#[repr(C)]
pub union DrOrShadowStack {
    pub dr_regs: DrRegs,
    pub shadow_stack_frame: ShadowStackFrame
}

#[repr(C)]
pub struct SpecialDebugRegisters {
    pub debug_control: u64,
    pub last_branch_to_rip: u64,
    pub last_branch_from_rip: u64,
    pub last_exception_to_rip: u64,
    pub last_exception_from_rip: u64
}

#[repr(C)]
pub union ErrorCodeOrExceptionFrame {
    pub error_code: u64,
    pub exception_frame: u64
}

#[repr(C)]
pub struct TrapFrame {

    //
    // Home address for the parameter registers.
    //
    
    pub p1_home: u64,
    pub p2_home: u64,
    pub p3_home: u64,
    pub p4_home: u64,
    pub p5: u64,
    
    //
    // Previous processor mode (system services only) and previous IRQL
    // (interrupts only).
    //
    
    pub previous_mode: KProcessorMode,
    
    pub previous_irql: KIRQL,
    
    //
    // Page fault load/store indicator.
    //
    
    pub fault_indicator: u8,
    
    //
    // Exception active indicator.
    //
    //    0 - interrupt frame.
    //    1 - exception frame.
    //    2 - service frame.
    //
    
    pub exception_active: u8,
    
    //
    // Floating point state.
    //
    
    pub mxcsr: u32,
    
    //
    //  Volatile registers.
    //
    // N.B. These registers are only saved on exceptions and interrupts. They
    //      are not saved for system calls.
    //
    
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    
    //
    // Gsbase is only used if the previous mode was kernel.
    //
    // GsSwap is only used if the previous mode was user.
    //
    
    pub gs_base_or_gs_swap: GsBaseOrGsSwap,
    
    //
    // Volatile floating registers.
    //
    // N.B. These registers are only saved on exceptions and interrupts. They
    //      are not saved for system calls.
    //
    
    pub xmm0: __m128i,
    pub xmm1: __m128i,
    pub xmm2: __m128i,
    pub xmm3: __m128i,
    pub xmm4: __m128i,
    pub xmm5: __m128i,
    
    //
    // First parameter, page fault address, context record address if user APC
    // bypass.
    //
    
    pub fault_address_or_context_record: FaultAddrOrContextRecord,
    
    //
    // The debug registers are only used in user-to-kernel traps. The pointer to
    // the shadow stack machine frame is only used in kernel-to-kernel traps. The
    // spares are available for use in kernel-to-kernel traps only.
    //
    

    pub dr_or_shadow_stack: DrOrShadowStack,
    
    //
    // Special debug registers.
    //
    
    pub special_debug_registers: SpecialDebugRegisters,

    //
    //  Segment registers
    //
    
    pub seg_ds: u16,
    pub seg_es: u16,
    pub seg_fs: u16,
    pub seg_gs: u16,
    
    //
    // Previous trap frame address.
    //
    
    pub trap_frame: u64,
    
    //
    // Saved nonvolatile registers RBX, RDI and RSI. These registers are only
    // saved in system service trap frames.
    //
    
    pub rbx: u64,
    pub rdi: u64,
    pub rsi: u64,
    
    //
    // Saved nonvolatile register RBP. This register is used as a frame
    // pointer during trap processing and is saved in all trap frames.
    //
    
    pub rbp: u64,
    
    //
    // Information pushed by hardware.
    //
    // N.B. The error code is not always pushed by hardware. For those cases
    //      where it is not pushed by hardware a dummy error code is allocated
    //      on the stack.
    //
    
    pub error_code_or_exception_frame: ErrorCodeOrExceptionFrame,

    pub rip: u64,
    pub seg_cs: u16,
    pub fill_0: u8,
    pub logging: u8,
    pub fill_1: [u16; 2],
    pub eflags: u32,
    pub fill_2: u32,
    pub rsp: u64,
    pub seg_ss: u16,
    pub fill_3: u16,
    pub fill_4: u32
}