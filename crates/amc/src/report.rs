use std::collections::BTreeMap;
use std::hash::Hash;
use std::time::Duration;

use num_format::SystemLocale;
use num_format::ToFormattedString;
use stateright::Model;

/// A reporter with more information about the rate of new states being processed.
#[derive(Debug, Default)]
pub struct Reporter {
    last_total: usize,
    last_unique: usize,
}

impl<M> stateright::report::Reporter<M> for Reporter
where
    M: Model,
{
    fn report_checking(&mut self, data: stateright::report::ReportData) {
        let new_total = data.total_states - self.last_total;
        let total_rate = (data.total_states as f64 / data.duration.as_secs_f64()).round() as u64;
        let new_unique = data.unique_states - self.last_unique;
        let unique_rate = (data.unique_states as f64 / data.duration.as_secs_f64()).round() as u64;
        let max_depth = data.max_depth;
        let status = if data.done { "Done    " } else { "Checking" };
        let locale = SystemLocale::default().unwrap();
        let duration = Duration::new(data.duration.as_secs(), 0);
        println!(
            "{} states={: >8} (+{: <8} {: >8}/s), unique={: >8} (+{: <8} {: >8}/s), max_depth={}, duration={:?}",
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
        discoveries: std::collections::HashMap<
            &'static str,
            stateright::report::ReportDiscovery<M>,
        >,
    ) where
        <M as Model>::Action: std::fmt::Debug,
        <M as Model>::State: std::fmt::Debug,
        <M as Model>::State: Hash,
    {
        let discoveries: BTreeMap<_, _> = discoveries.into_iter().collect();
        for (name, discovery) in discoveries {
            print!(
                "Discovered \"{}\" {} {}",
                name, discovery.classification, discovery.path,
            );
            println!(
                "To explore this path try re-running with `explore {}`",
                discovery.path.encode()
            );
        }
    }
}
