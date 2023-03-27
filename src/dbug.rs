use std::{sync::{Mutex, Arc}, thread, time::Duration};


#[allow(non_snake_case, unused)]
pub fn ________BREAKPOINT_________(name: &str) {
    let name = name.to_string();
    let mutex = Arc::new(Mutex::new(()));
    let mc = mutex.clone();
    thread::spawn(move || {
        let _locked = mc.lock().unwrap();
        
        native_dialog::MessageDialog::new()
            .set_text(name.as_str())
            .show_alert()
        
    });

    thread::sleep(Duration::from_secs_f32(0.1));
    {
        let _locked = mutex.lock().unwrap();
    }
    
}