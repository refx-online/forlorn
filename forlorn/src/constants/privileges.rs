use bitflags::bitflags;

bitflags! {
    #[derive(Default, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct Privileges: u32 {
        // NOTE: only defining privileges that are currently used
        // TODO: define other privileges as needed?
        const UNRESTRICTED       = 1 << 0;
        const VERIFIED           = 1 << 1;
        const WHITELISTED        = 1 << 2;
    }
}
