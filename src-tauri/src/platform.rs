use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY,
    VK_CONTROL,
};

pub fn simulate_ctrl_v() -> Result<(), String> {
    unsafe {
        let mut inputs: [INPUT; 4] = std::mem::zeroed();
        inputs[0].r#type = INPUT_KEYBOARD;
        inputs[0].Anonymous.ki = KEYBDINPUT {
            wVk: VK_CONTROL,
            dwFlags: KEYBD_EVENT_FLAGS::default(),
            ..Default::default()
        };
        inputs[1].r#type = INPUT_KEYBOARD;
        inputs[1].Anonymous.ki = KEYBDINPUT {
            wVk: VIRTUAL_KEY(0x56), // 'V'
            dwFlags: KEYBD_EVENT_FLAGS::default(),
            ..Default::default()
        };
        inputs[2].r#type = INPUT_KEYBOARD;
        inputs[2].Anonymous.ki = KEYBDINPUT {
            wVk: VIRTUAL_KEY(0x56),
            dwFlags: KEYEVENTF_KEYUP,
            ..Default::default()
        };
        inputs[3].r#type = INPUT_KEYBOARD;
        inputs[3].Anonymous.ki = KEYBDINPUT {
            wVk: VK_CONTROL,
            dwFlags: KEYEVENTF_KEYUP,
            ..Default::default()
        };
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        if sent != inputs.len() as u32 {
            let error = windows::core::Error::from_win32();
            return Err(format!(
                "SendInput отправил {sent} из {} событий: {error}",
                inputs.len()
            ));
        }
    }
    Ok(())
}
