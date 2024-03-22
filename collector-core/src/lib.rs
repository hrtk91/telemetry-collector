use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use opentelemetry::metrics::MeterProvider;
use opentelemetry::metrics::ObservableGauge;
use prometheus::Encoder;

pub struct CollectorCore {
    registry: prometheus::Registry,
    meter: opentelemetry::metrics::Meter,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    running: Arc<AtomicBool>
}

impl CollectorCore {
    pub fn new() -> Self {
        let sleep_time = sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_millis();

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

        let cpu_meters: Vec<(String, ObservableGauge<f64>)> = sys
            .cpus()
            .iter()
            .map(|cpu| cpu.name().replace(" ", "_"))
            .map(|cpuname| (cpuname.clone(), meter.f64_observable_gauge(cpuname).init()))
            .collect();

        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);

        let thread_handle = std::thread::spawn(move || {
            while running_clone.load(std::sync::atomic::Ordering::Acquire) {
                std::thread::sleep(std::time::Duration::from_millis(sleep_time as u64));
                sys.refresh_cpu();

                for (cpuname, meter) in cpu_meters.iter() {
                    if let Some(a) = sys
                        .cpus()
                        .iter()
                        .find(|cpu| cpu.name() == cpuname.replace("_", " "))
                    {
                        meter.observe(a.cpu_usage() as f64, &[]);
                    }
                }
            }
        });

        CollectorCore {
            registry,
            meter,
            thread_handle: Some(thread_handle),
            running
        }
    }

    pub fn get_metrics(&self) -> Result<String, String> {
        let used_memory = self.meter.f64_observable_gauge("used_memory").init();
        let total_memory = self.meter.f64_observable_gauge("total_memory").init();
        let free_memory = self.meter.f64_observable_gauge("free_memory").init();

        let mut sys = sysinfo::System::new_all();
        sys.refresh_memory();

        used_memory.observe(sys.used_memory() as f64, &[]);
        total_memory.observe(sys.total_memory() as f64, &[]);
        free_memory.observe(sys.free_memory() as f64, &[]);

        let encoder = prometheus::TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut result = Vec::new();
        if let Err(e) = encoder.encode(&metric_families, &mut result) {
            eprintln!("text encode error: {}", e);
            return Err("text encode error".to_string());
        };

        String::from_utf8(result).map_err(|e| {
            eprintln!("utf8 encode error: {}", e);
            "utf8 encode error".to_string()
        })
    }
}

impl Drop for CollectorCore {
    fn drop(&mut self) {
        if let Some(thread_handle) = self.thread_handle.take() {
            self.running.store(false, std::sync::atomic::Ordering::Release);
            thread_handle.join().map_err(|e| {
                eprintln!("failed to join thread: {:#?}", e);
            }).unwrap();
        }
    }
}
