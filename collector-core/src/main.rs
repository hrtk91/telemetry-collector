use std::time::Duration;

use opentelemetry::metrics::{MeterProvider, ObservableGauge};
use prometheus::{Encoder, TextEncoder};

fn main() {
    #[cfg(target_os = "windows")]
    println!("This is compiled for Windows!");

    #[cfg(target_os = "macos")]
    println!("This is compiled for macOS!");

    #[cfg(target_os = "linux")]
    println!("This is compiled for Linux!");

    let args: Vec<String> = std::env::args().collect();
    let sleep_time = if args.len() > 1 {
        let sleep_time = args[1]
            .parse::<u128>()
            .unwrap_or(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_millis());
        sleep_time.max(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_millis())
    } else {
        sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_millis()
    };

    dotenvy::dotenv()
        .map_err(|e| {
            println!("failed to load .env file : {:#?}", e);
        })
        .ok();

    let meter_name = std::env::var("METER_NAME").unwrap_or("meter".to_string());

    let registry = prometheus::Registry::new();
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .expect("failed init exporter");

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(exporter)
        .build();
    let meter = provider.meter(meter_name);

    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let cpus: Vec<(String, ObservableGauge<f64>)> = sys
        .cpus()
        .iter()
        .map(|cpu| cpu.name().replace(" ", "_"))
        .map(|cpuname| (cpuname.clone(), meter.f64_observable_gauge(cpuname).init()))
        .collect();
    let free_memory = meter.u64_observable_gauge("free_memory").init();
    let used_memory = meter.u64_observable_gauge("used_memory").init();
    let total_memory = meter.u64_observable_gauge("total_memory").init();

    loop {
        sys.refresh_all();

        cpus.iter().for_each(|(name, meter)| {
            sys.cpus()
                .iter()
                .find(|cpu| cpu.name() == name.replace("_", " "))
                .map(|cpu| meter.observe(cpu.cpu_usage() as f64, &[]));
        });

        free_memory.observe(sys.free_memory(), &[]);
        used_memory.observe(sys.used_memory(), &[]);
        total_memory.observe(sys.total_memory(), &[]);

        let encoder = TextEncoder::new();
        let metric_families = registry.gather();
        let mut result = Vec::new();
        if let Err(e) = encoder.encode(&metric_families, &mut result) {
            eprintln!("Error: {}", e);
            continue;
        };

        if let Ok(s) = String::from_utf8(result) {
            println!("{}", s);
        } else {
            eprintln!("Error: Failed to convert to string");
            continue;
        }
        std::thread::sleep(Duration::from_millis(sleep_time as u64));
    }
}
