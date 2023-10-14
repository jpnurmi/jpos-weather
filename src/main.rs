mod daytime;
mod geoip;
mod smhi;

pub mod ui {
    slint::include_modules!();
}
use slint::ComponentHandle;

fn main() {
    let window = ui::MainWindow::new().unwrap();

    geoip::refresh(window.as_weak());

    window.global::<ui::GeoIP>().on_refresh({
        let handle = window.as_weak();
        move || geoip::refresh(handle.clone())
    });
    window.global::<ui::DayTime>().on_refresh({
        let handle = window.as_weak();
        move |latitude, longitude, timezone| {
            daytime::refresh(latitude, longitude, timezone.to_string(), handle.clone())
        }
    });
    window.global::<ui::Smhi>().on_refresh({
        let handle = window.as_weak();
        move |latitude, longitude| smhi::refresh(latitude, longitude, handle.clone())
    });

    slint::Timer::default().start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(24 * 3600),
        {
            let handle = window.as_weak();
            move || geoip::refresh(handle.clone())
        },
    );

    slint::Timer::default().start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_secs(3600),
        {
            let geoip = window.global::<ui::GeoIP>();
            let latitude = geoip.get_latitude();
            let longitude = geoip.get_longitude();
            let handle = window.as_weak();
            move || smhi::refresh(latitude, longitude, handle.clone())
        },
    );

    window.run().unwrap();
}
