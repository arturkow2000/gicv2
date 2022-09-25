use tock_registers::{
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

register_bitfields! [
    u32,
    pub Ctrl [
        ENABLE 0
    ],
    pub Type [
        ITLINES OFFSET(0) NUMBITS(4) [],
        CPUS OFFSET(5) NUMBITS(3) [],
        SEC OFFSET(10) NUMBITS(1) []
    ],
    pub SGI [
        SGIINTID OFFSET(0) NUMBITS(4) []
    ],
    pub CpuCtrl [
        ENABLE 0,
        FIQ_BYPASS_DISABLE 5,
        IRQ_BYPASS_DISABLE 6,
        EOI_MODE 9
    ],
    pub Iar [
        INTERRUPT OFFSET(0) NUMBITS(10) []
    ]
];

register_structs! {
    pub Distributor {
        (0x000 => pub ctrl: ReadWrite<u32, Ctrl::Register>),
        (0x004 => pub typer: ReadOnly<u32, Type::Register>),
        (0x008 => pub iidr: ReadOnly<u32>),
        (0x00c => _reserved0),
        (0x080 => pub igroup: [ReadWrite<u32>; 32]),
        (0x100 => pub isenable: [ReadWrite<u32>; 32]),
        (0x180 => pub icenable: [ReadWrite<u32>; 32]),
        (0x200 => pub ispend: [ReadWrite<u32>; 32]),
        (0x280 => pub icpend: [ReadWrite<u32>; 32]),
        (0x300 => pub isactive: [ReadWrite<u32>; 32]),
        (0x380 => pub icactive: [ReadWrite<u32>; 32]),
        (0x400 => pub ipriority: [ReadWrite<u32>; 256]),
        (0x800 => pub itarget: [ReadWrite<u32>; 256]),
        (0xc00 => pub icfg: [ReadWrite<u32>; 64]),
        (0xd00 => pub ppis: ReadOnly<u32>),
        (0xd04 => pub spis: [ReadOnly<u32>; 6]),
        (0xd1c => _reserved12),
        (0xf00 => pub sgi: WriteOnly<u32, SGI::Register>),
        (0xf04 => _reserved13),
        (0xf10 => pub cpendsgi: [ReadWrite<u32>; 4]),
        (0xf20 => pub spendsgi: [ReadWrite<u32>; 4]),
        (0xf30 => _reserved15),
        (0xfd0 => pub pidr4: ReadOnly<u32>),
        (0xfd4 => pub pidr5: ReadOnly<u32>),
        (0xfd8 => pub pidr6: ReadOnly<u32>),
        (0xfdc => pub pidr7: ReadOnly<u32>),
        (0xfe0 => pub pidr0: ReadOnly<u32>),
        (0xfe4 => pub pidr1: ReadOnly<u32>),
        (0xfe8 => pub pidr2: ReadOnly<u32>),
        (0xfec => pub pidr3: ReadOnly<u32>),
        (0xff0 => pub cidr0: ReadOnly<u32>),
        (0xff4 => pub cidr1: ReadOnly<u32>),
        (0xff8 => pub cidr2: ReadOnly<u32>),
        (0xffc => pub cidr3: ReadOnly<u32>),
        (0x1000 => @END),
    },
    pub CPU {
        (0x000 => pub ctrl: ReadWrite<u32, CpuCtrl::Register>),
        (0x004 => pub pmr: ReadWrite<u32>),
        (0x008 => pub bpr: ReadWrite<u32>),
        (0x00c => pub iar: ReadOnly<u32, Iar::Register>),
        (0x010 => pub eoir: WriteOnly<u32>),
        (0x014 => pub rpr: ReadOnly<u32>),
        (0x018 => pub hppir: ReadOnly<u32>),
        (0x01c => pub abpr: ReadWrite<u32>),
        (0x020 => pub aiar: ReadOnly<u32>),
        (0x024 => pub aeoir: WriteOnly<u32>),
        (0x028 => pub ahppir: ReadOnly<u32>),
        (0x02c => _reserved0),
        (0x0d0 => pub apr: [ReadWrite<u32>; 4]),
        (0x0e0 => pub nsapr: [ReadWrite<u32>; 4]),
        (0x0f0 => _reserved1),
        (0x0fc => pub iidr: ReadOnly<u32>),
        (0x100 => _reserved2),
        (0x1000 => pub dir: WriteOnly<u32>),
        (0x1004 => @END),
    }
}
