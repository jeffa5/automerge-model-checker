use std::collections::BTreeMap;
use std::hash::Hash;
use std::time::Duration;
use std::time::Instant;

use num_format::SystemLocale;
use num_format::ToFormattedString;
use stateright::actor::ActorModel;
use stateright::Expectation;
use stateright::Model;

use crate::global::GlobalActor;
use crate::model::ModelBuilder;

/// A reporter with more information about the rate of new states being processed.
#[derive(Debug, Default)]
pub struct Reporter {
    last_total: usize,
    last_unique: usize,
    last_report: Option<Instant>,
    properties: BTreeMap<&'static str, Expectation>,
}

impl Reporter {
    /// Create a new reporter.
    pub fn new<M: ModelBuilder>(
        model: &ActorModel<GlobalActor<M::App, M::Driver>, M::Config, M::History>,
    ) -> Self {
        let properties = model
            .properties()
            .iter()
            .map(|p| (p.name, p.expectation.clone()))
            .collect();
        Self {
            last_total: 0,
            last_unique: 0,
            last_report: None,
            properties,
        }
    }
}

impl<M> stateright::report::Reporter<M> for Reporter
where
    M: Model,
{
    fn report_checking(&mut self, data: stateright::report::ReportData) {
        if !data.done {
            if let Some(last_report) = self.last_report {
                let time_since_last_report = last_report.elapsed();
                if time_since_last_report < Duration::from_secs(1) {
                    return;
                }
            }
            self.last_report = Some(Instant::now());
        }

        let new_total = data.total_states - self.last_total;
        let total_rate = (data.total_states as f64 / data.duration.as_secs_f64()).round() as u64;
        let new_unique = data.unique_states - self.last_unique;
        let unique_rate = (data.unique_states as f64 / data.duration.as_secs_f64()).round() as u64;
        let max_depth = data.max_depth;
        let status = if data.done { "Done    " } else { "Checking" };
        let locale = SystemLocale::default().unwrap();
        let duration = data.duration.as_millis();
        println!(
            "{} states={: >8} (+{: <8} {: >8}/s), unique={: >8} (+{: <8} {: >8}/s), max_depth={}, duration={:?}ms",
            status,
            data.total_states.to_formatted_string(&locale),
            new_total.to_formatted_string(&locale),
            total_rate.to_formatted_string(&locale),
            data.unique_states.to_formatted_string(&locale),
            new_unique.to_formatted_string(&locale),
            unique_rate.to_formatted_string(&locale),
            max_depth,
            duration
        );

        self.last_total = data.total_states;
        self.last_unique = data.unique_states;
    }

    fn report_discoveries(
        &mut self,
        discoveries: BTreeMap<&'static str, stateright::report::ReportDiscovery<M>>,
    ) where
        <M as Model>::Action: std::fmt::Debug,
        <M as Model>::State: std::fmt::Debug,
        <M as Model>::State: Hash,
    {
        let discoveries: BTreeMap<_, _> = discoveries.into_iter().collect();
        let (success, failure): (Vec<_>, Vec<_>) =
            self.properties.iter().partition(|(name, expectation)| {
                property_holds(expectation, discoveries.get(*name).is_some())
            });

        for (name, expectation) in &self.properties {
            let status = if property_holds(expectation, discoveries.get(name).is_some()) {
                "OK"
            } else {
                "FAILED"
            };
            println!("Property {:?} {:?} {}", expectation, name, status);
            if let Some(discovery) = discoveries.get(name) {
                print!("{}, {}", discovery.classification, discovery.path,);
                println!(
                    "To explore this path try re-running with `explore {}`",
                    discovery.path.encode()
                );
            }
        }

        println!(
            "Properties checked. {} succeeded, {} failed",
            success.len(),
            failure.len()
        );
    }

    fn delay(&self) -> std::time::Duration {
        std::time::Duration::from_millis(10)
    }
}

fn property_holds(expectation: &Expectation, discovery: bool) -> bool {
    match (expectation, discovery) {
        // counter-example
        (Expectation::Always, true) => false,
        // no counter-example
        (Expectation::Always, false) => true,
        // counter-example
        (Expectation::Eventually, true) => false,
        // no counter-example
        (Expectation::Eventually, false) => true,
        // example
        (Expectation::Sometimes, true) => true,
        // no example
        (Expectation::Sometimes, false) => false,
    }
}

/// Reporter that only reports the final status with a stable output for use in tests.
#[derive(Default)]
pub struct TestReporter {
    /// Data collected by the reporter.
    pub data: String,
}

impl<M> stateright::report::Reporter<M> for TestReporter
where
    M: Model,
{
    fn report_checking(&mut self, data: stateright::report::ReportData) {
        if !data.done {
            return;
        }

        self.data.push_str(&format!(
            "Done states={}, unique={}, max_depth={}\n",
            data.total_states, data.unique_states, data.max_depth,
        ));
    }

    fn report_discoveries(
        &mut self,
        discoveries: BTreeMap<&'static str, stateright::report::ReportDiscovery<M>>,
    ) where
        <M as Model>::Action: std::fmt::Debug,
        <M as Model>::State: std::fmt::Debug,
        <M as Model>::State: Hash,
    {
        let discoveries: BTreeMap<_, _> = discoveries.into_iter().collect();
        for (name, discovery) in discoveries {
            self.data.push_str(&format!(
                "Discovered \"{}\" {} {}",
                name, discovery.classification, discovery.path,
            ));
            self.data.push_str(&format!(
                "To explore this path try re-running with `explore {}`",
                discovery.path.encode()
            ));
        }
    }
}
