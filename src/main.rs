// GP2Y0E02 I2CデバイスID書き換えプログラム
// 
// リファレンス
// https://akizukidenshi.com/goodsaffix/GP2Y0E02_an_20180829.pdf

/*
実行結果例 
I (3461) gpio: GPIO[13]| InputEn: 0| OutputEn: 0| OpenDrain: 0| Pullup: 0| Pulldown: 0| Intr:0 
I (3461) change_addr_gp2y0e03_i2c: Start
I (3461) change_addr_gp2y0e03_i2c: Set Device ID : 0x10, Write Id: 0x20
I (3461) change_addr_gp2y0e03_i2c: Stage1
I (3461) change_addr_gp2y0e03_i2c: Stage2
I (3471) change_addr_gp2y0e03_i2c: Stage3
I (3471) change_addr_gp2y0e03_i2c: Stage4
I (3481) change_addr_gp2y0e03_i2c: Stage5
I (3481) change_addr_gp2y0e03_i2c: Stage6
I (3491) change_addr_gp2y0e03_i2c: Stage7
I (3491) change_addr_gp2y0e03_i2c: Stage8
I (3501) change_addr_gp2y0e03_i2c: Finish!
I (3501) change_addr_gp2y0e03_i2c: Check Device Id
I (3511) change_addr_gp2y0e03_i2c: I2C Device Id : 0x10
I (3511) change_addr_gp2y0e03_i2c: Stage9
I (3521) change_addr_gp2y0e03_i2c: Result > I2C Device Id : 0x10, Write Id: 0x20
I (3531) change_addr_gp2y0e03_i2c: Finish!
*/

use esp_idf_hal::peripherals::Peripherals;

use esp_idf_hal::gpio::PinDriver;

use esp_idf_hal::i2c::I2C0;
use esp_idf_hal::i2c::I2cConfig;
use esp_idf_hal::i2c::I2cDriver;

use esp_idf_hal::units::Hertz;
use esp_idf_hal::sys::TickType_t;
use esp_idf_hal::delay::TickType;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // 設定したいデバイスID (要書き換え) > Slave ID : Write
    const NEW_WRITE_DEVICE_ID : u8 = 0x20;

    // ペリフェラルの取得 (必要に応じて書き換え)
    let peripherals = Peripherals::take().unwrap();
    let i2c_scl = peripherals.pins.gpio27; // SCL Pin
    let i2c_sda = peripherals.pins.gpio26; // SDA Pin
    let vpp_pin = peripherals.pins.gpio13; // GPIO Write Pin
    let i2c: I2C0 = peripherals.i2c0;

    // GP2Uセンサー I2C Default Address
    let i2c_device_id : u8 = 0x40;

    // 設定したい書き込みデバイスIDから、デバイスID・書き込み情報を作成
    const NEW_DEVICE_ID : u8 = NEW_WRITE_DEVICE_ID >> 1;
    const I2C_WRITE_DEVICE_ID_DATA : u8 = NEW_DEVICE_ID >> 3 & 0xf;

    // I2C 初期化
    let i2c_config = I2cConfig::new().baudrate(Hertz(100_000));
    let mut i2c_driver = I2cDriver::new(i2c, i2c_sda, i2c_scl, &i2c_config).unwrap();

    // GPIOの取得
    let mut gpio_vpp_driver = PinDriver::output(vpp_pin).unwrap();
    gpio_vpp_driver.set_low().unwrap(); // Vpp Low

    // 書き込みシーケンス開始
    log::info!("Start");
    log::info!("Set Device ID : 0x{:x}, Write Id: 0x{:x}", NEW_DEVICE_ID, NEW_DEVICE_ID << 1);
    
    std::thread::sleep(std::time::Duration::from_millis(3000));

    log::info!("Stage1");
    match i2c_driver.write(i2c_device_id, &[0xec, 0xff], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }
    gpio_vpp_driver.set_high().unwrap(); // Vpp High

    log::info!("Stage2"); // E-Fuse R/WにE-Fuse bit Map(LSB 0x00)を設定
    match i2c_driver.write(i2c_device_id, &[0xc8, 0x00], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }

    log::info!("Stage3"); // E-Fuse  書き込み場所(I2cSlaveId)を指定 (BankE (0x05) |  書込ビット数(5-1) 0x40)
    match i2c_driver.write(i2c_device_id, &[0xc9, 0x45], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }

    log::info!("Stage4"); // E-Fuse Program Data 
    match i2c_driver.write(i2c_device_id, &[0xcd, I2C_WRITE_DEVICE_ID_DATA], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }

    log::info!("Stage5"); // E-Fuse Program Enable  (0xca:rw 0x01 enable)
    match i2c_driver.write(i2c_device_id, &[0xca, 0x01], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }

    std::thread::sleep(std::time::Duration::from_micros(500)); // 500μs待機

    log::info!("Stage6"); // E-Fuse Program Enable  (0xca:rw 0x00 disable)
    match i2c_driver.write(i2c_device_id, &[0xca, 0x00], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }
    gpio_vpp_driver.set_low().unwrap(); // Vpp Low

    log::info!("Stage7");
    // Bank select  (0xef:rw)
    match i2c_driver.write(i2c_device_id, &[0xef, 0x00], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }
    // E-Fuse load bank3 registe (0xc8:rw) # 0x40 = 0100 0000
    match i2c_driver.write(i2c_device_id, &[0xc8, 0x40], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }
    // E-Fuse load bank3 registe (0xc8:rw)
    match i2c_driver.write(i2c_device_id, &[0xc8, 0x00], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }

    log::info!("Stage8"); // Software Reset
    match i2c_driver.write(i2c_device_id, &[0xee, 0x06], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }

    log::info!("Finish!");
    log::info!("Check Device Id");
    let i2c_device_id : u8 = NEW_DEVICE_ID;
    log::info!("I2C Device Id : 0x{:x}", i2c_device_id);

    log::info!("Stage9");
    // Bank Select (0xEF:R/W)
    match i2c_driver.write(i2c_device_id, &[0xef, 0x00], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }
    // Clock Select (0xEC:R/W)
    match i2c_driver.write(i2c_device_id, &[0xec, 0xff], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }
    // Bank Select (0xEF:R/W)
    match i2c_driver.write(i2c_device_id, &[0xef, 0x03], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }
    // 書き換え済みDeviceIDの確認
    match i2c_driver.write(i2c_device_id, &[0x27], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }

    let mut rx_buf: [u8; 1] = [0; 1];
    let device_id = match i2c_driver.read(i2c_device_id, &mut rx_buf, TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => {
            (rx_buf[0] << 3) & 0x7F
        }
        Err(e) => {
            log::warn!("Error: {:?}", e);
            return;
        }
    };
    log::info!("Result > I2C Device Id : 0x{:x}, Write Id: 0x{:x}", device_id, device_id << 1);

    // Bank Select (0xEF:R/W)
    match i2c_driver.write(i2c_device_id, &[0xef, 0x00], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }
    // Clock Select (0xEC:R/W)
    match i2c_driver.write(i2c_device_id, &[0xec, 0x7f], TickType_t::from(TickType::new_millis(100))) {
        Ok(_) => (),
        Err(e) => log::warn!("Error: {:?}", e),
    }

    log::info!("Finish!");
}
