use crate::{
    binary::include::esp_bt_controller_config_t,
    hal::{
        clock::ModemClockController,
        interrupt,
        peripherals::{BT, Interrupt},
    },
};

pub(crate) static mut ISR_INTERRUPT_4: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

pub(crate) static mut ISR_INTERRUPT_7: (
    *mut crate::binary::c_types::c_void,
    *mut crate::binary::c_types::c_void,
) = (core::ptr::null_mut(), core::ptr::null_mut());

// keep them aligned with BT_CONTROLLER_INIT_CONFIG_DEFAULT in ESP-IDF
// ideally _some_ of these values should be configurable
pub(crate) static BLE_CONFIG: esp_bt_controller_config_t = esp_bt_controller_config_t {
    config_version: 0x20240422,
    ble_ll_resolv_list_size: 4,
    ble_hci_evt_hi_buf_count: 30,
    ble_hci_evt_lo_buf_count: 8,
    ble_ll_sync_list_cnt: 5,
    ble_ll_sync_cnt: 20,
    ble_ll_rsp_dup_list_count: 20,
    ble_ll_adv_dup_list_count: 20,
    ble_ll_tx_pwr_dbm: 9,
    rtc_freq: 32000,
    ble_ll_sca: 60,
    ble_ll_scan_phy_number: 1,
    ble_ll_conn_def_auth_pyld_tmo: 3000,
    ble_ll_jitter_usecs: 16,
    ble_ll_sched_max_adv_pdu_usecs: 376,
    ble_ll_sched_direct_adv_max_usecs: 502,
    ble_ll_sched_adv_max_usecs: 852,
    ble_scan_rsp_data_max_len: 31,
    ble_ll_cfg_num_hci_cmd_pkts: 1,
    ble_ll_ctrl_proc_timeout_ms: 40000,
    nimble_max_connections: 2,
    ble_whitelist_size: 12,
    ble_acl_buf_size: 255,
    ble_acl_buf_count: 24,
    ble_hci_evt_buf_size: 70,
    ble_multi_adv_instances: 1,
    ble_ext_adv_max_size: 31,
    controller_task_stack_size: 4096,
    controller_task_prio: 253,
    controller_run_cpu: 0,
    enable_qa_test: 0,
    enable_bqb_test: 0,
    enable_tx_cca: 0,
    cca_rssi_thresh: (256 - 50) as u8,
    sleep_en: 0,
    coex_phy_coded_tx_rx_time_limit: 0,
    dis_scan_backoff: 0,
    ble_scan_classify_filter_enable: 1,
    cca_drop_mode: 0,  //???
    cca_low_tx_pwr: 0, //???
    main_xtal_freq: 40,
    ignore_wl_for_direct_adv: 0,
    config_magic: 0x5A5AA5A5,

    cpu_freq_mhz: 160,
    enable_pcl: 0, // CONFIG_BT_LE_POWER_CONTROL_ENABLED
    version_num: 0,
    csa2_select: 1,
};

pub(crate) fn bt_periph_module_enable() {
    // stealing BT is safe, since it is passed into the initialization function of the BLE
    // controller.
    let mut bt = unsafe { BT::steal() };
    bt.enable_modem_clock(true);
}

pub(crate) fn disable_sleep_mode() {
    // nothing
}

pub(super) unsafe extern "C" fn esp_intr_alloc(
    source: u32,
    flags: u32,
    handler: *mut crate::binary::c_types::c_void,
    arg: *mut crate::binary::c_types::c_void,
    ret_handle: *mut *mut crate::binary::c_types::c_void,
) -> i32 {
    trace!(
        "esp_intr_alloc {} {} {:?} {:?} {:?}",
        source, flags, handler, arg, ret_handle
    );

    unsafe {
        match source {
            4 => {
                ISR_INTERRUPT_4 = (handler, arg);
                unwrap!(interrupt::enable(
                    Interrupt::BT_MAC,
                    interrupt::Priority::Priority1
                ));
            }
            7 => {
                ISR_INTERRUPT_7 = (handler, arg);
                unwrap!(interrupt::enable(
                    Interrupt::LP_TIMER,
                    interrupt::Priority::Priority1
                ));
            }
            _ => panic!("Unexpected interrupt source {}", source),
        }
    }

    0
}

pub(super) fn ble_rtc_clk_init() {
    // stealing BT is safe, since it is passed into the initialization function of the BLE
    // controller.
    let mut bt = unsafe { BT::steal() };
    bt.ble_rtc_clk_init();
}

pub(super) unsafe extern "C" fn esp_reset_rpa_moudle() {
    trace!("esp_reset_rpa_moudle");
    // stealing BT is safe, since it is passed into the initialization function of the BLE
    // controller.
    let mut bt = unsafe { BT::steal() };
    bt.reset_rpa();
}

#[allow(improper_ctypes_definitions)]
#[unsafe(no_mangle)]
unsafe extern "C" fn jrand48(
    _xsubi: [crate::binary::c_types::c_ushort; 3],
) -> crate::binary::c_types::c_long {
    // this is not very random but good enough for now - it's apparently not used
    // for crypto
    unsafe {
        static mut VALUE: u32 = 0;
        VALUE = VALUE.wrapping_add(3);
        VALUE as i32
    }
}
