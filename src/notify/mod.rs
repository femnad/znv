use ::notify_rust::{Hint, Notification, Urgency};

const NOTIFICATION_SUMMARY: &str = "nor";

fn get_volume_classifier(volume: f32) -> String {
    let vol = if volume == 0.0 {
        "muted"
    } else if volume <= 0.3 {
        "low"
    } else if volume <= 0.6 {
        "medium"
    } else if volume <= 1.0 {
        "high"
    } else {
        "overamplified"
    };
    vol.to_string()
}

fn get_icon(volume: f32) -> String {
    let classifier = get_volume_classifier(volume);
    format!("audio-volume-{}-symbolic", classifier)
}

pub fn message(msg: &str) {
    Notification::new()
        .summary(NOTIFICATION_SUMMARY)
        .body(msg)
        .show()
        .expect("error sending notification");
}

pub fn volume(volume: f32) {
    let icon = get_icon(volume);
    let vol_int = (volume * 100.0) as u32;

    Notification::new()
        .appname("volume")
        .urgency(Urgency::Low)
        .hint(Hint::CustomInt("value".to_string(), vol_int as i32))
        .icon(icon.as_str())
        .summary("Volume")
        .show()
        .expect("error sending notification");
}
