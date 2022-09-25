#![cfg_attr(not(test), no_std)]

use bitvec::prelude::BitArray;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

pub mod hw;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GicType {
    CortexA15,
}

pub struct GicV2<'a> {
    distributor: &'a hw::Distributor,
    cpu: &'a hw::CPU,
    supported_interrupts: BitArray<[u32; 32]>,
    permanent_interrupts: BitArray<[u32; 32]>,
    num_cpu_interfaces: u32,
}

unsafe impl Sync for GicV2<'_> {}

impl GicV2<'_> {
    pub unsafe fn new(distributor: usize, cpu: usize, _gic_type: GicType) -> Self {
        critical_section::with(|_cs| {
            let distributor = &*(distributor as *const hw::Distributor);
            let cpu = &*(cpu as *const hw::CPU);
            // Disable all interrupts
            distributor.ctrl.modify(hw::Ctrl::ENABLE::CLEAR);

            let typer = distributor.typer.extract();
            let num_cpu_interfaces = typer.read(hw::Type::CPUS) + 1;
            let itlines = typer.read(hw::Type::ITLINES);
            let max_interrupts = 32 * (itlines + 1);
            // let max_spis = max_interrupts.saturating_sub(32);
            let max_isenable = div_ceil(max_interrupts, 32);
            let max_icfg = div_ceil(max_interrupts, 16);
            let max_ipriority = div_ceil(max_interrupts, 4);
            // TODO: handle security extensions
            let _security_ext = typer.read(hw::Type::SEC);

            let mut supported_interrupts = BitArray::new([0u32; 32]);
            let mut permanent_interrupts = BitArray::new([0u32; 32]);

            for i in 0..max_isenable as usize {
                // Deactivate all interrupts
                distributor.icactive[i].set(0xffffffff);

                distributor.isenable[i].set(0xffffffff);
                let supported = distributor.isenable[i].get();
                supported_interrupts.data[i] = supported;
                distributor.icenable[i].set(0xffffffff);
                let permanent = distributor.icenable[i].get();
                permanent_interrupts.data[i] = permanent;

                // Leave all interrupts disabled
            }

            // Set all interrupts to active low, level triggered. PPIs and SGIs
            // are not configurable on GICv2 so skip them.
            for i in 2..max_icfg as usize {
                distributor.icfg[i].set(0);
            }

            // Configure interrupt priority.
            for i in 0..max_ipriority as usize {
                distributor.ipriority[i].set(0xa0a0a0a0);
            }

            if num_cpu_interfaces > 1 {
                let mut cpu_mask = 0;
                // First 8 ITARGET registers are banked for each CPU. By reading
                // them we can determine current CPU mask so that we can redirect
                // all interrupts to this CPU.
                for i in 0..8 {
                    cpu_mask = distributor.itarget[i].get();
                    cpu_mask |= cpu_mask >> 16;
                    cpu_mask |= cpu_mask >> 8;
                    if cpu_mask != 0 {
                        break;
                    }
                }

                if cpu_mask == 0 {
                    panic!("Could not determine CPU mask");
                }

                cpu_mask |= cpu_mask << 8;
                cpu_mask |= cpu_mask << 16;

                for i in 8..max_interrupts {
                    // Redirect all interrupts to this CPU.
                    distributor.itarget[i as usize].set(cpu_mask);
                }
            }

            let gic = Self {
                distributor,
                cpu,
                supported_interrupts,
                permanent_interrupts,
                num_cpu_interfaces,
            };

            gic.cpu_init();
            gic.distributor_enable();
            gic
        })
    }

    /// Checks if the given interrupt is supported.
    pub fn is_interrupt_supported(&self, interrupt: u32) -> bool {
        self.supported_interrupts
            .get(interrupt as usize)
            .map(|x| *x)
            .unwrap_or(false)
    }

    /// Checks if the given interrupt is permanently enabled.
    pub fn is_interrupt_permanent(&self, interrupt: u32) -> bool {
        self.permanent_interrupts
            .get(interrupt as usize)
            .map(|x| *x)
            .unwrap_or(false)
    }

    /// Globally disable interrupts
    pub fn distributor_disable(&self) {
        self.distributor.ctrl.modify(hw::Ctrl::ENABLE::CLEAR);
    }

    /// Globally enable interrupts
    pub fn distributor_enable(&self) {
        self.distributor.ctrl.modify(hw::Ctrl::ENABLE::SET);
    }

    /// Get number of CPU interfaces supported by GIC, this is not the same as
    /// the number of CPUs present.
    pub fn num_cpu_interfaces(&self) -> u32 {
        self.num_cpu_interfaces
    }

    pub fn interrupt_unmask(&self, interrupt: u32) {
        if !self.is_interrupt_supported(interrupt) {
            panic!("Interrupt #{} is not supported", interrupt);
        }

        let reg = interrupt / 32;
        let bit = interrupt % 32;

        self.distributor.isenable[reg as usize].set(1 << bit);
    }

    /// Per-CPU initialization. Currently we don't support more than one CPU so
    /// this is private.
    unsafe fn cpu_init(&self) {
        self.cpu.pmr.set(0xf0);
        for i in 0..4 {
            self.cpu.apr[i as usize].set(0);
        }

        // Enable CPU interface.
        self.cpu
            .ctrl
            .write(hw::CpuCtrl::ENABLE::SET + hw::CpuCtrl::EOI_MODE::CLEAR)
    }

    pub fn get_first_sgi(&self) -> Option<u32> {
        let mask = self.supported_interrupts.data[0] & 0xF;
        if mask == 0 {
            return None;
        }
        Some(mask.trailing_zeros())
    }

    pub fn send_sgi_to_self(&self, sgi: u32) {
        assert!(sgi <= 16);
        assert!(self.is_interrupt_supported(sgi));
        self.distributor.sgi.set(sgi | (2 << 24));
    }

    pub fn handle_irq<F>(&self, f: F)
    where
        F: FnOnce(u32),
    {
        let iar = self.cpu.iar.extract();
        f(iar.read(hw::Iar::INTERRUPT));
        self.cpu.eoir.set(iar.get());
    }
}

#[inline(always)]
fn div_ceil<T>(dividend: T, divisor: T) -> <T as core::ops::Div>::Output
where
    T: Copy,
    T: core::ops::Div + core::ops::Rem,
    <T as core::ops::Rem<T>>::Output: PartialEq + From<u8>,
    <T as core::ops::Div<T>>::Output: core::ops::AddAssign + From<u8>,
{
    let mut x = dividend / divisor;
    if dividend % divisor != 0.into() {
        core::ops::AddAssign::add_assign(&mut x, 1.into());
    }
    x
}
