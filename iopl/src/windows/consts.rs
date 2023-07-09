pub(crate) const CONTEXT_AMD64: u32 = 0x10_0000;

pub(crate) const CONTEXT_CONTROL        : u32 = CONTEXT_AMD64 | 0x0000_0001;
pub(crate) const CONTEXT_INTEGER        : u32 = CONTEXT_AMD64 | 0x0000_0002;
pub(crate) const CONTEXT_SEGMENTS       : u32 = CONTEXT_AMD64 | 0x0000_0004;
pub(crate) const CONTEXT_FLOATING_POINT : u32 = CONTEXT_AMD64 | 0x0000_0008;
pub(crate) const CONTEXT_DEBUG_REGISTERS: u32 = CONTEXT_AMD64 | 0x0000_0010;

pub(crate) const CONTEXT_ALL: u32 = CONTEXT_CONTROL
    | CONTEXT_INTEGER
    | CONTEXT_SEGMENTS
    | CONTEXT_FLOATING_POINT
    | CONTEXT_DEBUG_REGISTERS;

pub(crate) const EFLAGS_IOPL_OFFSET: u32 = 12;

#[allow(clippy::unreadable_literal)] pub(crate) const MARK_RAX: u64 = 0x1EE7C0DE;
#[allow(clippy::unreadable_literal)] pub(crate) const MARK_RCX: u64 = 0xC0FFEE;
#[allow(clippy::unreadable_literal)] pub(crate) const MARK_RDX: u64 = 0xCACA0;
#[allow(clippy::unreadable_literal)] pub(crate) const MARK_R8: u64  = 0x7EA;
#[allow(clippy::unreadable_literal)] pub(crate) const MARK_R9: u64  = 0xFACADE;