use std::process::Command;

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

fn get_boost_level(volume: f32) -> u32 {
    let vol_i = (volume * 100.0) as u32;
    vol_i / 100
}

pub fn volume(volume: f32) {
    let mut normalized_volume = volume;
    let boosting = volume > 1.0;
    let boost_level = get_boost_level(volume);
    if boosting {
        normalized_volume = boost_level as f32 * 1.0;
    }
    let boosted = if boosting {
        format!("{boost_level}x boost").to_string()
    } else { String::new() };

    let mut cmd = Command::new("notify-send");
    let icon = get_icon(volume);
    let vol_int = (normalized_volume * 100.0) as u32;
    cmd.args([
        "-a",
        "volume",
        "-u",
        "low",
        "-h",
        format!("int:value:{vol_int}").as_str(),
        "-i",
        icon.as_str(),
        format!("Volume{boosted}").as_str(),
    ])
        .status()
        .expect("error notifying");
}
