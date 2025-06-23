#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::ospi::{
    self, AddressSize, ChipSelectHighTime, DummyCycles, FIFOThresholdLevel, MemorySize, MemoryType, OspiWidth, WrapSize
};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p_config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        p_config.rcc.hsi = true;
        p_config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI, // 16 MHz
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL10,
            divp: None,
            divq: Some(PllDiv::DIV2), // 80 MHz for OCTOSPI
            divr: Some(PllDiv::DIV1), // 160 MHz
        });
        p_config.rcc.sys = Sysclk::PLL1_R;
        p_config.rcc.ahb_pre = AHBPrescaler::DIV1;
        p_config.rcc.apb1_pre = APBPrescaler::DIV1;
        p_config.rcc.apb2_pre = APBPrescaler::DIV1;
        p_config.rcc.apb3_pre = APBPrescaler::DIV1;

        p_config.rcc.voltage_range = VoltageScale::RANGE1;
        p_config.rcc.hsi48 = Some(Hsi48Config {
            sync_from_usb: true,
        }); // needed for USB

        p_config.rcc.pll2 = Some(Pll {
            source: PllSource::HSI, // 16 MHz
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL8,  // 128MHz
            divp: None,
            divq: Some(PllDiv::DIV128), // 1 MHz
            divr: Some(PllDiv::DIV2),
        });
        p_config.rcc.mux.iclksel = mux::Iclksel::HSI48; // USB uses ICLK
        
        p_config.rcc.mux.octospisel = mux::Octospisel::PLL2_Q;
    }
    let p = embassy_stm32::init(p_config);

    info!("Initializing OCTOSPI...");

    let ospi_config = ospi::Config {
        fifo_threshold: FIFOThresholdLevel::_8Bytes,
        memory_type: MemoryType::Standard,
        device_size: MemorySize::_16MiB, // 设置一个合理的设备大小
        chip_select_high_time: ChipSelectHighTime::_5Cycle,
        free_running_clock: false,
        clock_mode: false,
        wrap_size: WrapSize::None,
        clock_prescaler: 3, // 80MHz / (3+1) = 20MHz，这是一个安全的频率
        sample_shifting: false,
        delay_hold_quarter_cycle: false,
        chip_select_boundary: 0,
        delay_block_bypass: true,
        max_transfer: 0,
        refresh: 0,
    };

    let tx_config = ospi::TransferConfig {
        iwidth: OspiWidth::OCTO, // 先用单线模式测试
        instruction: Some(0x0072),
        isize: AddressSize::_16Bit, // 改为8位指令
        idtr: false,

        adwidth: OspiWidth::OCTO, // 先用单线模式测试
        address: Some(0x7000),
        adsize: AddressSize::_16Bit,
        addtr: false,

        abwidth: OspiWidth::NONE, // 不使用alternate bytes
        alternate_bytes: None,
        absize: AddressSize::_8Bit,
        abdtr: false,

        dwidth: OspiWidth::OCTO, // 先用单线模式测试
        ddtr: false,

        dummy: DummyCycles::_4, // 先不使用dummy cycles来简化
    };

    info!("Creating OCTOSPI instance...");

    let mut ospi = ospi::Ospi::new_blocking_octospi(
        p.OCTOSPI1,
        p.PF10, // CLK
        p.PF8,  // D0
        p.PF9,  // D1
        p.PF7,  // D2
        p.PF6,  // D3
        p.PC1,  // D4
        p.PD5,  // D5
        p.PC3,  // D6
        p.PC0,  // D7
        p.PA2,  // NCS
        ospi_config,
    );

    info!("OCTOSPI initialized successfully!");

    info!("Testing write operation...");
    let mut buf: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    match ospi.blocking_write(&mut buf, tx_config) {
        Ok(()) => info!("Write successful! buf: {:?}", buf),
        Err(e) => info!("Write failed: {:?}", e),
    }

    loop {
        info!("OCTOSPI Demo running...");
        Timer::after(Duration::from_millis(1000)).await;
    }
}